use clap::Parser;
use evdev::{Device, EventSummary, KeyCode};
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

    /// Make GPIO pulses active at program startup
    #[arg(long)]
    start_active: bool,

    /// By default, modifier keys and Esc don't cause a pulse. This flag makes them behave like regular keys.
    #[arg(long)]
    no_dead_keys: bool,
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

fn is_plopping_key(code: KeyCode, no_dead_keys: bool) -> bool {
    let key = normalize_modifier(code);
    let dead_keys = [
        KeyCode::KEY_ESC,
        KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_LEFTALT,
        KeyCode::KEY_LEFTMETA,
        KeyCode::KEY_LEFTSHIFT,
    ];
    let non_kbd_keys = [
        KeyCode::BTN_0,
        KeyCode::BTN_1,
        KeyCode::BTN_2,
        KeyCode::BTN_3,
        KeyCode::BTN_4,
        KeyCode::BTN_5,
        KeyCode::BTN_6,
        KeyCode::BTN_7,
        KeyCode::BTN_8,
        KeyCode::BTN_9,
        KeyCode::BTN_LEFT,
        KeyCode::BTN_RIGHT,
        KeyCode::BTN_MIDDLE,
        KeyCode::BTN_SIDE,
        KeyCode::BTN_EXTRA,
        KeyCode::BTN_FORWARD,
        KeyCode::BTN_BACK,
        KeyCode::BTN_TASK,
        KeyCode::BTN_TRIGGER,
        KeyCode::BTN_THUMB,
        KeyCode::BTN_THUMB2,
        KeyCode::BTN_TOP,
        KeyCode::BTN_TOP2,
        KeyCode::BTN_PINKIE,
        KeyCode::BTN_BASE,
        KeyCode::BTN_BASE2,
        KeyCode::BTN_BASE3,
        KeyCode::BTN_BASE4,
        KeyCode::BTN_BASE5,
        KeyCode::BTN_BASE6,
        KeyCode::BTN_DEAD,
        KeyCode::BTN_SOUTH,
        KeyCode::BTN_EAST,
        KeyCode::BTN_C,
        KeyCode::BTN_NORTH,
        KeyCode::BTN_WEST,
        KeyCode::BTN_Z,
        KeyCode::BTN_TL,
        KeyCode::BTN_TR,
        KeyCode::BTN_TL2,
        KeyCode::BTN_TR2,
        KeyCode::BTN_SELECT,
        KeyCode::BTN_START,
        KeyCode::BTN_MODE,
        KeyCode::BTN_THUMBL,
        KeyCode::BTN_THUMBR,
        KeyCode::BTN_TOOL_PEN,
        KeyCode::BTN_TOOL_RUBBER,
        KeyCode::BTN_TOOL_BRUSH,
        KeyCode::BTN_TOOL_PENCIL,
        KeyCode::BTN_TOOL_AIRBRUSH,
        KeyCode::BTN_TOOL_FINGER,
        KeyCode::BTN_TOOL_MOUSE,
        KeyCode::BTN_TOOL_LENS,
        KeyCode::BTN_TOOL_QUINTTAP,
        KeyCode::BTN_TOUCH,
        KeyCode::BTN_STYLUS,
        KeyCode::BTN_STYLUS2,
        KeyCode::BTN_TOOL_DOUBLETAP,
        KeyCode::BTN_TOOL_TRIPLETAP,
        KeyCode::BTN_TOOL_QUADTAP,
        KeyCode::BTN_GEAR_DOWN,
        KeyCode::BTN_GEAR_UP,
        KeyCode::BTN_DPAD_UP,
        KeyCode::BTN_DPAD_DOWN,
        KeyCode::BTN_DPAD_LEFT,
        KeyCode::BTN_DPAD_RIGHT,
        KeyCode::BTN_TRIGGER_HAPPY1,
        KeyCode::BTN_TRIGGER_HAPPY2,
        KeyCode::BTN_TRIGGER_HAPPY3,
        KeyCode::BTN_TRIGGER_HAPPY4,
        KeyCode::BTN_TRIGGER_HAPPY5,
        KeyCode::BTN_TRIGGER_HAPPY6,
        KeyCode::BTN_TRIGGER_HAPPY7,
        KeyCode::BTN_TRIGGER_HAPPY8,
        KeyCode::BTN_TRIGGER_HAPPY9,
        KeyCode::BTN_TRIGGER_HAPPY10,
        KeyCode::BTN_TRIGGER_HAPPY11,
        KeyCode::BTN_TRIGGER_HAPPY12,
        KeyCode::BTN_TRIGGER_HAPPY13,
        KeyCode::BTN_TRIGGER_HAPPY14,
        KeyCode::BTN_TRIGGER_HAPPY15,
        KeyCode::BTN_TRIGGER_HAPPY16,
        KeyCode::BTN_TRIGGER_HAPPY17,
        KeyCode::BTN_TRIGGER_HAPPY18,
        KeyCode::BTN_TRIGGER_HAPPY19,
        KeyCode::BTN_TRIGGER_HAPPY20,
        KeyCode::BTN_TRIGGER_HAPPY21,
        KeyCode::BTN_TRIGGER_HAPPY22,
        KeyCode::BTN_TRIGGER_HAPPY23,
        KeyCode::BTN_TRIGGER_HAPPY24,
        KeyCode::BTN_TRIGGER_HAPPY25,
        KeyCode::BTN_TRIGGER_HAPPY26,
        KeyCode::BTN_TRIGGER_HAPPY27,
        KeyCode::BTN_TRIGGER_HAPPY28,
        KeyCode::BTN_TRIGGER_HAPPY29,
        KeyCode::BTN_TRIGGER_HAPPY30,
        KeyCode::BTN_TRIGGER_HAPPY31,
        KeyCode::BTN_TRIGGER_HAPPY32,
        KeyCode::BTN_TRIGGER_HAPPY33,
        KeyCode::BTN_TRIGGER_HAPPY34,
        KeyCode::BTN_TRIGGER_HAPPY35,
        KeyCode::BTN_TRIGGER_HAPPY36,
        KeyCode::BTN_TRIGGER_HAPPY37,
        KeyCode::BTN_TRIGGER_HAPPY38,
        KeyCode::BTN_TRIGGER_HAPPY39,
        KeyCode::BTN_TRIGGER_HAPPY40,
    ];

    !non_kbd_keys.contains(&key) && (no_dead_keys || !dead_keys.contains(&key))
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
    let mut plopp_active = args.start_active;

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
                        if plopp_active && is_plopping_key(code, args.no_dead_keys) {
                            tracker.spawn(plopp(gpio.get(args.pin), pulse_length));
                        }

                        key_state.insert(normalize_modifier(code));
                    },
                    EventSummary::Key(_, code, KEYPRESS_UP) => {
                        if key_state == activation_keycombo {
                            plopp_active = !plopp_active;
                        }

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
