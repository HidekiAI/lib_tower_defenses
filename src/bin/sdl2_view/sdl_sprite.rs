extern crate sdl2;
//use crate::components::*;
use std::borrow::BorrowMut;
use std::time::Duration;
// Note: at this level, we assume sprites are based off of SDL2 agnostic
use image::{GenericImageView, Rgba};
use sdl2::render::Canvas;
use sdl2::video::{Window, WindowContext, WindowPos};
use sdl2::{
    event::Event,
    image::{InitFlag, LoadSurface, LoadTexture},
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::{Texture, TextureCreator, WindowCanvas}, // WindowCanvas is a type alias for Canvas<Window>
    surface::Surface,
    Sdl,
    VideoSubsystem,
};
use specs::prelude::*; //{ReadStorage, VecStorage};
use specs_derive::Component;

use lib_tower_defense::{
    resource_system::{self, TResourceID},
    sprite_system::{self, *},
};

//Note: type WindowCanvas = Canvas<Window>;
pub struct SDLDisplay {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    // though we'd want texture within same location as WindowCanvas, we can't have it here because of lifetime issues
    canvas: WindowCanvas,
    // we need to keep the texture creator around to create textures
    texture_creator: TextureCreator<WindowContext>,
    //texture: Texture<'r>,
}

impl SDLDisplay {
    pub fn new(win_title: &str, win_width: u32, win_height: u32) -> Result<SDLDisplay, String> {
        match sdl2::init() {
            Ok(sdl_context) => match sdl_context.video() {
                Ok(video_subsystem) => {
                    match video_subsystem
                        .window(win_title, win_width, win_height)
                        .position_centered()
                        .build()
                        .map_err(|win_build_error| win_build_error.to_string())
                    {
                        Ok(window) => {
                            match window.into_canvas().build().map_err(|e| e.to_string()) {
                                Ok(canvas) => {
                                    let texture_creator = canvas.texture_creator();
                                    return Ok(SDLDisplay {
                                        sdl_context,
                                        canvas,
                                        texture_creator,
                                        video_subsystem,
                                    });
                                }
                                Err(e) => {
                                    return Err(e.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            return Err(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    return Err(e.to_string());
                }
            },
            Err(e) => {
                return Err(e.to_string());
            }
        }
    }

    //it's essential to understand that this Texture can only be rendered in the Canvas that created it
    //pub fn load_texture(&mut self, str_filename: &str) -> Result<Texture, String> {
    //    return self.texture_creator.load_texture(str_filename);
    //}

    //pub fn update(&mut self, frame: &i32) -> Result<(), String> {
    //    match self.texture_creator.load_texture("assets/texture.png".into()) {
    //        Ok(texture) => {
    //            //let mut canvas = &self.canvas;
    //            //canvas.set_draw_color(Color::RGB(255, 255, 255));
    //            //canvas.clear();
    //            //let current_frame = (frame / 10) % 6;
    //            //let src_rect = Rect::new(current_frame * 64, 0, 64, 64);
    //            //let dst_rect = Rect::new(100, 100, 64, 64);
    //            //canvas.copy(&texture, src_rect, dst_rect)?;
    //            //canvas.present();
    //            Ok(())
    //        }
    //        Err(e) => Err(e),
    //    }
    //}
}

//struct Spritesheet<'a> {
//    spritesheet_texture: sdl2::render::Texture<'a>,
//    clip_rect: sdl2::rect::Rect,
//}
//
//impl Spritesheet {
//    pub fn new<'Ta>(texture: &'Ta sdl2::render::Texture, rows: u32, columns: u32) -> Self {
//        let width = texture.query().width / columns;
//        let height = texture.query().height / rows;
//        Self {
//            spritesheet_texture: texture,
//            clip_rect: sdl2::rect::Rect::new(0, 0, width, height),
//        }
//    }
//
//    fn select_sprite(&mut self, x: i32, y: i32) {
//        self.clip_rect.set_x(x * self.clip_rect.width() as i32);
//        self.clip_rect.set_y(y * self.clip_rect.height() as i32);
//    }
//
//    fn draw_selected_sprite(
//        &self,
//        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
//        position: sdl2::rect::Rect,
//    ) {
//        canvas
//            .copy(&self.spritesheet_texture, self.clip_rect, position)
//            .unwrap();
//    }
//}
//
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

/// The current position of a given entity
#[derive(Component, Debug)]
#[storage(VecStorage)]
pub struct Position(pub Point);

// Type alias for the data needed by the renderer
pub type SystemData<'a> = (ReadStorage<'a, Position>, ReadStorage<'a, Sprite>);
pub fn render(
    canvas: &mut WindowCanvas,
    background: Color,
    textures: &[Texture],
    data: SystemData,
) -> Result<(), String> {
    canvas.set_draw_color(background);
    canvas.clear();

    let (width, height) = canvas.output_size()?;

    for (pos, sprite) in (&data.0, &data.1).join() {
        let current_frame = sprite.region;

        // Treat the center of the screen as the (0, 0) coordinate
        let screen_position = pos.0 + Point::new(width as i32 / 2, height as i32 / 2);
        let screen_rect = Rect::from_center(
            screen_position,
            current_frame.width(),
            current_frame.height(),
        );
        canvas.copy(&textures[sprite.spritesheet], current_frame, screen_rect)?;
    }

    canvas.present();

    Ok(())
}
