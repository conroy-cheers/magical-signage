use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    prelude::*,
    primitives::PrimitiveStyle,
    text::{Alignment, Text},
};
use epd_waveshare::{epd2in66b::*, prelude::*};

pub(crate) fn draw(text: &str) -> Display2in66b {
    // Create a Display buffer to draw on, specific for this ePaper
    let mut display = Display2in66b::default();

    // Landscape mode, USB plug to the right
    display.set_rotation(DisplayRotation::Rotate270);

    // let style = MonoTextStyleBuilder::new()
    //     .font(&embedded_graphics::mono_font::ascii::FONT_10X20)
    //     .text_color(TriColor::Black)
    //     .background_color(TriColor::White)
    //     .build();

    // Change the background from the default black to white
    let _ = display
        .bounding_box()
        .into_styled(PrimitiveStyle::with_fill(TriColor::White))
        .draw(&mut display);

    // Draw some text on the buffer
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(1, 0),
        MonoTextStyle::new(&FONT_10X20, TriColor::Black),
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();
    Text::with_alignment(
        text,
        display.bounding_box().center() + Point::new(0, 1),
        MonoTextStyle::new(&FONT_10X20, TriColor::Chromatic),
        Alignment::Center,
    )
    .draw(&mut display)
    .unwrap();

    display
}
