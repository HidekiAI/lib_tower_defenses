use std::{cell::Cell, slice};

const MAX_MAP_WIDTH: usize = 1024;
const MAX_MAP_HEIGHT: usize = 1024;
const MAX_LAYERS_PER_CELL: usize = 16;
type TcellValue = i64;

#[derive(Debug, Clone, Copy)]
pub struct CellLayer {
    pub id: u8,
    pub val: TcellValue, // TODO: if val becomes a complex data-model, make this private and create impl for this struct
}
#[derive(Debug, Clone)]
pub struct MapCell {
    pub layers: Vec<CellLayer>, // this means we cannot derive Copy, use .clone()
}
impl MapCell {
    pub fn set(self: &Self, newID: u8, newVal: TcellValue) -> Result<MapCell, String> {
        let mut new_layers: Vec<CellLayer> = Vec::new();
        let new_layers_iter = self.layers.iter().filter(|v| v.id != newID).map(|v| *v);

        for iter in new_layers_iter {
            new_layers.push(iter);
        }
        new_layers.push(CellLayer {
            id: newID,
            val: newVal,
        });

        if new_layers.len() + 1 > MAX_LAYERS_PER_CELL {
            return Err(format!("Cell ID {} exceeds {}", newID, MAX_LAYERS_PER_CELL));
        }

        return Ok(MapCell { layers: new_layers });
    }
    pub fn get(self: &Self) -> Vec<CellLayer> {
        return self.layers.clone(); // make a copy
    }
    pub fn first(self: &Self) -> Option<CellLayer> {
        return match self.layers.is_empty() {
            false => Some(self.layers.first().unwrap().clone()),
            true => None,
        };
    }
    pub fn last(self: &Self) -> Option<CellLayer> {
        return match self.layers.is_empty() {
            false => Some(self.layers.last().unwrap().clone()),
            true => return None,
        };
    }
}

// UpperLeft (0,0), while BottomRight is (map_width, map_height)
// hence movement on +X moves to right and movment in +Y moves downwards.
#[derive(Debug, Clone)]
pub struct Map {
    // for optimization reasons, rather than being column-ordered grid[x][y], the grid is
    // row-ordered grid[row][column] for views
    grid: Vec<Vec<MapCell>>, // having it as fixed 2D-array causes stack-overflow so had to switch to Vec<Vec<>>
    width: u16,
    height: u16,
    current_x: u16, // UpperLeft, for moving about the map (mainly for View)
    current_y: u16,
}
impl Map {
    pub fn get_width(self: &Self) -> u16 {
        return self.width;
    }
    pub fn get_height(self: &Self) -> u16 {
        return self.height;
    }
    pub fn create(width: u16, height: u16) -> Result<Map, String> {
        if (width as usize) > MAX_MAP_WIDTH {
            return Err(format!(
                "Invalid Dimension: Width({}) is greater than {}",
                width, MAX_MAP_WIDTH
            ));
        }
        if (height as usize) > MAX_MAP_HEIGHT {
            return Err(format!(
                "Invalid Dimension: Height({}) is greater than {}",
                height, MAX_MAP_HEIGHT
            ));
        }

        let g = vec![
            vec![
                MapCell {
                    layers: Vec::new(), // start with Empty (flat plain cell)
                };
                MAX_MAP_WIDTH
            ];
            MAX_MAP_HEIGHT  // per row
        ];
        let map = Map {
            width: width,
            height: height,
            grid: g,
            current_x: 0,
            current_y: 0,
        };
        Ok(map)
    }
    pub fn get_upper_left(self: &Self) -> (u16, u16) {
        return (self.current_x, self.current_y);
    }

    pub fn set_upper_left(self: &mut Self, x: u16, y: u16) -> (u16, u16) {
        let mut new_x = x;
        let mut new_y = y;
        if x >= self.width {
            new_x = self.width;
        }
        if x >= self.width {
            new_y = self.height;
        }
        self.current_x = new_x;
        self.current_y = new_y;
        return (new_x, new_y);
    }

