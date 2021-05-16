// This file contains all code related to saving and loading JSON into the ECS.
// For the most part this code is taken from the tutorial at
// http://bfnightly.bracketproductions.com/rustbook/chapter_11.html
// It might be good in the future to look into making a custom impl for SerializeComponents
// to replace the custom macros
use crate::components::{
    AreaOfEffect, Armable, BlocksTile, Blood, CausesDamage, CausesFire, CausesLight, CombatStats,
    Consumable, Container, DamageHistory, Disarmable, Door, Dousable, EntityMoved, EntryTrigger,
    Equipable, Equipment, Flammable, Furniture, Grabbable, Grabbing, Hidden, Hiding, HidingSpot,
    Info, Inventory, Item, Lightable, Memory, Monster, Name, Objective, OnFire, Paralyze,
    ParticleLifetime, Player, Position, ProvidesHealing, Ranged, Renderable, Saveable,
    SerializationHelper, SingleActivation, SufferDamage, Trap, Triggered, Viewshed,
};
use crate::dungeon::dungeon::Dungeon;
use specs::{
    error::NoError,
    join::Join,
    saveload::{
        DeserializeComponents, MarkedBuilder, SerializeComponents, SimpleMarker,
        SimpleMarkerAllocator,
    },
    world::Builder,
    Entity, World, WorldExt,
};
use std::io::Write;

macro_rules! serialize_individually {
  ($world:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
      $(
      SerializeComponents::<NoError, SimpleMarker<Saveable>>::serialize(
          &( $world.read_storage::<$type>(), ),
          &$data.0,
          &$data.1,
          &mut $ser,
      )
      .unwrap();
      )*
  };
}

macro_rules! deserialize_individually {
  ($world:expr, $de:expr, $data:expr, $( $type:ty),*) => {
      $(
      DeserializeComponents::<NoError, _>::deserialize(
          &mut ( &mut $world.write_storage::<$type>(), ),
          &mut $data.0,
          &mut $data.1,
          &mut $data.2,
          &mut $de,
      )
      .unwrap();
      )*
  };
}

fn create_save_game_helpers(world: &mut World) {
    let dungeon_copy = world.get_mut::<Dungeon>().unwrap().clone();
    world
        .create_entity()
        .with(SerializationHelper {
            dungeon: dungeon_copy,
        })
        .marked::<SimpleMarker<Saveable>>()
        .build();
}

fn delete_helpers(world: &mut World) {
    let helper_ents: Vec<Entity> = {
        let helpers = world.read_storage::<SerializationHelper>();
        let entities = world.entities();
        (&entities, &helpers).join().map(|(e, _h)| e).collect()
    };
    world
        .delete_entities(helper_ents.as_slice())
        .expect("Delete Helpers Failed");
}

pub fn save_game_with_writer<T: Write>(world: &mut World, writer: T) -> serde_json::Serializer<T> {
    create_save_game_helpers(world);

    let mut serializer = serde_json::Serializer::new(writer);
    {
        let ent_markers = (
            world.entities(),
            world.read_storage::<SimpleMarker<Saveable>>(),
        );
        serialize_individually!(
            world,
            serializer,
            ent_markers,
            Position,
            Renderable,
            Player,
            Viewshed,
            Monster,
            Name,
            BlocksTile,
            CombatStats,
            SufferDamage,
            Item,
            Consumable,
            Ranged,
            AreaOfEffect,
            Paralyze,
            ProvidesHealing,
            Blood,
            ParticleLifetime,
            Hidden,
            EntryTrigger,
            EntityMoved,
            SingleActivation,
            Triggered,
            Objective,
            Container,
            Flammable,
            OnFire,
            CausesFire,
            Trap,
            Grabbable,
            Grabbing,
            Furniture,
            HidingSpot,
            Hiding,
            Memory,
            Equipment,
            Equipable,
            CausesDamage,
            CausesLight,
            Info,
            Lightable,
            Dousable,
            Armable,
            Disarmable,
            DamageHistory,
            Inventory,
            Door,
            SerializationHelper
        );
    }
    delete_helpers(world);
    serializer
}

fn deserialize_from_string(world: &mut World, game_string: String) {
    let mut deserializer = serde_json::Deserializer::from_str(&game_string);
    let mut ent_markers = (
        &mut world.entities(),
        &mut world.write_storage::<SimpleMarker<Saveable>>(),
        &mut world.write_resource::<SimpleMarkerAllocator<Saveable>>(),
    );

    deserialize_individually!(
        world,
        deserializer,
        ent_markers,
        Position,
        Renderable,
        Player,
        Viewshed,
        Monster,
        Name,
        BlocksTile,
        CombatStats,
        SufferDamage,
        Item,
        Consumable,
        Ranged,
        AreaOfEffect,
        Paralyze,
        ProvidesHealing,
        Blood,
        ParticleLifetime,
        Hidden,
        EntryTrigger,
        EntityMoved,
        SingleActivation,
        Triggered,
        Objective,
        Container,
        Flammable,
        OnFire,
        CausesFire,
        Trap,
        Grabbable,
        Grabbing,
        Furniture,
        HidingSpot,
        Hiding,
        Memory,
        Equipment,
        Equipable,
        CausesDamage,
        CausesLight,
        Info,
        Lightable,
        Dousable,
        Armable,
        Disarmable,
        DamageHistory,
        Inventory,
        Door,
        SerializationHelper
    );
}

fn get_dungeon(world: &mut World) -> Dungeon {
    let serialization_helpers = world.read_storage::<SerializationHelper>();
    let mut dungeons: Vec<Dungeon> = (serialization_helpers)
        .join()
        .map(|h| {
            let mut cloned_dungeon = h.dungeon.clone();
            for mut level in cloned_dungeon.levels.iter_mut() {
                let map_count = level.width * level.height;
                level.tile_content = (0..map_count).map(|_| vec![]).collect();
            }
            cloned_dungeon
        })
        .collect();
    dungeons.remove(0)
}

fn populate_map_from_helper(world: &mut World) {
    let dungeon = get_dungeon(world);
    world.insert(dungeon);
}

fn get_player_parts(world: &mut World) -> Entity {
    let entities = world.entities();
    let player = world.read_storage::<Player>();
    let parts: Vec<(Entity, &Player)> = (&entities, &player).join().collect();
    let player_part = parts.get(0).unwrap();
    player_part.0
}

fn populate_player(world: &mut World) {
    let player_ent = get_player_parts(world);
    world.insert(player_ent);
}

pub fn load_game_from_string(world: &mut World, game_string: String) {
    world.delete_all();
    deserialize_from_string(world, game_string);
    populate_map_from_helper(world);
    delete_helpers(world);
    populate_player(world);
}
