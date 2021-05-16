use std::collections::HashSet;

use super::ui_map::{RenderData, UIMap};
use super::{
    ui_hud::UIHud,
    ui_mouse_pos::UIMousePos,
    ui_tooltip::{UIToolTip, UIToolTipPosition},
};
use crate::dungeon::level::Level;
use crate::screens::constants::SCREEN_WIDTH;
use rltk::Rltk;

pub struct UIMapScreen<'a, 'b> {
    mouse_x: i32,
    mouse_y: i32,
    tool_tip_lines: &'b Box<[&'a str]>,
    messages: &'b Box<[&'a str]>,
    depth: usize,
    hp: i32,
    max_hp: i32,
    level: &'a Level,
    renderables: &'a Vec<RenderData>,
    render_offset: (i32, i32),
    visible_tiles: &'a HashSet<usize>
}

impl<'a, 'b> UIMapScreen<'a, 'b> {
    pub fn new(
        mouse_x: i32,
        mouse_y: i32,
        tool_tip_lines: &'b Box<[&'a str]>,
        messages: &'b Box<[&'a str]>,
        depth: usize,
        hp: i32,
        max_hp: i32,
        level: &'a Level,
        renderables: &'a Vec<RenderData>,
        render_offset: (i32, i32),
        visible_tiles: &'a HashSet<usize>
    ) -> Self {
        Self {
            mouse_x,
            mouse_y,
            tool_tip_lines,
            messages,
            depth,
            hp,
            max_hp,
            level,
            renderables,
            render_offset,
            visible_tiles
        }
    }

    pub fn draw(&self, ctx: &mut Rltk) {
        UIMap::new(self.level, self.renderables, self.render_offset, self.visible_tiles).draw(ctx);
        UIHud::new(self.depth, self.hp, self.max_hp, self.messages).draw(ctx);
        if !self.tool_tip_lines.is_empty() {
            let tool_tip_pos = match self.mouse_x > (SCREEN_WIDTH / 2) as i32 {
                true => UIToolTipPosition::Left,
                false => UIToolTipPosition::Right,
            };
            UIToolTip::new(
                self.mouse_x,
                self.mouse_y,
                tool_tip_pos,
                &self.tool_tip_lines,
            )
            .draw(ctx);
        }
        UIMousePos::new(self.mouse_x, self.mouse_y).draw(ctx);
    }
}
