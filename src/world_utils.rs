use crate::{
    components::{
        AreaOfEffect, Armable, BlocksTile, Blood, CausesDamage, CausesFire, CausesLight,
        CombatStats, Confused, Confusion, Consumable, Container, DamageHistory, Disarmable, Door,
        Dousable, EntityMoved, EntryTrigger, Equipable, Equipment, Flammable, Furniture, Grabbable,
        Grabbing, Hidden, Hiding, HidingSpot, Info, Inventory, Item, Lightable, Memory, Monster,
        Name, Objective, OnFire, ParticleLifetime, Player, Position, Potion, ProvidesHealing,
        Ranged, Renderable, Saveable, SerializationHelper, SingleActivation, SufferDamage, Trap,
        Triggered, Viewshed, WantsToCloseDoor, WantsToDisarmTrap, WantsToDouse, WantsToDropItem,
        WantsToEquip, WantsToExit, WantsToGoDownStairs, WantsToGrab, WantsToHide, WantsToLight,
        WantsToMelee, WantsToMove, WantsToOpenDoor, WantsToPickUpItem, WantsToReleaseGrabbed,
        WantsToSearchHidden, WantsToTrap, WantsToUse,WantsToGoUpStairs
    },
    services::{
        BloodSpawner, CorpseSpawner, DebrisSpawner, GameLog, ItemSpawner, ParticleEffectSpawner,
        TrapSpawner,
    },
};
use rltk::RandomNumberGenerator;
use specs::{
    saveload::{SimpleMarker, SimpleMarkerAllocator},
    World, WorldExt,
};

pub fn initialize_new_game(world: &mut World) {
    world.write_storage::<Position>().clear();
    world.write_storage::<Renderable>().clear();
    world.write_storage::<Player>().clear();
    world.write_storage::<Viewshed>().clear();
    world.write_storage::<Monster>().clear();
    world.write_storage::<Name>().clear();
    world.write_storage::<BlocksTile>().clear();
    world.write_storage::<SufferDamage>().clear();
    world.write_storage::<CombatStats>().clear();
    world.write_storage::<Item>().clear();
    world.write_storage::<Potion>().clear();
    world.write_storage::<ProvidesHealing>().clear();
    world.write_storage::<Consumable>().clear();
    world.write_storage::<Ranged>().clear();
    world.write_storage::<AreaOfEffect>().clear();
    world.write_storage::<Confusion>().clear();
    world.write_storage::<Confused>().clear();
    world.write_storage::<SimpleMarker<Saveable>>().clear();
    world.write_storage::<SerializationHelper>().clear();
    world.write_storage::<Blood>().clear();
    world.write_storage::<ParticleLifetime>().clear();
    world.write_storage::<Hidden>().clear();
    world.write_storage::<EntryTrigger>().clear();
    world.write_storage::<EntityMoved>().clear();
    world.write_storage::<SingleActivation>().clear();
    world.write_storage::<Triggered>().clear();
    world.write_storage::<Objective>().clear();
    world.write_storage::<Container>().clear();
    world.write_storage::<Flammable>().clear();
    world.write_storage::<OnFire>().clear();
    world.write_storage::<CausesFire>().clear();
    world.write_storage::<Trap>().clear();
    world.write_storage::<Grabbable>().clear();
    world.write_storage::<Grabbing>().clear();
    world.write_storage::<Memory>().clear();
    world.write_storage::<Furniture>().clear();
    world.write_storage::<Hiding>().clear();
    world.write_storage::<HidingSpot>().clear();
    world.write_storage::<Equipment>().clear();
    world.write_storage::<Equipable>().clear();
    world.write_storage::<CausesDamage>().clear();
    world.write_storage::<CausesLight>().clear();
    world.write_storage::<Info>().clear();
    world.write_storage::<Lightable>().clear();
    world.write_storage::<Dousable>().clear();
    world.write_storage::<Armable>().clear();
    world.write_storage::<Disarmable>().clear();
    world.write_storage::<DamageHistory>().clear();
    world.write_storage::<Inventory>().clear();
    world.remove::<SimpleMarkerAllocator<Saveable>>();
    world.insert(SimpleMarkerAllocator::<Saveable>::new());
    world.remove::<GameLog>();
    world.insert(GameLog {
        entries: vec!["Enter the dungeon apprentice! Bring back the Talisman!".to_owned()],
    });
}

pub fn get_world() -> World {
    let mut world = World::new();
    world.register::<Memory>();
    world.register::<Position>();
    world.register::<Renderable>();
    world.register::<Player>();
    world.register::<Viewshed>();
    world.register::<Monster>();
    world.register::<Name>();
    world.register::<BlocksTile>();
    world.register::<WantsToMelee>();
    world.register::<SufferDamage>();
    world.register::<CombatStats>();
    world.register::<Item>();
    world.register::<Potion>();
    world.register::<WantsToPickUpItem>();
    world.register::<WantsToUse>();
    world.register::<WantsToDropItem>();
    world.register::<ProvidesHealing>();
    world.register::<Consumable>();
    world.register::<Ranged>();
    world.register::<AreaOfEffect>();
    world.register::<Confusion>();
    world.register::<Confused>();
    world.register::<SimpleMarker<Saveable>>();
    world.register::<SerializationHelper>();
    world.register::<Blood>();
    world.register::<ParticleLifetime>();
    world.register::<Hidden>();
    world.register::<EntryTrigger>();
    world.register::<EntityMoved>();
    world.register::<SingleActivation>();
    world.register::<Triggered>();
    world.register::<Objective>();
    world.register::<Container>();
    world.register::<Flammable>();
    world.register::<OnFire>();
    world.register::<CausesFire>();
    world.register::<WantsToSearchHidden>();
    world.register::<Trap>();
    world.register::<WantsToTrap>();
    world.register::<WantsToDisarmTrap>();
    world.register::<WantsToGrab>();
    world.register::<Grabbable>();
    world.register::<Grabbing>();
    world.register::<WantsToMove>();
    world.register::<WantsToReleaseGrabbed>();
    world.register::<WantsToOpenDoor>();
    world.register::<WantsToCloseDoor>();
    world.register::<WantsToGoDownStairs>();
    world.register::<WantsToGoUpStairs>();
    world.register::<WantsToExit>();
    world.register::<Door>();
    world.register::<Furniture>();
    world.register::<HidingSpot>();
    world.register::<Hiding>();
    world.register::<WantsToHide>();
    world.register::<Equipment>();
    world.register::<Equipable>();
    world.register::<WantsToEquip>();
    world.register::<CausesDamage>();
    world.register::<CausesLight>();
    world.register::<Info>();
    world.register::<Lightable>();
    world.register::<Dousable>();
    world.register::<WantsToDouse>();
    world.register::<WantsToLight>();
    world.register::<Disarmable>();
    world.register::<Armable>();
    world.register::<DamageHistory>();
    world.register::<Inventory>();
    world.insert(SimpleMarkerAllocator::<Saveable>::new());
    world.insert(GameLog {
        entries: vec!["Enter the dungeon apprentice! Bring back the Talisman!".to_owned()],
    }); // This needs to get moved to a continue game function I think...
    world.insert(RandomNumberGenerator::new());
    world.insert(ParticleEffectSpawner::new());
    world.insert(BloodSpawner::new());
    world.insert(DebrisSpawner::new());
    world.insert(TrapSpawner::new());
    world.insert(ItemSpawner::new());
    world.insert(CorpseSpawner::new());
    world
}
