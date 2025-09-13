use super::{ColourSpace, EPDType, Palette, quantise_image};
use crate::AppError;
use crate::controller::quantise_and_dither_image;
use gpio_cdev::{Chip, LineHandle, LineRequestFlags};
use image::RgbImage;
use spidev::{SpiModeFlags, Spidev, SpidevOptions};
use std::io::Write;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct GpioLines {
    pub cs: LineHandle,
    pub dc: LineHandle,
    pub reset: LineHandle,
    pub busy: LineHandle,
    pub led: LineHandle,
}

impl GpioLines {
    fn new(
        chip: &mut Chip,
        cs: u32,
        dc: u32,
        reset: u32,
        busy: u32,
        led: u32,
    ) -> Result<Self, AppError> {
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
        let led = chip
            .get_line(led)?
            .request(LineRequestFlags::OUTPUT, 0, "inky-led")?;

        Ok(GpioLines {
            cs,
            dc,
            reset,
            busy,
            led,
        })
    }
}

// #[derive(Debug, Clone, Copy)]
// #[repr(u8)]
// pub enum InkyColour {
//     Black = 0,
//     White = 1,
//     Yellow = 2,
//     Red = 3,
//     // Note: 4 is missing
//     Blue = 5,
//     Green = 6,
// }

// impl From<u8> for InkyColour {
//     fn from(value: u8) -> Self {
//         match value {
//             0 => InkyColour::Black,
//             // 1 => InkyColour::White,
//             2 => InkyColour::Yellow,
//             3 => InkyColour::Red,
//             // No colour 4
//             5 => InkyColour::Blue,
//             6 => InkyColour::Green,
//             // Default to white, note: 1 is also white
//             _ => InkyColour::White,
//         }
//     }
// }

#[derive(Debug, Clone, Copy, Eq, PartialEq, std::hash::Hash)]
#[repr(u8)]
pub enum InkyColour {
    Black = 0,
    Blue = 1,
    Yellow = 2,
    Red = 3,
    Green = 4,
    White = 5,
}

impl From<u8> for InkyColour {
    fn from(value: u8) -> Self {
        match value {
            0 => InkyColour::Black,
            1 => InkyColour::Blue,
            2 => InkyColour::Yellow,
            3 => InkyColour::Red,
            4 => InkyColour::Green,
            5 => InkyColour::White,
            _ => {
                tracing::warn!("Got inky colour out of range: '{value}'");
                InkyColour::White
            }
        }
    }
}

#[derive(Debug)]
pub struct Inky {
    spi: Spidev,
    eeprom: EPDType,
    chip: Chip,
    gpio: GpioLines,
    pub buf: [InkyColour; Self::HEIGHT * Self::WIDTH],

    palette: Palette,
    colour_space: ColourSpace,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum LedState {
    Off = 0,
    On = 1,
}

impl Inky {
    pub const WIDTH: usize = 800;
    pub const HEIGHT: usize = 480;

    const RESET_PIN: u32 = 27;
    const BUSY_PIN: u32 = 17;
    const DC_PIN: u32 = 22;
    const LED_PIN: u32 = 13;

    const MOSI_PIN: u32 = 10;
    const SCLK_PIN: u32 = 11;
    const CS0_PIN: u32 = 8;

    const EL673_PSR: u8 = 0x00;
    const EL673_PWR: u8 = 0x01;
    const EL673_POF: u8 = 0x02;
    const EL673_POFS: u8 = 0x03;
    const EL673_PON: u8 = 0x04;
    const EL673_BTST1: u8 = 0x05;
    const EL673_BTST2: u8 = 0x06;
    const EL673_DSLP: u8 = 0x07;
    const EL673_BTST3: u8 = 0x08;
    const EL673_DTM1: u8 = 0x10;
    const EL673_DSP: u8 = 0x11;
    const EL673_DRF: u8 = 0x12;
    const EL673_PLL: u8 = 0x30;
    const EL673_CDI: u8 = 0x50;
    const EL673_TCON: u8 = 0x60;
    const EL673_TRES: u8 = 0x61;
    const EL673_REV: u8 = 0x70;
    const EL673_VDCS: u8 = 0x82;
    const EL673_PWS: u8 = 0xE3;

    const SPI_DEVICE: &str = "/dev/spidev";
    const SPI_MAX_SPEED_HZ: u32 = 1_000_000;
    const SPI_CHUNK_SIZE: usize = 4096;

