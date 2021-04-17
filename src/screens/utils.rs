use super::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
    ui::ui_map::RenderData,
};
use crate::components::{Hidden, Hiding, OnFire, Position, Renderable, Viewshed};
use rltk::{GREY, ORANGE, RGB};
use specs::{Entity, Join, World, WorldExt};

pub fn get_render_data(world: &World) -> Vec<RenderData> {
    let positions = world.read_storage::<Position>();
    let hidden = world.read_storage::<Hidden>();
    let renderables = world.read_storage::<Renderable>();
    let on_fire = world.read_storage::<OnFire>();
    let hiding = world.read_storage::<Hiding>();
    let viewsheds = world.read_storage::<Viewshed>();
    let entities = world.entities();
    let player_ent = world.fetch::<Entity>();
    let player_level = positions.get(*player_ent).unwrap().level;
    let player_viewshed = viewsheds.get(*player_ent).unwrap();
    let mut render_data: Vec<RenderData> = (
        &positions,
        &renderables,
        (&on_fire).maybe(),
        (&hidden).maybe(),
        (&hiding).maybe(),
        &entities,
    )
        .join()
        .filter(|(p, _r, _f, h, hiding, entity)| {
            let is_visible = match h {
                Some(h) => h.found_by.contains(&*player_ent),
                None => true,
            };
            let hiding = match hiding {
                Some(_) => *entity != *player_ent,
                _ => false,
            };
            return p.level == player_level && is_visible && !hiding
                && player_viewshed.visible_tiles.contains(&p.idx);
        })
        .map(|(p, r, f, _h, hiding, entity)| {
            let mut fg = if hiding.is_some() && entity == *player_ent {
                RGB::named(GREY)
            } else if f.is_some() {
                RGB::named(ORANGE)
            } else {
                r.fg
            };
            RenderData {
                idx: p.idx,
                fg,
                bg: r.bg,
                glyph: r.glyph,
                layer: r.layer,
            }
        })
        .collect();
    render_data.sort_unstable_by(|a, b| b.layer.cmp(&a.layer));
    return render_data;
}

pub fn get_render_offset(center_x: i32, center_y: i32) -> (i32, i32) {
    let offset_x = center_x - MAP_WIDTH as i32 / 2;
    let offset_y = center_y - MAP_HEIGHT as i32 / 2;
    (offset_x, offset_y)
}

pub fn get_render_offset_for_xy(center_x: i32, center_y: i32, x: i32, y: i32) -> (i32, i32) {
    let (center_offset_x, center_offset_y) = get_render_offset(center_x, center_y);
    let offset_x = x + center_offset_x;
    let offset_y = y + center_offset_y;
    (offset_x, offset_y)
}
