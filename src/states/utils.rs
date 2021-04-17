use crate::screens::{
    constants::{MAP_HEIGHT, MAP_WIDTH},
};

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