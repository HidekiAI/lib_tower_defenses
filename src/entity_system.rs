use crate::sprite_system;

pub use super::sprite_system::*;
use once_cell::sync::Lazy;

use serde_derive::{Deserialize, Serialize};
use std::{sync::Mutex, time::Instant};

// Note: No need to drop/deconstruct/destroy once it's created
static ENTITY_SINGLETON: Lazy<Mutex<EntityFactory>> =
    Lazy::new(|| Mutex::new(EntityFactory::new()));
struct EntityFactory {
    entities: Vec<Entity>,
    next_entity_to_update: usize, // based on time-slices, may not have been able to update entire list, so we track where we've left off and continue on from here
}

impl EntityFactory {
    fn new() -> EntityFactory {
        // Initialize your data here
        EntityFactory {
            entities: Vec::new(),
            next_entity_to_update: 0, // start at index=0 (edge-case: if entities.len() == 0)
        }
    }
}

pub type TEntityID = u16;

pub fn add(sprite_id: TSpriteID, layer_weight: u8) -> Result<TEntityID, String> {
    let mut singleton = ENTITY_SINGLETON.lock().unwrap();

    // NOTE: because we always get +1 of last max EntityID, there will be (hopefully rare)
    // chance that it may wrap around after Max==65535 entities.  Hoopefully, during each
    // loading/reloading, we can re-pack and/or flush Entity list
    let max_id = match singleton.entities.iter().max_by_key(|e| e.id) {
        Some(found) => found.id,
        _ => 0, // edge-case, list is empty
    };

    let new_entity = Entity {
        id: max_id + 1,
        sprite_id: sprite_id,
        layer_weight: layer_weight,
        current_sprite_index: 0,
        sprite_update_interval_reset: 24 * 1000,
        last_sprite_update_millis: 0,
        health_points: 0,
        mana_points: 0,
        physics_info: PhysicsObject::new(),
    };
    singleton.entities.push(new_entity);

    return Ok(new_entity.id);
}
pub fn remove(entity_id: TEntityID) -> Result<TSpriteID, String> {
    let mut singleton = ENTITY_SINGLETON.lock().unwrap();

    let found_index = singleton
        .entities
        .binary_search_by(|entity| entity.id.cmp(&entity_id));
    match found_index {
        Ok(i) => {
            let x = singleton.entities.remove(i);
            return Ok(x.sprite_id);
        }
        Err(e) => Err(format!("spriteID={} already delted - {}", entity_id, e)), // do nothing if already deleted...
    }
}
// there is no get(), for it can potentially cause deadlocks based on temptations
// to be used at critical sections; hence it is intentionally exposing try_get
// that can (and will) return None immediately rather than blocking
pub fn try_get(entity_id: TEntityID) -> Option<Entity> {
    match ENTITY_SINGLETON.try_lock() {
        Ok(singleton) => match singleton
            .entities
            .binary_search_by(|entity| entity.id.cmp(&entity_id))
        {
            Ok(entity_index) => match singleton.entities.get(entity_index) {
                Some(ent) => Some(ent.clone()),
                _ => None,
            },
            Err(_) => None,
        },
        Err(_) => None,
    }
}
// See: Instant::now() and Instant::elapsed() for more details on how to pass deltaT
// if max time slice is 0, will process entire list
pub fn update(last_frame_delta_millis: u128, max_time_slice: u128) {
    let start_time_now = Instant::now();
    let mut singleton = ENTITY_SINGLETON.lock().unwrap();
    let is_collidable = |entity: Entity| -> bool {
        entity
            .physics_info
            .collision_type
            .eq(&PhysicsObjectCollisionTypes::NotCollidable)
            == false
    };

    let mut exit_update = false;
    let mut processed_entity_count = 0;
    loop {
        let entity_index = singleton.next_entity_to_update;
        // TODO: update each entity
        format!("Updating: {:?}:", singleton.entities[entity_index]);

        singleton.entities[entity_index].update(last_frame_delta_millis);

        if is_collidable(singleton.entities[entity_index]) {
            // test collisions against others (exclude self)
            let current_entity = singleton.entities[entity_index];
            // note, even though we may only partially process the list, we still will
            // test collsion against ALL collidable objects
            for other in singleton.entities.clone() {
                if other.id != current_entity.id && is_collidable(other) {
                    // do collision test
                    //physics_system::test_collision(current_entity.id, other.id);
                }
            }
        }

        // check time-slice
        processed_entity_count += 1;
        singleton.next_entity_to_update += 1;
        if singleton.next_entity_to_update >= singleton.entities.len() {
            // need to
            singleton.next_entity_to_update = 0;
        }
        if max_time_slice > 0 {
            if start_time_now.elapsed().as_millis() >= max_time_slice {
                // bail out if we've exceeded allowed time slice
                exit_update = true;
                break;
            }
        }
        if processed_entity_count >= singleton.entities.len() {
            exit_update = true;
        }

        if exit_update {
            break;
        }
    }
}

