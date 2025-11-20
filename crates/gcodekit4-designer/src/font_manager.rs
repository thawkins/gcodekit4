use rusttype::Font;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref FONT: Font<'static> = {
        let font_data = include_bytes!("../../../assets/fonts/fira-code/FiraCode-Regular.ttf");
        Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font")
    };
}

pub fn get_font() -> &'static Font<'static> {
    &FONT
}
