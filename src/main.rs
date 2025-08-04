use std::time::Duration;
use tokio::time::sleep;
use evdev::{ Device, EventSummary };

async fn plopp() {
    println!("pin down");
    sleep(Duration::from_millis(1)).await;
    println!("pin up");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dev_path = std::env::args().nth(1).expect("No device specified. Specify /dev/input/event# as first argument.");

    let dev = Device::open(dev_path).unwrap();
    let mut events = dev.into_event_stream()?;

    loop {
        let ev = events.next_event().await?;

        match ev.destructure() {
            EventSummary::Key(_, _, 1) => {
                tokio::spawn(plopp());
            },
            _ => ()
        }
    }
}
