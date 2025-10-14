use clap::Parser;
use evdev::EventSummary;
use inotify::{Inotify, WatchMask};
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

mod keys;
use keys::{KeyClasses, KeyState};

/// Pulse on a GPIO pin every time a key is pressed
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Number of the GPIO pin to pulse
    #[arg(short, long, default_value_t = 26)]
    pin: u8,

    /// Length of the pulse in microseconds
    #[arg(short = 'l', long, default_value_t = 30000)]
    pulse_length_us: u64,

    /// Make GPIO pulses inactive at program startup (until the activation shortcut is pressed)
    #[arg(long)]
    start_inactive: bool,

    /// By default, modifier keys and Esc don't cause a pulse. This flag makes them behave like regular keys.
    #[arg(long)]
    no_dead_keys: bool,
}

fn open_hotplug_stream() -> Result<inotify::EventStream<[u8; 4096]>, std::io::Error> {
    let inotify = Inotify::init()?;
    inotify
        .watches()
        .add("/dev/input", WatchMask::CREATE | WatchMask::DELETE)?;
    inotify.into_event_stream([0; 4096])
}

fn open_keyboard_event_stream() -> StreamMap<usize, evdev::EventStream> {
    evdev::enumerate()
        .map(|t| t.1)
        .filter_map(|dev| dev.into_event_stream().ok())
        .enumerate()
        .collect::<StreamMap<_, _>>()
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

    let mut sigterm = signal(SignalKind::terminate())?;
    let tracker = TaskTracker::new();

    let pulse_length = Duration::from_micros(args.pulse_length_us);
    let key_classes = KeyClasses::new(args.no_dead_keys);
    let mut key_state = KeyState::new();
    let mut plopp_active = !args.start_inactive;

    let mut hotplug_stream = open_hotplug_stream()?;
    let mut events = open_keyboard_event_stream();

    let gpio = Gpio::new()?;
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
                        if plopp_active && key_classes.is_plopping(code) {
                            tracker.spawn(plopp(gpio.get(args.pin), pulse_length));
                        }

                        key_state.insert(code);
                    },
                    EventSummary::Key(_, code, KEYPRESS_UP) => {
                        if key_state.is_activation_combo() {
                            plopp_active = !plopp_active;
                        }

                        key_state.remove(code);
                    },
                    _ => {}
                }
            }
            _ = hotplug_stream.next() => {
                println!("Device (un)plugged, re-opening.");
                events = open_keyboard_event_stream()
            }
            _ = tokio::signal::ctrl_c() => { break }
            _ = sigterm.recv() => { break }
        }
    }

    tracker.close();
    tracker.wait().await;

    Ok(())
}
