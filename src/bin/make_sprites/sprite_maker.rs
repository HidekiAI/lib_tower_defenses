use lib_tower_defense::sprite_system::*;

// regions are WITHOUT the edge boundaries
struct Region {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}
struct RawSpriteAtlas {
    row: usize,          // from top down
    region: Vec<Region>, // sequence of regions from left to right
}

// Scan pixels each rows until we find first transparent pixel and treat that
// for the rest of the entire sprite sheet as the border color
fn find_border_color(pixels: &[u8], width: usize, height: usize) -> u32 {
    let mut border_color = 0;
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) * 4;
            let alpha = pixels[index + 3];
            if alpha == 0 {
                continue;
            }
            border_color = u32::from_be_bytes([
                pixels[index],
                pixels[index + 1],
                pixels[index + 2],
                pixels[index + 3],
            ]);
            break;
        }
        if border_color != 0 {
            break;
        }
    }
    border_color
}

// based off of the (x, y) as the upper left corner of the blocks of
// sprites, find the bottom right corner of the block of sprites
fn find_region(
    pixels: &[u8],
    width: usize,
    height: usize,
    x: usize,
    y: usize,
    border_color: u32,
) -> Region {
    let mut region = Region {
        x: x as u32,
        y: y as u32,
        width: 0,
        height: 0,
    };
    // first, we'll find the height of the region by scanning at (x, y+offset)
    // until y+offset returns a row which does not match the border color
    for offset in 0.. {
        let index = ((y + offset) * width + x) * 4;
        let alpha = pixels[index + 3];
        if alpha == 0 {
            break;
        }
        let color = u32::from_be_bytes([
            pixels[index],
            pixels[index + 1],
            pixels[index + 2],
            pixels[index + 3],
        ]);
        if color != border_color {
            break;
        }
        region.height += 1;
    }
    // then we'll find the width of the region by scanning at (x+offset, y)
    // until x+offset returns a column which does not match the border color
    for offset in 0.. {
        let index = (y * width + x + offset) * 4;
        let alpha = pixels[index + 3];
        if alpha == 0 {
            break;
        }
        let color = u32::from_be_bytes([
            pixels[index],
            pixels[index + 1],
            pixels[index + 2],
            pixels[index + 3],
        ]);
        if color != border_color {
            break;
        }
        region.width += 1;
    }
    region
}

// based off of a region, we'll find sub-regions which are the individual
// sprites
fn find_sub_regions(
    pixels: &[u8],
    width: usize,
    height: usize,
    region: &Region,
    border_color: u32,
) -> Vec<Region> {
    let mut sub_regions = Vec::new();
    let mut x = region.x;
    let mut y = region.y;
    loop {
        let sub_region = find_region(pixels, width, height, x as usize, y as usize, border_color);
        if sub_region.width == 0 || sub_region.height == 0 {
            break;
        }
        x += sub_region.width;
        sub_regions.push(sub_region);
    }
    sub_regions
}
