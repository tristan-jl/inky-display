use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use spidev::{SpiModeFlags, Spidev, SpidevOptions};

use crate::AppError;

const WIDTH: usize = 800;
const HEIGHT: usize = 480;
const ROTATION: u8 = 0;

/// GPIO pin numbers (BCM)
const RESET_PIN: u32 = 27;
const BUSY_PIN: u32 = 17;
const DC_PIN: u32 = 22;
const CS0_PIN: u32 = 8;

/// SPI settings
pub const SPI_DEVICE: &str = "/dev/spidev0.0";
pub const SPI_MAX_SPEED_HZ: u32 = 1_000_000; // 1 MHz

/// Wrapper for the GPIO lines we need
#[derive(Debug)]
pub struct GpioLines {
    pub cs: LineHandle,
    pub dc: LineHandle,
    pub reset: LineHandle,
    pub busy: LineHandle,
}

impl GpioLines {
    pub fn new(cs: u32, dc: u32, reset: u32, busy: u32) -> Result<Self, AppError> {
        let mut chip = Chip::new("/dev/gpiochip0")?;

        let cs = chip
            .get_line(cs)?
            .request(LineRequestFlags::OUTPUT, 1, "inky-cs")?;
        let dc = chip
            .get_line(dc)?
            .request(LineRequestFlags::OUTPUT, 0, "inky-dc")?;
        let reset = chip
            .get_line(reset)?
            .request(LineRequestFlags::OUTPUT, 1, "inky-reset")?;
        let busy = chip
            .get_line(busy)?
            .request(LineRequestFlags::INPUT, 0, "inky-busy")?;

        Ok(GpioLines {
            cs,
            dc,
            reset,
            busy,
        })
    }
}

/// Main Inky struct for E673
#[derive(Debug)]
pub struct Inky {
    spi: Spidev,
    gpio: GpioLines,
    buf: Vec<u8>, // pixel buffer, 0..7 per pixel
    width: usize,
    height: usize,
    h_flip: bool,
    v_flip: bool,
}

impl Inky {
    /// Constructor: initializes SPI and GPIO lines
    pub fn new() -> Result<Self, AppError> {
        // Open SPI device
        let mut spi = Spidev::open(SPI_DEVICE)?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(SPI_MAX_SPEED_HZ)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)?;

        // Initialize GPIO lines using constants
        let gpio = GpioLines::new(CS0_PIN, DC_PIN, RESET_PIN, BUSY_PIN)?;

        // Initialize pixel buffer
        let buf = vec![0u8; WIDTH * HEIGHT];

        Ok(Inky {
            spi,
            gpio,
            buf,
            width: WIDTH,
            height: HEIGHT,
            h_flip: false,
            v_flip: false,
        })
    }
}