    const DESATURATED_PALETTE_COLOURS: [[u8; 3]; 6] = [
        [0, 0, 0],
        [255, 255, 255],
        [255, 255, 0],
        [255, 0, 0],
        [0, 255, 0],
        [255, 255, 255],
    ];

    const SATURATED_PALETTE_COLOURS: [[u8; 3]; 6] = [
        [0, 0, 0],
        [161, 164, 165],
        [208, 190, 71],
        [156, 72, 75],
        [58, 91, 70],
        [255, 255, 255],
    ];

    fn from_eeprom(eeprom: EPDType, saturation: f32) -> Result<Self, AppError> {
        let mut spi = Spidev::open(format!("{}{}.{}", Self::SPI_DEVICE, 0, 0))?;
        let options = SpidevOptions::new()
            .bits_per_word(8)
            .max_speed_hz(Self::SPI_MAX_SPEED_HZ)
            .mode(SpiModeFlags::SPI_MODE_0)
            .build();
        spi.configure(&options)?;

        let mut chip = Chip::new("/dev/gpiochip0")?;
        let gpio = GpioLines::new(
            &mut chip,
            Self::CS0_PIN,
            Self::DC_PIN,
            Self::RESET_PIN,
            Self::BUSY_PIN,
            Self::LED_PIN,
        )?;

        let buf = [const { InkyColour::White }; Self::WIDTH * Self::HEIGHT];

        Ok(Inky {
            spi,
            eeprom,
            chip,
            gpio,
            buf,
            palette: Palette::from_blend(
                &Self::DESATURATED_PALETTE_COLOURS,
                &Self::SATURATED_PALETTE_COLOURS,
                saturation,
            )?,
            colour_space: ColourSpace::CIELAB,
        })
    }

    pub fn new(saturation: f32) -> Result<Self, AppError> {
        let epd = EPDType::from_eeprom_block()?;
        tracing::info!("Creating new Inky");
        Inky::from_eeprom(epd, saturation)
    }

    pub fn set_led(&mut self, led_state: LedState) -> Result<(), AppError> {
        self.gpio.led.set_value(led_state as u8)?;
        Ok(())
    }

    fn setup(&mut self) -> Result<(), AppError> {
        // Reset sequence
        self.gpio.reset.set_value(0)?; // INACTIVE
        std::thread::sleep(Duration::from_millis(30));
        self.gpio.reset.set_value(1)?; // ACTIVE
        std::thread::sleep(Duration::from_millis(30));

        self.busy_wait(Some(0.3))?;

        // Send startup commands
        self.send_command(0xAA, Some(&[0x49, 0x55, 0x20, 0x08, 0x09, 0x18]))?;
        self.send_command(Self::EL673_PWR, Some(&[0x3F]))?;
        self.send_command(Self::EL673_PSR, Some(&[0x5F, 0x69]))?;
        self.send_command(Self::EL673_BTST1, Some(&[0x40, 0x1F, 0x1F, 0x2C]))?;
        self.send_command(Self::EL673_BTST3, Some(&[0x6F, 0x1F, 0x1F, 0x22]))?;
        self.send_command(Self::EL673_BTST2, Some(&[0x6F, 0x1F, 0x17, 0x17]))?;
        self.send_command(Self::EL673_POFS, Some(&[0x00, 0x54, 0x00, 0x44]))?;
        self.send_command(Self::EL673_TCON, Some(&[0x02, 0x00]))?;
        self.send_command(Self::EL673_PLL, Some(&[0x08]))?;
        self.send_command(Self::EL673_CDI, Some(&[0x3F]))?;
        self.send_command(Self::EL673_TRES, Some(&[0x03, 0x20, 0x01, 0xE0]))?;
        self.send_command(Self::EL673_PWS, Some(&[0x2F]))?;
        self.send_command(Self::EL673_VDCS, Some(&[0x01]))?;

        Ok(())
    }

    fn send_command(&mut self, command: u8, data: Option<&[u8]>) -> Result<(), AppError> {
        self.gpio.cs.set_value(0)?; // INACTIVE
        self.gpio.dc.set_value(0)?; // Command mode
        std::thread::sleep(Duration::from_millis(300));

        let _ = self.spi.write(&[command])?;

        if let Some(d) = data {
            self.gpio.dc.set_value(1)?; // Data mode
            for chunk in d.chunks(Self::SPI_CHUNK_SIZE) {
                let _ = self.spi.write(chunk)?;
            }
        }

        self.gpio.cs.set_value(1)?; // Deselect device
        self.gpio.dc.set_value(0)?; // Reset DC to command mode
        Ok(())
    }

