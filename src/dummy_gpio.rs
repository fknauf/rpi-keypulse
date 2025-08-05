use rppal::gpio::Error;

pub struct Gpio {}

pub struct Pin {}

pub struct OutputPin {}

impl Gpio {
    pub fn new() -> Result<Gpio, Error> {
        Ok(Gpio {})
    }

    pub fn get(self: &Self, _: u8) -> Result<Pin, Error> {
        Ok(Pin {})
    }
}

impl Pin {
    pub fn into_output(self: Self) -> OutputPin {
        OutputPin {}
    }

    pub fn into_output_low(self: Self) -> OutputPin {
        OutputPin {}
    }
}

impl OutputPin {
    pub fn set_low(self: &mut Self) {
        println!("pin low");
    }

    pub fn set_high(self: &mut Self) {
        println!("pin high");
    }

    pub fn set_reset_on_drop(self: &mut Self, _: bool) {}
}
