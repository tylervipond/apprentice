use crate::artwork::VICTORY_ARTWORK;
use crate::screens::constants::{SCREEN_PADDING, SCREEN_WIDTH};
use crate::ui_components::ui_paragraph::UIParagraph;
use crate::ui_components::UITextLine;
use rltk::Rltk;

pub struct ScreenSuccess {}

impl ScreenSuccess {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        ctx.cls();
        VICTORY_ARTWORK
            .lines()
            .enumerate()
            .for_each(|(idx, line)| UITextLine::new(0, idx as i32 + 1, line, None).draw(ctx));
        let text = "\"Finally, the talisman is within my grasp, you have passed your final challenge as my apprentice. What magnificent adventures await you\"";
        UIParagraph::new(
            SCREEN_PADDING as i32,
            55,
            (SCREEN_WIDTH - SCREEN_PADDING * 2) as u32,
            text,
        )
        .draw(ctx);
    }
}
