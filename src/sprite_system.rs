use super::resource_system::*;
use once_cell::sync::Lazy;
//use serde::Serialize;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Mutex};

// Note: No need to drop/deconstruct/destroy once it's created
static SPRITE_SINGLETON: Lazy<Mutex<SpriteFactory>> =
    Lazy::new(|| Mutex::new(SpriteFactory::new()));
struct SpriteFactory {
    pub sprites: Vec<Sprite>,    // flattened list of sprites
    updates: HashSet<TSpriteID>, // based on subGroupID, can we determine what to do with it?
}

impl SpriteFactory {
    fn new() -> SpriteFactory {
        // Initialize your data here
        SpriteFactory {
            sprites: Vec::new(),
            updates: HashSet::new(),
        }
    }
}

// this should be derived, default returns empty sprite list
pub fn default_deserialize_sprite(_temp_sprite_resource_id: &TResourceID) -> Vec<Sprite> {
    return Vec::<Sprite>::new();
}
// GroupID: i.e. ogre sprites
// SubGroupID: walking sprites of ogre sprites
// SpriteID: each indivisual sprite ID of walking ogre sprites
// Eg: {Group:1, {SubGrup:0, {Sprite:0, 1, 2} }, {SubGroup:1, {Sprite:0, 1, 2}}
// another way to see this, is by looking at sprite ID, one can locate its
// siblings so that one can animate via cycled loop
pub type TSpriteSubGroupID = u8; // per group, it starts from ID=0
pub type TSubSpriteID = u8; // per sub-group, starts from ID=0
pub type TSpriteID = u16; // unique ID that groups together group/Resource ID + SubGroupID + SubSpriteID combinations

pub fn try_get(internal_sprite_id: &TSpriteID) -> Option<Sprite> {
    match SPRITE_SINGLETON.try_lock() {
        Ok(singleton) => {
            let possible_sprites: Vec<_> = singleton
                .sprites
                .iter()
                .filter(|sprite| sprite.id.eq(&internal_sprite_id))
                //.map(|s| s.clone())
                .collect();
            match possible_sprites.len() > 0 {
                true => {
                    if possible_sprites.len() > 1 {
                        panic!(
                            "found more than 1 ({cnt}) sprites with spriteID={sid}",
                            cnt = possible_sprites.len(),
                            sid = internal_sprite_id
                        )
                    } else {
                        possible_sprites.first().map(|s| *s.clone())
                    }
                }
                false => None,
            }
        }
        Err(_) => None,
    }
}
// there is no get(), for it can potentially cause deadlocks based on temptations
// to be used at critical sections; hence it is intentionally exposing try_get
// that can (and will) return None immediately rather than blocking
pub fn try_get_sub_groups(resource_id: &TResourceID) -> Vec<TSpriteSubGroupID> {
    match SPRITE_SINGLETON.try_lock() {
        Ok(singleton) => {
            let possible_sub_groups: Vec<_> = singleton
                .sprites
                .iter()
                .filter(|sprite| sprite.group_id.eq(&resource_id))
                .map(|spr| spr.subgroup_id)
                .collect();
            return possible_sub_groups;
        }
        Err(_) => Vec::<TSpriteSubGroupID>::new(), // cannot tell whether we've encountered MUTEX lock or really cannot locate the resource
    }
}
pub fn try_get_sprites(sub_group_id: &TSpriteSubGroupID) -> Vec<Sprite> {
    match SPRITE_SINGLETON.try_lock() {
        Ok(singleton) => {
            let possible_sprites: Vec<_> = singleton
                .sprites
                .iter()
                .filter(|sprite| sprite.subgroup_id.eq(&sub_group_id))
                .collect();
            let mut vec_sprites = Vec::<Sprite>::new();
            for s in possible_sprites {
                vec_sprites.push(s.clone());
            }
            return vec_sprites;
        }
        Err(_) => Vec::new(),
    }
}

/// Adds sprites to singleton collection; note that there will often be more
/// than one sprite with same ResourceID, but will have different SpriteID
/// assigned to it.  This is because same resource is not assumed to synchronize
/// on the animations.  I.e. one entity may be in the walking sprite sequence
/// while another entity, with same sprite group id is firing a projectile
/// Because the sprite system tries to be somewhat agnostic of the view (TUI vs SDL2, etc)
/// (somewhat, because collsion is somewhat associates to view), the callback
/// lambda/closures/fn/delegate/whatever will do it's own parsing
pub fn add(
    resource_id: &TResourceID,
    func_deserialize_sprites: impl Fn(&TResourceID) -> Vec<Sprite>, // using impl instead of "where" trait
) -> Result<HashSet<TSpriteSubGroupID>, String> {
    // we'll lock-and-block here pror to adding
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();
    let possible_sprites: Vec<&Sprite> = singleton
        .sprites
        .iter()
        .filter(|sprite| sprite.group_id.eq(&resource_id))
        .collect();
    // we should panic() if this group_id is found, or should we just return Vec<SpriteID's>?
    if possible_sprites.len() > 0 {
        //let error_msg = format!("resource_id={} (aka sprite group) is already added, no need to call add more than once", resource_id);
        //panic!(error_msg);
        //return Err(error_msg);

        // return ALL the subgroups for this resource
        let mut v_sprites = HashSet::<TSubSpriteID>::new(); // we need to clone so that caller can have their own copy
        for s in possible_sprites.iter().map(|spr| spr.subgroup_id) {
            v_sprites.insert(s);
        }
        return Ok(v_sprites);
    }

    // OK, let's add this group using enumerate so that we can gain index and the element (similar
    // to F# List.mapi(fun index element => ... ) )
    // NOTE: func_deserialize_sprites will make sure to set spriteID sequentially for this resource
    let binding = func_deserialize_sprites(resource_id);
    let sprites_from_resource = binding
        .iter()
        .enumerate()
        .map(|(index, element)| (index, element)); // won't call .collect() here, so we can iterate...

    let mut v_sprites = HashSet::<TSubSpriteID>::new(); // we need to clone so that caller can have their own copy
    for (_i, s) in sprites_from_resource {
        singleton.sprites.push(s.to_owned()); // take ownership, this system should have that ownership
        v_sprites.insert(s.sub_id); // match index i?
    }
    return Ok(v_sprites);
}