    fn busy_wait(&self, timeout: Option<f64>) -> Result<(), AppError> {
        let timeout = timeout.unwrap_or(40.0);

        // If busy pin is high initially, just wait full timeout
        if self.gpio.busy.get_value()? == 1 {
            std::thread::sleep(Duration::from_secs_f64(timeout));
            return Ok(());
        }

        let start = Instant::now();
        while self.gpio.busy.get_value()? != 1 {
            std::thread::sleep(Duration::from_millis(100));
            if start.elapsed().as_secs_f64() > timeout {
                tracing::warn!("Busy Wait: Timed out after {:.2}s", timeout);
                return Ok(()); // or Err if you want a real error
            }
        }

        Ok(())
    }

    fn update(&mut self, buf: &[u8]) -> Result<(), AppError> {
        // Ensure GPIO & SPI are set up and reset the display
        self.setup()?;

        // Send main buffer
        self.send_command(Self::EL673_DTM1, Some(buf))?;

        // Power on
        self.send_command(Self::EL673_PON, None)?;
        self.busy_wait(Some(0.3))?;

        // Second setting of BTST2 register
        self.send_command(Self::EL673_BTST2, Some(&[0x6F, 0x1F, 0x17, 0x49]))?;

        // Display refresh
        self.send_command(Self::EL673_DRF, Some(&[0x00]))?;
        self.busy_wait(Some(32.0))?;

        // Power off
        self.send_command(Self::EL673_POF, Some(&[0x00]))?;
        self.busy_wait(Some(0.3))?;

        Ok(())
    }

    fn update_display(&mut self) -> Result<(), AppError> {
        tracing::info!("Updating display...");
        let mut res = Vec::with_capacity(Self::WIDTH * Self::HEIGHT / 2);
        for i in self.buf.chunks(2) {
            let l = i[0] as u8;
            let r = i[1] as u8;
            res.push(l << 4 | r);
        }

        // Send to display
        self.update(&res)?;

        Ok(())
    }

    fn set_image(&mut self, image: &mut RgbImage, should_dither: bool) -> Result<(), AppError> {
        if image.width() as usize != Self::WIDTH || image.height() as usize != Self::HEIGHT {
            return Err(AppError::InvalidInput(std::borrow::Cow::Owned(format!(
                "Image incorrect size: {}x{}",
                image.width(),
                image.height()
            ))));
        }

        if should_dither {
            tracing::info!("Quantising and dithering...");
            quantise_and_dither_image(image, &self.palette, self.colour_space);
        } else {
            tracing::info!("Quantising...");
            quantise_image(image, &self.palette, self.colour_space);
        }
        tracing::info!("Done");

        tracing::info!("Setting buffer...");
        let mut map = std::collections::HashMap::new();
        for (i, pixel) in image.pixels().enumerate() {
            let mfdsafds = InkyColour::from(self.palette.to_idx(&pixel.0));
            map.insert(mfdsafds, pixel.0);
            self.buf[i] = mfdsafds;
        }
        tracing::info!("Done");
        tracing::info!("hashmap: {map:?}");

        Ok(())
    }

    pub fn set_stripes(&mut self) -> Result<(), AppError> {
        tracing::info!("Setting buffer...");
        let total = Self::HEIGHT * Self::WIDTH;
        for i in 0..total {
            self.buf[i] = InkyColour::from(((i * Self::WIDTH / 6) % 6) as u8);
        }
        tracing::info!("Done");
        self.update_display()?;

        Ok(())
    }

    pub fn set_display(
        &mut self,
        image: &mut RgbImage,
        should_dither: bool,
    ) -> Result<(), AppError> {
        self.set_image(image, should_dither)?;
        self.update_display()?;

        Ok(())
    }

    pub fn set_saturation(&mut self, saturation: f32) -> Result<(), AppError> {
        self.palette = Palette::from_blend(
            &Self::DESATURATED_PALETTE_COLOURS,
            &Self::SATURATED_PALETTE_COLOURS,
            saturation,
        )?;
        tracing::info!("Set saturation to {saturation}");

        Ok(())
    }
}
