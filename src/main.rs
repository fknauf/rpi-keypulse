use std::time::Duration;
use tokio::time::sleep;
use evdev::{ Device, EventSummary };
use rppal::gpio::{ Gpio, OutputPin };
use clap::Parser;

/// Pulse on a GPIO pin every time a key is pressed
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Device node for the keyboard
    #[arg(short, long, default_value = "/dev/input/event0")]
    device: String,

    /// Number of the GPIO pin to pulse
    #[arg(short, long)]
    pin: u8,

    /// Length of the pulse in microseconds
    #[arg(short = 'l', long, default_value_t = 1000)]
    pulse_length_us: u64
}

async fn plopp(mut pin: OutputPin, pulse_us: u64) {
    pin.set_high();
    sleep(Duration::from_micros(pulse_us)).await;
    pin.set_low();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let dev = Device::open(args.device)?;
    let mut events = dev.into_event_stream()?;
    let gpio = Gpio::new()?;

    loop {
        let ev = events.next_event().await?;

        match ev.destructure() {
            EventSummary::Key(_, _, 1) => {
                match gpio.get(args.pin) {
                    Ok(pin) => {
                        tokio::spawn(plopp(pin.into_output(), args.pulse_length_us));
                    },
                    Err(_) => {
                        println!("Typing too fast! GPIO pin is still in use.");
                    }
                }
            },
            _ => ()
        }
    }
}