pub fn remove(sprite_id: TSubSpriteID) -> Result<(TSpriteSubGroupID, TResourceID), String> {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();

    let found_index = singleton
        .sprites
        .binary_search_by(|sprite| sprite.sub_id.cmp(&sprite_id));
    match found_index {
        Ok(i) => {
            let x = singleton.sprites.remove(i);
            return Ok((x.subgroup_id, x.group_id));
        }
        Err(e) => Err(format!("spriteID={} already delted - {}", sprite_id, e)), // do nothing if already deleted...
    }
}
pub fn remove_group(resource_id: TResourceID) -> Result<Vec<TSubSpriteID>, String> {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();
    let found_sprites: Vec<TSubSpriteID> = singleton
        .sprites
        .iter()
        .filter(|sprite| sprite.group_id.eq(&resource_id))
        .map(|spr| spr.sub_id)
        .collect();
    if found_sprites.len() == 0 {
        return Err(format!(
            "Unable to locate sprite groupID={} (resource_id) in current sprite collection",
            resource_id
        ));
    }
    // create a new sprite list WITHOUT the resource_id
    let new_list = singleton
        .sprites
        .clone()
        .into_iter()
        .filter(|x| x.group_id != resource_id)
        .collect();
    singleton.sprites = new_list;

    return Ok(found_sprites);
}

// per frame, this gets called (and emptied on update completions)
pub fn add_sprite_for_update(sprite_id: TSpriteID) -> bool {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();
    return singleton.updates.insert(sprite_id);
}

// Note: singleton will be locked (MUTEX) while in this update loop
pub fn update_sprites() {
    let mut singleton = SPRITE_SINGLETON.lock().unwrap();

    // only update sprites that has been requested to be updated
    for sprite_id in &singleton.updates {
        match singleton
            .sprites
            .binary_search_by(|spr| spr.id.cmp(sprite_id))
        {
            Ok(spr_index) => {
                let sp = singleton.sprites[spr_index];
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

// TODO: Possibly, we can store this as SQLite3 (each struct represents a table?)
// flattened entity which holds group and subgroup ids, so that we don't have to
// juggle around 3 different collections which can become a sync nightmare (i.e.
// if I remove a sprite ID, I'd then have to go look at subgroup to see if it was
// the last in the list, in whihc I'd then have to delete that subgroup, and then
// I'd have to look at group... and so on..)
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct Sprite {
    pub id: TSpriteID, // unique ID (private?) used primarily in internal usage to identify group+subgroup+subID combination
    pub group_id: TResourceID, // GroupID; groups for example, a Cannon sprite group may have sub-groups of idle, firing, and broken sub-group sprites
    pub subgroup_id: TSpriteSubGroupID, // use this to find all the sprites that make up the cycling of sprite animations
    pub sub_id: TSubSpriteID, // NOTE: the ascending order will also indicate the animation cycle loop
    pub width: u16,
    pub height: u16,
    pub hotpoint_x: i16, // NOTE: Hotpoints are offset from upper-left corner of the sprite
    pub hotpoint_y: i16,
    pub collision_rect_upper_left_x: i16,
    pub collision_rect_upper_left_y: i16,
    pub collision_rect_width: u16,
    pub collision_rect_height: u16,
}
impl Sprite {
    pub fn new() -> Sprite {
        Sprite {
            id: todo!(),
            group_id: todo!(),
            subgroup_id: todo!(),
            sub_id: todo!(),
            width: todo!(),
            height: todo!(),
            hotpoint_x: todo!(),
            hotpoint_y: todo!(),
            collision_rect_upper_left_x: todo!(),
            collision_rect_upper_left_y: todo!(),
            collision_rect_width: todo!(),
            collision_rect_height: todo!(),
        }
    }
    pub fn update(self: &Self) {
        // currently, there's really NOTHING to update per sprite (any view-related does not go here)
        // if Sprite needs to be reloaded because it is no longere cached, it does not go here
    }
}

#[cfg(test)]
mod tests {
    //use serde_test::{assert_tokens, Token};

    #[test]
    fn my_test_1() {}
}
