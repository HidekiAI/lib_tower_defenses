use lib_tower_defense::{
    resource_system::{self, TResourceID},
    sprite_system::TSpriteID,
};

// Note: at this level, we assume sprites are based off of SDL2 agnostic
use sdl2::{image::LoadSurface, rect::Rect, surface::Surface};

use std::{path::Path, sync::Mutex};
pub struct ViewSprite {
    sprite_id: sprite_system::TSpriteID, // should be able to extract ResourceID/GroupID, SubGroupID, SubSpriteID from this ID...
    sprite_width: u32,    // dimension of sprite group
    sprite_height: u32,
}

fn deserialize_sprite(temp_sprite_resource_id: &TResourceID) -> Vec<Sprite> {
    todo!("CODE ME!")
}

pub fn get_sprite_dimensions(
    sprite_sheet_path: TResourceID,
    chroma_key: (u8, u8, u8),
    sprite_groupings: &[(i32, i32, i32, i32)],
) -> Vec<(u32, u32)> {
    let mut sprite_dimensions = Vec::new();

    let resource = resource_system::Resource::try_get(sprite_sheet_path).unwrap();

    // Load the sprite sheet surface
    let mut sprite_sheet = Surface::from_file(resource.paths).unwrap();

    // Set the color key for the sprite sheet surface
    sprite_sheet
        .set_color_key(
            true,
            sdl2::pixels::Color::RGB(chroma_key.0, chroma_key.1, chroma_key.2),
        )
        .unwrap();

    // Iterate over the sprite groupings
    for grouping in sprite_groupings {
        // Calculate the width and height of the current grouping
        let width = (grouping.2 - grouping.0) + 1;
        let height = (grouping.3 - grouping.1) + 1;

        // Create a new surface with the dimensions of the current grouping
        let mut group_surface = Surface::new(
            width as u32,
            height as u32,
            sprite_sheet.pixel_format_enum(),
        )
        .unwrap();

        // Blit the current grouping from the sprite sheet surface to the group surface
        sprite_sheet
            .blit(
                Rect::new(grouping.0, grouping.1, width as u32, height as u32),
                &mut group_surface,
                None,
            )
            .unwrap();

        // Lock the group surface pixels
        let _ = group_surface.with_lock(|pixels| {
            // Iterate over the pixels in the group surface
            for y in 0..height {
                for x in 0..width {
                    // Get the pixel at the current position
                    let pixel = unsafe { *pixels.as_ptr().offset((y * width + x) as isize) };

                    // Check if the pixel is not transparent
                    if pixel != 0 {
                        // Calculate the dimensions of the current sprite
                        let sprite_width = (x + 1) as u32;
                        let sprite_height = (y + 1) as u32;

                        // Add the dimensions of the current sprite to the vector of sprite dimensions
                        sprite_dimensions.push((sprite_width, sprite_height));

                        // Break out of the inner loop
                        break;
                    }
                }
            }
        });
    }

    // Return the vector of sprite dimensions
    sprite_dimensions
}
