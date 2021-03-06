use crate::components::{
    Armable, Consumable, Item, Name, Position, Ranged, Renderable, Saveable, Trap,
};
use crate::services::ItemSpawner;
use crate::types::item_type;
use rltk::{BLACK, RGB};
use specs::{
    saveload::{MarkerAllocator, SimpleMarker, SimpleMarkerAllocator},
    Entities, System, WriteExpect, WriteStorage,
};

pub struct ItemSpawnSystem {}
impl<'a> System<'a> for ItemSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, Name>,
        WriteStorage<'a, Item>,
        WriteStorage<'a, Consumable>,
        WriteStorage<'a, Ranged>,
        WriteStorage<'a, Trap>,
        WriteStorage<'a, Armable>,
        WriteExpect<'a, ItemSpawner>,
        WriteExpect<'a, SimpleMarkerAllocator<Saveable>>,
        WriteStorage<'a, SimpleMarker<Saveable>>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut positions,
            mut renderables,
            mut names,
            mut items,
            mut consumables,
            mut ranged,
            mut traps,
            mut armables,
            mut spawner,
            mut marker_allocator,
            mut markers,
        ) = data;
        for request in spawner.requests.iter() {
            let new_item = entities.create();
            positions
                .insert(
                    new_item,
                    Position {
                        idx: request.idx,
                        level: request.level,
                    },
                )
                .expect("failed inserting position for new item");
            renderables
                .insert(
                    new_item,
                    Renderable {
                        fg: item_type::get_foreground_color_for_item(&request.item_type),
                        bg: RGB::from(BLACK),
                        glyph: item_type::get_glyph_for_item(&request.item_type),
                        layer: 2,
                    },
                )
                .expect("failed inserting renderable for new item");
            names
                .insert(
                    new_item,
                    Name {
                        name: item_type::get_name_for_item(&request.item_type),
                    },
                )
                .expect("failed inserting name for new item");
            items
                .insert(new_item, Item {})
                .expect("failed inserting item for new item");
            if item_type::item_is_consumable(&request.item_type) {
                consumables
                    .insert(new_item, Consumable {})
                    .expect("failed inserting consumable for new item");
            }

            if let Some(range) = item_type::get_range_for_item(&request.item_type) {
                ranged
                    .insert(new_item, Ranged { range })
                    .expect("failed inserting ranged for new item");
            }

            if let Some(trap_type) = item_type::get_trap_type_for_item(&request.item_type) {
                traps
                    .insert(new_item, Trap { trap_type })
                    .expect("failed inserting trap for new item");
                armables
                    .insert(new_item, Armable {})
                    .expect("failed inserting armable for new item");
            }
            marker_allocator.mark(new_item, &mut markers);
        }
        spawner.requests.clear();
    }
}
