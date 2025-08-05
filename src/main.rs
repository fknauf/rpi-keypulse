use clap::Parser;
use evdev::{Device, EventType};
use std::time::Duration;
use tokio::signal::unix::{SignalKind, signal};
use tokio::time::sleep;
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
    /// Device node for the keyboard
    #[arg(short, long, default_value = "/dev/input/event0")]
    device: String,

    /// Number of the GPIO pin to pulse
    #[arg(short, long, default_value_t = 26)]
    pin: u8,

    /// Length of the pulse in microseconds
    #[arg(short = 'l', long, default_value_t = 1000)]
    pulse_length_us: u64,
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
    let dev = Device::open(args.device)?;
    let mut events = dev.into_event_stream()?;
    let mut sigterm = signal(SignalKind::terminate())?;
    let tracker = TaskTracker::new();

    let gpio = Gpio::new()?;

    // init: configure so that transistor's gate is always driven into the ground when no keys are pressed.
    gpio.get(args.pin)?
        .into_output_low()
        .set_reset_on_drop(false);

    loop {
        // If you ask me, this should be part of the evdev crate. But it isn't, so
        // I make my own named constant with blackjack and hookers.
        const KEYPRESS_DOWN: i32 = 1;

        tokio::select! {
            Ok(ev) = events.next_event() => {
                if ev.event_type() == EventType::KEY && ev.value() == KEYPRESS_DOWN {
                    tracker.spawn(plopp(gpio.get(args.pin), pulse_length));
                }
            },
            _ = tokio::signal::ctrl_c() => { break }
            _ = sigterm.recv() => { break }
        }
    }

    tracker.close();
    tracker.wait().await;

    Ok(())
}
