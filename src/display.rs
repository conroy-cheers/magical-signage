use embedded_hal_1::delay::DelayUs;
use embedded_hal_1::digital::{InputPin, OutputPin};
use embedded_hal_1::spi::SpiDevice;
use epd_waveshare::{epd2in66b::*, prelude::*};

use crate::graphics;

pub(crate) fn display_frame<SPI, BUSY, DC, RST, DLY>(
    text: &str,
    e_paper: &mut Epd2in66b<SPI, BUSY, DC, RST, DLY>,
    spi_device: &mut SPI,
    delay: &mut DLY,
) where
    SPI: SpiDevice,
    BUSY: InputPin,
    DC: OutputPin,
    RST: OutputPin,
    DLY: DelayUs,
{
    let display = graphics::draw(&text);
    e_paper.wake_up(spi_device, delay).unwrap();
    e_paper
        .update_color_frame(
            spi_device,
            delay,
            &display.bw_buffer(),
            &display.chromatic_buffer(),
        )
        .expect("disaster!");
    // Render the ePaper RAM - takes time.
    e_paper.display_frame(spi_device, delay).unwrap();
    e_paper.sleep(spi_device, delay).unwrap();
}
