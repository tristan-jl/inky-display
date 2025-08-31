use inky_display::controller::EPDType;

fn main() {
    let epd = EPDType::from_eeprom_block().unwrap();

    println!("{epd:?}");
    println!("{epd}");
}
