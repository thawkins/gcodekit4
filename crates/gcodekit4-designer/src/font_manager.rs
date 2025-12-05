use rusttype::Font;
use std::sync::OnceLock;

pub fn get_font() -> &'static Font<'static> {
    static FONT: OnceLock<Font<'static>> = OnceLock::new();
    FONT.get_or_init(|| {
        let font_data = include_bytes!("../../../assets/fonts/fira-code/FiraCode-Regular.ttf");
        Font::try_from_bytes(font_data as &[u8]).expect("Error constructing Font")
    })
}
