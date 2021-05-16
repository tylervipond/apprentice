use crate::screens::constants::{MAP_HEIGHT, MAP_WIDTH, SCREEN_HEIGHT};
use crate::ui_components::{Style, UIBox, UITextLine};
use rltk::{Rltk, BLACK, RED, RGB, WHITE, YELLOW};

const HUD_LEFT: i32 = 0;
const HUD_TOP: i32 = MAP_HEIGHT as i32;
const HUD_WIDTH: u8 = MAP_WIDTH - 1;
const HUD_HEIGHT: u8 = SCREEN_HEIGHT - MAP_HEIGHT - 1;
const HUD_HEALTH_LEFT: i32 = 12;
const HUD_HEALTH_BAR_LEFT: i32 = 28;
const HUD_HEALTH_BAR_WIDTH: i32 = HUD_WIDTH as i32 - HUD_HEALTH_BAR_LEFT;
const MESSAGES_TOP: i32 = HUD_TOP + 1;
const MESSAGES_LEFT: i32 = HUD_LEFT + 1;
const MESSAGE_COUNT: u8 = HUD_HEIGHT - 2;

pub struct UIHud<'a, 'b> {
    depth: usize,
    hp: i32,
    max_hp: i32,
    messages: &'b Box<[&'a str]>,
}

impl<'a, 'b> UIHud<'a, 'b> {
    pub fn new(depth: usize, hp: i32, max_hp: i32, messages: &'b Box<[&'a str]>) -> Self {
        Self {
            depth,
            hp,
            max_hp,
            messages,
        }
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        UIBox::new(HUD_LEFT, HUD_TOP, HUD_WIDTH, HUD_HEIGHT, WHITE, BLACK).draw(ctx);
        UITextLine::new(
            2,
            HUD_TOP,
            &format!("Depth: {}", self.depth),
            Some(Style {
                fg: YELLOW,
                bg: BLACK,
            }),
        )
        .draw(ctx);
        let health = format!("HP: {} / {}", self.hp, self.max_hp);
        UITextLine::new(
            HUD_HEALTH_LEFT,
            HUD_TOP,
            &health,
            Some(Style {
                fg: YELLOW,
                bg: BLACK,
            }),
        )
        .draw(ctx);
        ctx.draw_bar_horizontal(
            HUD_HEALTH_BAR_LEFT,
            HUD_TOP,
            HUD_HEALTH_BAR_WIDTH,
            self.hp,
            self.max_hp,
            RGB::named(RED),
            RGB::named(BLACK),
        );
        for (i, message) in self
            .messages
            .iter()
            .enumerate()
            .take(MESSAGE_COUNT as usize)
        {
            ctx.print(MESSAGES_LEFT, MESSAGES_TOP + i as i32, &message.to_string());
        }
    }
}
