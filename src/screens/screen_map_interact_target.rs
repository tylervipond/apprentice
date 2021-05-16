use super::utils::{get_render_data, get_render_offset};
use super::{
    ui::{
        ui_map::UIMap,
        ui_tooltip::{UIToolTip, UIToolTipPosition},
    },
    SCREEN_WIDTH,
};
use crate::components::{Name, Position, Viewshed};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use crate::ui_components::{Style, UITextLine};
use rltk::{Rltk, BLACK, YELLOW};
use specs::{Entity, World, WorldExt};

pub struct ScreenMapInteractTarget<'a> {
    target: Option<&'a Entity>,
    cta: Option<&'a str>,
}

impl<'a> ScreenMapInteractTarget<'a> {
    pub fn new(target: Option<&'a Entity>, cta: Option<&'a str>) -> Self {
        Self { target, cta }
    }
    pub fn draw(&self, ctx: &mut Rltk, world: &mut World) {
        ctx.cls();
        let target = match self.target {
            Some(e) => *e,
            None => *world.fetch::<Entity>(),
        };
        let positions = world.read_storage::<Position>();
        let target_position = positions.get(target).unwrap();
        let dungeon = world.fetch::<Dungeon>();
        let level = dungeon.get_level(target_position.level).unwrap();
        let render_data = get_render_data(world);
        let (center_x, center_y) = level_utils::idx_xy(level.width as u32, target_position.idx);
        let render_offset = get_render_offset(center_x, center_y);
        let player_ent = world.fetch::<Entity>();
        let viewsheds = world.read_storage::<Viewshed>();
        let player_viewshed = viewsheds.get(*player_ent).unwrap();
        UIMap::new(
            level,
            &render_data,
            render_offset,
            &player_viewshed.visible_tiles,
        )
        .draw(ctx);
        UITextLine::new(
            1,
            0,
            self.cta
                .unwrap_or("press escape to exit, left/right to cycle targets"),
            Some(Style {
                fg: YELLOW,
                bg: BLACK,
            }),
        )
        .draw(ctx);
        let focus_x = center_x as i32 - render_offset.0;
        let focus_y = center_y as i32 - render_offset.1;
        ctx.set_bg(focus_x, focus_y, YELLOW);
        let tool_tip_pos = match focus_x > (SCREEN_WIDTH / 2) as i32 {
            true => UIToolTipPosition::Left,
            false => UIToolTipPosition::Right,
        };
        let names = world.read_storage::<Name>();
        let target_name = names.get(target).unwrap();
        let tool_tip_lines: Box<[&str]> = Box::new([target_name.name.as_str()]);
        UIToolTip::new(focus_x, focus_y, tool_tip_pos, &tool_tip_lines).draw(ctx);
    }
}