/// ultimate method of garbage collection...
pub fn reset() {
    let mut singleton = ENTITY_SINGLETON.lock().unwrap();
    singleton.entities.clear();
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum PhysicsObjectCollisionTypes {
    NotCollidable, // i.e. smoke, vapor, etc
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct PhysicsObject {
    pub collision_type: PhysicsObjectCollisionTypes,
    pub max_velocity: u8, // (absolute value) number of grids per second (currently maxing to 255 grids per second, pretty darn fast)
    pub max_acceleration: u8, // allows fake effect of rubberband on flying objects without mass (F=ma => a=F/m)
    pub current_velocity_x: i16,
    pub current_velocity_y: i16,
    pub current_acceleration_x: i8,
    pub current_acceleration_y: i8,
}
impl PhysicsObject {
    fn new() -> PhysicsObject {
        return PhysicsObject {
            collision_type: PhysicsObjectCollisionTypes::NotCollidable,
            max_velocity: 0,
            max_acceleration: 0,
            current_velocity_x: 0,
            current_velocity_y: 0,
            current_acceleration_x: 0,
            current_acceleration_y: 0,
        };
    }
}
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Entity {
    pub id: TEntityID,
    pub sprite_id: TSpriteID, // no need to track ResourceID for this SpriteID, since Sprite system internally tracks resources associated to it
    pub layer_weight: u8, // lighter the weight, it bubbles towards the top when stacked (255 means most heaviest, 0 is lightest)
    pub current_sprite_index: u8, // Assumes there are no more than 255 sprites in sprite groups
    pub sprite_update_interval_reset: u128, // reset timer value
    pub last_sprite_update_millis: u128, // duration as_millis() returns u128
    pub health_points: u16, // max of 65535 HP
    pub mana_points: u16, // max of 65535 MP
    pub physics_info: PhysicsObject,
}
impl Entity {
    fn new(id: TEntityID, sid: TSpriteID, weight: u8) -> Entity {
        // private, must call add_entity instead!
        Entity {
            id: id,
            sprite_id: sid,
            layer_weight: weight, // mid-weight is 0x80
            current_sprite_index: 0,
            sprite_update_interval_reset: 60 * 1000,
            last_sprite_update_millis: 0,
            health_points: 0,
            mana_points: 0,
            physics_info: PhysicsObject::new(),
        }
    }
    pub fn update(self: &mut Self, last_frame_delta_millis: u128) {
        // make sure to update with elapsed time (animation)
        let time_left = self.last_sprite_update_millis as i128 - last_frame_delta_millis as i128;
        if time_left > 0 {
            self.last_sprite_update_millis = time_left as u128;
        } else {
            // time to update frame and reset clock
            sprite_system::add_sprite_for_update(self.sprite_id);
            self.last_sprite_update_millis = self.sprite_update_interval_reset;
        }
        if self.physics_info.current_velocity_x > 0
            || self.physics_info.current_velocity_y > 0
            || self.physics_info.current_acceleration_x > 0
            || self.physics_info.current_acceleration_y > 0
        {
            // update position if moving
        }
    }
}

#[cfg(test)]
mod tests {
    //use sdl2::sys::__pthread_internal_list;

    //use crate::sample_lib::add;

    //use core::slice::SlicePattern;
    use super::*;
    //use std::io::{BufReader, BufWriter, Read, Write};
    //use serde_test::{assert_tokens, Token};

    #[test]
    fn test_create_and_remove() {
        let sprite_id = 5;
        let layer_weight = 12;
        let new_entity = add(sprite_id, layer_weight).unwrap();
        match remove(new_entity) {
            Ok(_) => (),
            Err(serr) => panic!("{}", serr),
        }
    }
}
