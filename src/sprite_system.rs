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
}

impl SpriteFactory {
    fn new() -> SpriteFactory {
        // Initialize your data here
        SpriteFactory {
            sprites: Vec::new(),
        }
    }
}

pub type TSpriteID = u16;

pub fn get(resource_id: &TResourceID) -> Option<Sprite> {
    let singleton = SPRITE_SINGLETON.lock().unwrap();
    match singleton
        .sprites
        .binary_search_by(|sprite| sprite.resource_id.cmp(&resource_id))
    {
        Ok(index) => {
            return Some(singleton.sprites.get(index).unwrap().clone());
        }
        Err(_) => {
            return None;
        }
    };
}
pub fn add_sprite(resource_id: &TResourceID) -> Result<TSpriteID, String> {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();

    // NOTE: because we always get +1 of last max spriteID, there will be (hopefully rare)
    // chance that it may wrap around after Max==65535 sprites.  Hoopefully, during each
    // loading/reloading, we can re-pack and/or flush sprite list
    let max_id = match singleton.sprites.iter().max_by_key(|e| e.id) {
        Some(s) => s.id,
        _ => 0 as TSpriteID, // edge-case when list is empty
    };
    let found_sprite = get(&resource_id);

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
pub fn del_sprite(sprite_id: TSpriteID) -> Result<TSpriteID, String> {
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
pub fn update_sprites() {
    //let mut singleton = SPRITE_SINGLETON.lock().unwrap();
}
pub fn reset_sprites() {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();
    singleton.sprites.clear();
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Sprite {
    pub id: TSpriteID,
    pub resource_id: TResourceID,
}

#[cfg(test)]
mod tests {
    //use serde_test::{assert_tokens, Token};

    #[test]
    fn my_test_1() {}
}
