use clap::Parser;
use evdev::{Device, EventSummary, EventType, KeyCode};
use std::collections::HashSet;
use std::time::Duration;
use tokio::signal::unix::{SignalKind, signal};
use tokio::time::sleep;
use tokio_stream::{StreamExt, StreamMap};
use tokio_util::task::TaskTracker;

#[cfg(feature = "gpio")]
use rppal::gpio::{Gpio, Pin};

#[cfg(not(feature = "gpio"))]
mod dummy_gpio;
#[cfg(not(feature = "gpio"))]
use dummy_gpio::{Gpio, Pin};

/// Pulse on a GPIO pin every time a key is pressed
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Device node for the keyboard, e.g. /dev/input/event0.
    /// If not specified, all devices will be polled for keyboard events.
    #[arg(short, long)]
    device: Option<String>,

    /// Number of the GPIO pin to pulse
    #[arg(short, long, default_value_t = 26)]
    pin: u8,

    /// Length of the pulse in microseconds
    #[arg(short = 'l', long, default_value_t = 30000)]
    pulse_length_us: u64,
}

fn open_keyboard(devpath_arg: Option<String>) -> Result<Vec<Device>, std::io::Error> {
    if let Some(devpath) = devpath_arg {
        Device::open(devpath).and_then(|dev| Ok(vec![dev]))
    } else {
        Ok(evdev::enumerate().map(|t| t.1).collect())
    }
}

fn normalize_modifier(code: KeyCode) -> KeyCode {
    match code {
        KeyCode::KEY_RIGHTCTRL => KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_RIGHTALT => KeyCode::KEY_LEFTALT,
        KeyCode::KEY_RIGHTMETA => KeyCode::KEY_LEFTMETA,
        KeyCode::KEY_RIGHTSHIFT => KeyCode::KEY_LEFTSHIFT,
        other => other,
    }
}

async fn plopp(maybe_pin: Result<Pin, rppal::gpio::Error>, pulse_length: Duration) {
    match maybe_pin {
        Ok(pin) => {
            let mut out = pin.into_output();

            out.set_high();
            sleep(pulse_length).await;
            out.set_low();
        }
        Err(_) => {
            println!("Typing too fast! GPIO pin is still in use.");
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let pulse_length = Duration::from_micros(args.pulse_length_us);
    let mut sigterm = signal(SignalKind::terminate())?;
    let tracker = TaskTracker::new();
    let gpio = Gpio::new()?;
    let devices = open_keyboard(args.device)?;

    let mut key_state = HashSet::new();
    let activation_keycombo = HashSet::from([
        KeyCode::KEY_LEFTSHIFT,
        KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_LEFTMETA,
        KeyCode::KEY_S,
    ]);
    let mut plopp_active = false;

    let mut events = devices
        .into_iter()
        .filter_map(|dev| dev.into_event_stream().ok())
        .enumerate()
        .collect::<StreamMap<_, _>>();

    // init: configure so that transistor's gate is always driven into the ground when no keys are pressed.
    gpio.get(args.pin)?
        .into_output_low()
        .set_reset_on_drop(false);

    loop {
        // If you ask me, this should be part of the evdev crate. But it isn't, so
        // I make my own named constant with blackjack and hookers.
        const KEYPRESS_UP: i32 = 0;
        const KEYPRESS_DOWN: i32 = 1;

        tokio::select! {
            Some((_, Ok(ev))) = events.next() => {
                match ev.destructure() {
                    EventSummary::Key(_, code, KEYPRESS_DOWN) => {
                        if plopp_active && ev.event_type() == EventType::KEY && ev.value() == KEYPRESS_DOWN {
                            tracker.spawn(plopp(gpio.get(args.pin), pulse_length));
                        }

                        key_state.insert(normalize_modifier(code));

                        if key_state == activation_keycombo {
                            plopp_active = !plopp_active;
                        }
                    },
                    EventSummary::Key(_, code, KEYPRESS_UP) => {
                        let normalized_code = normalize_modifier(code);
                        key_state.remove(&normalized_code);
                    },
                    _ => {}
                }
            }
            _ = tokio::signal::ctrl_c() => { break }
            _ = sigterm.recv() => { break }
        }
    }

    tracker.close();
    tracker.wait().await;

    Ok(())
}