    pub fn get_cell(self: &Self, map_x: u16, map_y: u16) -> Result<MapCell, String> {
        if map_x as usize > MAX_MAP_WIDTH {
            return Err(format!(
                "Cannot assign MapCell to position ([{}], {}) for its X position exceeds {}",
                map_x, map_y, MAX_MAP_WIDTH
            ));
        }
        if map_y as usize > MAX_MAP_HEIGHT {
            return Err(format!(
                "Cannot assign MapCell to position ({}, [{}]) for its Y position exceeds {}",
                map_x, map_y, MAX_MAP_HEIGHT
            ));
        }
        return Ok(self.grid[map_y as usize][map_x as usize].clone()); // need to clone() since we cannot copy()
    }
    pub fn get_cell_view(
        self: &Self,
        view_offset_x: u8,
        view_offset_y: u8,
    ) -> Result<MapCell, String> {
        return self.get_cell(
            self.current_x + view_offset_x as u16,
            self.current_y + view_offset_y as u16,
        );
    }

    // Using Vec<T> instead of Box<[Tslice_type]>
    pub fn get_cell_row(
        self: &Self,
        map_x: u16,
        map_y: u16,
        width: u8,
    ) -> Result<Vec<MapCell>, String> {
        let row = &self.grid[map_y as usize];
        let islice = row.iter().skip(map_x as usize).take(width as usize);

        let mut vslice: Vec<MapCell> = Vec::new();
        islice.for_each(|mc| vslice.push(mc.clone()));
        return Ok(vslice); // clone since we don't have Copy
    }

    pub fn set(self: &mut Self, map_x: u16, map_y: u16, cell: MapCell) -> Result<bool, String> {
        if map_x as usize > MAX_MAP_WIDTH {
            return Err(format!(
                "Cannot assign MapCell to position ([{}], {}) for its X position exceeds {}",
                map_x, map_y, MAX_MAP_WIDTH
            ));
        }
        if map_y as usize > MAX_MAP_HEIGHT {
            return Err(format!(
                "Cannot assign MapCell to position ({}, [{}]) for its Y position exceeds {}",
                map_x, map_y, MAX_MAP_HEIGHT
            ));
        }
        self.grid[map_y as usize][map_x as usize] = cell;
        return Ok(true);
    }
    pub fn set_view(
        self: &mut Self,
        view_offset_x: u8,
        view_offset_y: u8,
        cell: MapCell,
    ) -> Result<bool, String> {
        return self.set(
            self.current_x + view_offset_x as u16,
            self.current_y + view_offset_y as u16,
            cell,
        );
    }
    pub fn set_row(
        self: &mut Self,
        map_y: u16,
        x_offset: u8,
        row_slice: Vec<MapCell>,
    ) -> Result<bool, String> {
        if map_y as usize > MAX_MAP_HEIGHT {
            return Err(format!(
                "Cannot assign row of MapCells to position ({}, [{}]) for its Y position exceeds {}",
                x_offset, map_y, MAX_MAP_HEIGHT
            ));
        }
        if (x_offset as usize + row_slice.len()) > MAX_MAP_WIDTH {
            return Err(format!(
                "Cannot assign row of MapCells to position ([{}], {}) for its X position exceeds {}",
                x_offset, map_y, MAX_MAP_WIDTH
            ));
        }
        for i in 0..(row_slice.len() - 1) {
            self.set(
                x_offset as u16 + i as u16,
                map_y as u16,
                row_slice[i].clone(),
            );
        }
        return Ok(true);
    }

