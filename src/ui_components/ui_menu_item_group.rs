use super::{style::Style, utils, UITextLine};
use crate::menu::{MenuOption, MenuOptionState};
use rltk::{Rltk, BLACK, GREY, WHITE, YELLOW1, YELLOW4};

pub struct UIMenuItemGroup<'a, 'b, T: ToString> {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    menu_options: &'b Box<[&'a MenuOption<T>]>,
    active: bool,
}

impl<'a, 'b, T: ToString> UIMenuItemGroup<'a, 'b, T> {
    pub fn new(x: i32, y: i32, menu_options: &'b Box<[&'a MenuOption<T>]>, active: bool) -> Self {
        let lines: Box<[String]> = menu_options.iter().map(|o| o.text.to_string()).collect();
        let width = utils::get_longest_line_length(&lines) as u32;
        let height = menu_options.len() as u32;
        Self {
            x,
            y,
            height,
            width,
            menu_options,
            active,
        }
    }
    pub fn draw(&self, ctx: &mut Rltk) {
        for (i, menu_option) in self.menu_options.iter().enumerate() {
            let this_text_y = self.y + i as i32;
            let fg = match menu_option.state {
                MenuOptionState::Highlighted => match self.active {
                    true => YELLOW1,
                    false => YELLOW4,
                },
                MenuOptionState::Normal => WHITE,
                MenuOptionState::Disabled => GREY,
            };
            let bg = match menu_option.state {
                MenuOptionState::Highlighted => (30, 30, 30),
                _ => BLACK,
            };
            let style = Style { fg, bg };
            UITextLine::new(self.x, this_text_y, menu_option.text.to_string(), Some(style)).draw(ctx);
        }
    }
}
