use lib_tower_defense::{
    resource_system::{self, TResourceID},
    sprite_system::{*, self},
};

// Note: at this level, we assume sprites are based off of SDL2 agnostic
use sdl2::{image::LoadSurface, rect::Rect, surface::Surface};

use std::{path::Path, sync::Mutex};

use image::{GenericImageView, Rgba};

// This version of the function takes an additional chroma_key: Option<Rgba<u8>> parameter.
// If chroma_key is Some(key), then pixels with the value key will be treated as transparent.
// If chroma_key is None, then only pixels with an alpha value of 0 will be treated as transparent.

fn find_bounding_boxes(image: &image::DynamicImage, chroma_key: Option<Rgba<u8>>) -> Vec<Rect> {
    let mut boxes = Vec::new();
    let (width, height) = image.dimensions();
    let mut visited = vec![vec![false; height as usize]; width as usize];
    for y in 0..height {
        for x in 0..width {
            if visited[x as usize][y as usize] {
                continue;
            }
            let pixel = image.get_pixel(x, y);
            if pixel[3] != 0 && chroma_key.map_or(true, |key| key != pixel) {
                let mut min_x = x;
                let mut max_x = x;
                let mut min_y = y;
                let mut max_y = y;
                let mut stack = vec![(x, y)];
                while let Some((x, y)) = stack.pop() {
                    if visited[x as usize][y as usize] {
                        continue;
                    }
                    visited[x as usize][y as usize] = true;
                    let pixel = image.get_pixel(x, y);
                    if pixel[3] != 0 && chroma_key.map_or(true, |key| key != pixel) {
                        min_x = min_x.min(x);
                        max_x = max_x.max(x);
                        min_y = min_y.min(y);
                        max_y = max_y.max(y);
                        if x > 0 {
                            stack.push((x - 1, y));
                        }
                        if x < width - 1 {
                            stack.push((x + 1, y));
                        }
                        if y > 0 {
                            stack.push((x, y - 1));
                        }
                        if y < height - 1 {
                            stack.push((x, y + 1));
                        }
                    }
                }
                boxes.push(Rect::new(
                    min_x as i32,
                    min_y as i32,
                    (max_x - min_x + 1) as u32,
                    (max_y - min_y + 1) as u32,
                ));
            }
        }
    }
    boxes
}

pub struct ViewSprite {
    sprite_id: sprite_system::TSpriteID, // should be able to extract ResourceID/GroupID, SubGroupID, SubSpriteID from this ID...
    sprite_width: u32,                   // dimension of sprite group
    sprite_height: u32,
}

pub fn deserialize_sprite(temp_sprite_resource_id: &TResourceID) -> Vec<Sprite> {
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
