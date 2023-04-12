use super::resource_system::*;
use once_cell::sync::Lazy;
//use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use std::sync::Mutex;

// Note: No need to drop/deconstruct/destroy once it's created
static SPRITE_SINGLETON: Lazy<Mutex<SpriteFactory>> =
    Lazy::new(|| Mutex::new(SpriteFactory::new()));
struct SpriteFactory {
    sprites: Vec<Sprite>,
    updates: Vec<TSpriteID>,
}

impl SpriteFactory {
    fn new() -> SpriteFactory {
        // Initialize your data here
        SpriteFactory {
            sprites: Vec::new(),
            updates: Vec::new(),
        }
    }
}

pub type TSpriteID = u16;

// there is no get(), for it can potentially cause deadlocks based on temptations
// to be used at critical sections; hence it is intentionally exposing try_get
// that can (and will) return None immediately rather than blocking
pub fn try_get(resource_id: &TResourceID) -> Option<Sprite> {
    match SPRITE_SINGLETON.try_lock() {
        Ok(singleton) => match singleton
            .sprites
            .binary_search_by(|sprite| sprite.resource_id.cmp(&resource_id))
        {
            Ok(index) => {
                return Some(singleton.sprites.get(index).unwrap().clone());
            }
            Err(_) => {
                return None;
            }
        },
        Err(_) => None,
    }
}
/// Adds sprites to singleton collection; note that there will often be more
/// than one sprite with same ResourceID, but will have different SpriteID
/// assigned to it.  This is because same resource is not assumed to synchronize
/// on the animations.  I.e. one entity may be in the walking sprite sequence
/// while another entity, with same sprite group id is firing a projectile
pub fn add(resource_id: &TResourceID) -> Result<TSpriteID, String> {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();

    // NOTE: because we always get +1 of last max spriteID, there will be (hopefully rare)
    // chance that it may wrap around after Max==65535 sprites.  Hoopefully, during each
    // loading/reloading, we can re-pack and/or flush sprite list
    let max_id = match singleton.sprites.iter().max_by_key(|e| e.id) {
        Some(s) => s.id,
        _ => 0 as TSpriteID, // edge-case when list is empty
    };
    let found_sprite = try_get(&resource_id);

    // if resource_id already found/exists, return the previously added spriteID instead, else add and return newly created one
    match found_sprite {
        Some(spr) => {
            return Ok(spr.id);
        }
        None => {
            let new_sprite = Sprite {
                id: max_id,
                resource_id: *resource_id,
            };
            singleton.sprites.push(new_sprite);
            return Ok(new_sprite.id);
        }
    };
}
pub fn remove(sprite_id: TSpriteID) -> Result<TSpriteID, String> {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();

    let found_index = singleton
        .sprites
        .binary_search_by(|sprite| sprite.id.cmp(&sprite_id));
    match found_index {
        Ok(i) => {
            let x = singleton.sprites.remove(i);
            return Ok(x.id);
        }
        Err(e) => Err(format!("spriteID={} already delted - {}", sprite_id, e)), // do nothing if already deleted...
    }
}
// per frame, this gets called (and emptied on update completions)
pub fn add_sprite_for_update(sprite_id: TSpriteID) {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();
    match singleton.updates.binary_search_by(|k| k.cmp(&sprite_id)) {
        Ok(_k_index) => (),
        Err(_) => singleton.updates.push(sprite_id),
    }
}

// Note: singleton will be locked (MUTEX) while in this update loop
pub fn update_sprites() {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();

    // only update sprites that has been requested to be updated
    for sprite_id in &singleton.updates {
        match singleton.sprites.binary_search_by(|s| s.id.cmp(sprite_id)) {
            Ok(sid) => {
                let sp = singleton.sprites[sid];
                sp.update();
            }
            Err(_) => (),
        };
    }

    // finally, clear process list
    singleton.updates.clear();
}

pub fn reset_sprites() {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();
    singleton.sprites.clear();
    singleton.updates.clear();
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Sprite {
    pub id: TSpriteID, // GroupID; groups for example, a Cannon sprite group may have sub-groups of idle, firing, and broken sub-group sprites
    pub resource_id: TResourceID,
    // TODO: Add sub-group collection (should it be recursive of sprite holding a collection of sprite?)
}
impl Sprite {
    pub fn new() {}
    pub fn update(self: &Self) {}
}

#[cfg(test)]
mod tests {
    //use serde_test::{assert_tokens, Token};

    #[test]
    fn my_test_1() {}
}
