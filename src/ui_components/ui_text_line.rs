use super::Style;
use rltk::{Rltk, RGB};

pub struct UITextLine<T: ToString> {
    x: i32,
    y: i32,
    text: T,
    style: Style,
}

impl<T: ToString> UITextLine<T> {
    pub fn new(x: i32, y: i32, text: T, style: Option<Style>) -> Self {
        let style = match style {
            Some(s) => s,
            None => Style {
                fg: (255, 255, 255),
                bg: (0, 0, 0),
            },
        };
        Self { x, y, text, style }
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        ctx.print_color(
            self.x,
            self.y,
            RGB::named(self.style.fg),
            RGB::named(self.style.bg),
            self.text.to_string(),
        );
    }
}