    // convert 2D to single array strided
    pub fn build_view(
        self: &Self,
        view_offset_x: u8,
        view_offset_y: u8,
        view_width: u8,
        view_height: u8,
    ) -> Result<Vec<TcellValue>, String> {
        let map_x = self.current_x + view_offset_x as u16;
        let map_y = self.current_y + view_offset_y as u16;
        if (map_x as usize) > MAX_MAP_WIDTH {
            return Err(format!(
                "ViewXTop={} exceeds max width dimension {}",
                map_x, MAX_MAP_WIDTH
            ));
        }
        if (map_y as usize) > MAX_MAP_HEIGHT {
            return Err(format!(
                "ViewYTop={} exceeds max height dimension {}",
                map_y, MAX_MAP_HEIGHT
            ));
        }
        if (map_x as usize + view_width as usize) > MAX_MAP_WIDTH {
            return Err(format!(
                "ViewXBottom={} (width={}) exceeds max width dimension {}",
                map_x + view_width as u16,
                view_width,
                MAX_MAP_WIDTH
            ));
        }
        if (map_y as usize + view_height as usize) > MAX_MAP_HEIGHT {
            return Err(format!(
                "ViewYBottom={} (height={}) exceeds max height dimension {}",
                map_y + view_height as u16,
                view_height,
                MAX_MAP_HEIGHT
            ));
        }
        if view_width as u16 > self.width {
            return Err(format!(
                "ViewWidth={} exceeds max width dimension of this map {}",
                view_width, self.width
            ));
        }
        if view_height as u16 > self.height {
            return Err(format!(
                "ViewHeight={} exceeds max height dimension of this map {}",
                view_height, self.height
            ));
        }
        let mut slices: Vec<Vec<MapCell>> = Vec::new();
        for row in 0..view_height {
            let perRowSlice = self.get_cell_row(map_x, map_y, view_width).unwrap();
            slices.push(perRowSlice);
        }

        let mut retSlicesFlatten: Vec<TcellValue> = Vec::new();
        for vRow in slices {
            let mut vals: Vec<TcellValue> = Vec::new();
            for vc in vRow {
                let mut topMost: TcellValue = 0;
                if vc.layers.len() > 0 {
                    // this this be reversed iteratations where tail of the queue is the topmost?
                    topMost = vc.layers.first().unwrap().val; // we've already tested len() > 0, so safe to unwrap here
                    break;
                }

                vals.push(topMost);
            }
            retSlicesFlatten.append(&mut vals);
        }

        return Ok(retSlicesFlatten);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create64x128() {
        let theMap = Map::create(64, 128).unwrap();
        assert_eq!(theMap.grid[0][0].layers.len(), 0); // when freshly creaed, each/any layers are empty
    }

    #[test]
    fn can_update_layer() {
        let mut theMap = Map::create(64, 128).unwrap(); // gotta make it mutable if we're going to allow update
        let posX = 0;
        let posY = 0;
        let layerID = 0;
        let newVal = 2;
        let theResult = theMap.grid[posX as usize][posY as usize].set(layerID, newVal);
        theMap.grid[posX as usize][posY as usize] = theResult.unwrap();
        assert_eq!(
            theMap.grid[posX as usize][posY as usize].layers[layerID as usize].val,
            newVal
        );
    }

    #[test]
    fn test_set_map_row() {
        let mut theMap = Map::create(16, 32).unwrap();
        let map_y = 8;
        let x_offset = 4;
        let row_slice = theMap.get_cell_row(x_offset, map_y, 8).unwrap();
        theMap.set_row(map_y as u16, x_offset as u8, row_slice);
    }

    #[test]
    fn test_view_for_rendering() {
        let theMap = Map::create(16, 32).unwrap();
        let view_x = 2;
        let view_y = 2;
        let view_width = 4;
        let view_height = 8;
        let view = theMap
            .build_view(view_x, view_y, view_width, view_height)
            .unwrap();
        for h in 0..view_height {
            for w in 0..view_width {
                print!("{}", view[((h * view_width) + w) as usize]);
            }
            println!("");
        }
    }
}
