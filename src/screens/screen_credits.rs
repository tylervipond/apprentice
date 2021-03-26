use super::constants::{SCREEN_PADDING, SCREEN_WIDTH};
use crate::ui_components::ui_text_line_centered::UITextLineCentered;
use rltk::{Rltk, BLACK, WHITE};

const CREDIT_LINE_WIDTH: u32 = (SCREEN_WIDTH - SCREEN_PADDING * 2) as u32;

fn create_credit_line(y: i32, text: &str) -> UITextLineCentered {
    UITextLineCentered::new(
        SCREEN_PADDING as i32,
        y,
        CREDIT_LINE_WIDTH,
        WHITE,
        BLACK,
        text,
    )
}

pub struct ScreenCredits {}

impl ScreenCredits {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        ctx.cls();
        create_credit_line(5, "Artwork By").draw(ctx);
        create_credit_line(6, "Cameron Stott").draw(ctx);
        create_credit_line(8, "Music By").draw(ctx);
        create_credit_line(9, "Cameron Stott").draw(ctx);
        create_credit_line(11, "Code By").draw(ctx);
        create_credit_line(12, "Alex Eagleson").draw(ctx);
        create_credit_line(13, "Tyler Vipond").draw(ctx);
        create_credit_line(15, "Created By").draw(ctx);
        create_credit_line(16, "Tyler Vipond").draw(ctx);
    }
}
