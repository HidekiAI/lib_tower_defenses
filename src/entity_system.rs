pub use super::sprite_system::*;
use once_cell::sync::Lazy;

use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;

// Note: No need to drop/deconstruct/destroy once it's created
static ENTITY_SINGLETON: Lazy<Mutex<EntityFactory>> =
    Lazy::new(|| Mutex::new(EntityFactory::new()));
struct EntityFactory {
    entities: Vec<Entity>,
}

impl EntityFactory {
    fn new() -> EntityFactory {
        // Initialize your data here
        EntityFactory {
            entities: Vec::new(),
        }
    }
}

pub type TEntityID = u16;

pub fn add_entity(sprite_id: TSpriteID, layer_weight: u8) -> Result<TEntityID, String> {
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
    };
    singleton.entities.push(new_entity);

    return Ok(new_entity.id);
}
pub fn del_entity(entity_id: TEntityID) -> Result<TSpriteID, String> {
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
pub fn update_entities() {
    let mut _singleton = ENTITY_SINGLETON.lock().unwrap();
}
pub fn reset_entities() {
    let mut singleton = ENTITY_SINGLETON.lock().unwrap();
    singleton.entities.clear();
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Entity {
    id: TEntityID,
    pub sprite_id: TSpriteID,
    pub layer_weight: u8, // lighter the weight, it bubbles towards the top when stacked (255 means most heaviest, 0 is lightest)
}
impl Entity {
    fn _new() -> Entity {
        // private, must call add_entity instead!
        Entity {
            id: 0,              // default unused entity
            sprite_id: 0,       // default sprite
            layer_weight: 0x80, // mid-weight
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
        let new_entity = add_entity(sprite_id, layer_weight).unwrap();
        match del_entity(new_entity) {
            Ok(_) => (),
            Err(serr) => panic!("{}", serr),
        }
    }
}
