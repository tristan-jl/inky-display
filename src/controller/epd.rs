use super::PascalString;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use std::io::{Cursor, Read};

const EEPROM_PATH: &str = "/dev/i2c-1";
const EEP_ADDRESS: u16 = 0x50;
const EEPROM_SIZE: usize = 29;

#[derive(Debug)]
#[repr(C)]
pub struct EPDType {
    width: u16,
    height: u16,
    colour: u8,
    pcb_variant: u8,
    display_variant: u8,
    eeprom_write_time: PascalString,
}

impl std::fmt::Display for EPDType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Dimensions (wxh): {}x{}, colour: {}, pcb_variant: {}, eeprom_write_time: {}",
            self.width, self.height, self.colour, self.pcb_variant, self.eeprom_write_time
        )
    }
}

impl EPDType {
    fn from_bytes(data: &[u8]) -> Result<Self> {
        if data.len() < EEPROM_SIZE {
            anyhow::bail!("Data len too short");
        }

        let mut rdr = Cursor::new(data);
        let width = rdr.read_u16::<LittleEndian>()?;
        let height = rdr.read_u16::<LittleEndian>()?;
        let color = rdr.read_u8()?;
        let pcb_variant = rdr.read_u8()?;
        let display_variant = rdr.read_u8()?;

        if display_variant != 22 {
            anyhow::bail!(
                "Was expecting display_variant=22, corresponding to 'Spectra 6 7.3 800 x 480 (E673)', got: {display_variant}",
            );
        }

        let mut eeprom_write_time = PascalString::with_len(rdr.read_u8()?);
        rdr.read(&mut eeprom_write_time.chars)?;

        Ok(Self {
            width,
            height,
            colour: color,
            pcb_variant,
            display_variant,
            eeprom_write_time,
        })
    }

    pub fn from_eeprom_block() -> Result<Self> {
        let mut dev = LinuxI2CDevice::new(EEPROM_PATH, EEP_ADDRESS)?;

        // Set address pointer to 0x0000
        dev.write(&[0x00, 0x00])?;

        // Read 29 bytes
        let mut buf = [0u8; EEPROM_SIZE];
        dev.read(&mut buf)?;

        Self::from_bytes(&buf)
    }
}
