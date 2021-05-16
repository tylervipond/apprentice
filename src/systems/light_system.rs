use crate::components::{CausesLight, Equipment, Position};
use crate::dungeon::{dungeon::Dungeon, level_utils};
use specs::{Entity, Join, ReadStorage, System, WriteExpect, WriteStorage};

fn get_light_radius_from_equipment(
    equipment: Entity,
    causes_light: &ReadStorage<CausesLight>,
) -> u32 {
    match causes_light.get(equipment) {
        Some(light) => match light.lit {
            true => light.radius as u32,
            false => 0,
        },
        None => 0,
    }
}

pub struct LightSystem {}

impl<'a> System<'a> for LightSystem {
    type SystemData = (
        WriteExpect<'a, Dungeon>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Equipment>,
        ReadStorage<'a, CausesLight>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (mut dungeon, position, equipment, causes_light) = data;
        dungeon
            .levels
            .iter_mut()
            .for_each(|level| level.lit_tiles.iter_mut().for_each(|t| *t = false));

        (&equipment, &position)
            .join()
            .map(|(equipment, position)| {
                let light_radius = {
                    let dominant_hand_light_range = match equipment.dominant_hand {
                        Some(item) => get_light_radius_from_equipment(item, &causes_light),
                        _ => 0,
                    };
                    let off_hand_light_range = match equipment.off_hand {
                        Some(item) => get_light_radius_from_equipment(item, &causes_light),
                        _ => 0,
                    };
                    i32::max(
                        dominant_hand_light_range as i32,
                        off_hand_light_range as i32,
                    )
                };
                (light_radius as i32, position.level, position.idx as i32)
            })
            .chain(
                (&position, &causes_light)
                    .join()
                    .filter(|(_, causes_light)| causes_light.lit)
                    .map(|(position, causes_light)| {
                        (
                            causes_light.radius as i32,
                            position.level,
                            position.idx as i32,
                        )
                    }),
            )
            .for_each(|(radius, level, pos_idx)| {
                let level = dungeon.get_level_mut(level).unwrap();
                let level_width = level.width as i32;
                let lit_points = (-radius..=radius)
                    .map(|y| {
                        (-radius..=radius)
                            .map(move |x| level_utils::add_xy_to_idx(level_width, x, y, pos_idx))
                    })
                    .flatten()
                    .filter(|idx| {
                        level_utils::get_distance_between_idxs(
                            level,
                            pos_idx as usize,
                            *idx as usize,
                        ) < radius as f32
                    })
                    .collect::<Vec<i32>>();
                for index in lit_points {
                    match level.lit_tiles.get_mut(index as usize) {
                        Some(i) => {
                            *i = true;
                        }
                        _ => {}
                    }
                }
            });
    }
}
