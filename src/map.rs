extern crate serde;

use rmp_serde::{decode, encode, from_slice, Deserializer, Serializer};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};
//use serde::{Deserialize, Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{cell::Cell, error::Error, slice};

const MAX_MAP_WIDTH: usize = 1024;
const MAX_MAP_HEIGHT: usize = 1024;
const MAX_LAYERS_PER_CELL: usize = 16;
type TcellValue = i64;

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct CellLayer {
    pub id: u8,
    pub val: TcellValue, // TODO: if val becomes a complex data-model, make this private and create impl for this struct
}

//impl Serialize for CellLayer {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer,
//    {
//        let mut state = serializer.serialize_struct("CellLayer", 2)?; // currently, only 2 fields
//        state.serialize_field("id", &self.id)?;
//        state.serialize_field("val", &self.val)?;
//        state.end()
//    }
//}

impl CellLayer {
    pub fn new(new_id: u8, new_val: TcellValue) -> CellLayer {
        CellLayer {
            id: new_id,
            val: new_val,
        }
    }
    //pub fn save(self: &Self) -> Result<String, String> {
    //    let s = serde_json::to_string(self).unwrap();
    //    return Ok(s);
    //}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
//#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapCell {
    pub layers: Vec<CellLayer>, // this means we cannot derive Copy, use .clone()
}

//impl Serialize for MapCell {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//        S: Serializer,
//    {
//        let mut state = serializer.serialize_struct("MapCell", 1)?; // currently, only 2 fields
//        state.serialize_field("layers", &self.layers)?;
//        state.end()
//    }
//}
impl MapCell {
    // add or update layer
    pub fn set(self: &mut Self, new_id: u8, new_val: TcellValue) -> Result<(), String> {
        let mut new_layers: Vec<CellLayer> = Vec::new();
        let new_layers_iter = self.layers.iter().filter(|v| v.id != new_id).map(|v| *v);

        for iter in new_layers_iter {
            new_layers.push(iter);
        }
        new_layers.push(CellLayer::new(new_id, new_val));

        if new_layers.len() + 1 > MAX_LAYERS_PER_CELL {
            return Err(format!(
                "Cell ID {} exceeds {}",
                new_id, MAX_LAYERS_PER_CELL
            ));
        }

        self.layers = new_layers;
        return Ok(());
    }
    pub fn update(self: &mut Self, layers: Vec<CellLayer>) -> Result<(), String> {
        self.layers = layers;
        return Ok(());
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
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
//#[derive(Debug, PartialEq, Clone)]
//#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    // for optimization reasons, rather than being column-ordered grid[x][y], the grid is
    // row-ordered grid[row][column] for views
    grid: Vec<Vec<MapCell>>, // having it as fixed 2D-array causes stack-overflow so had to switch to Vec<Vec<>>
    width: u16,
    height: u16,
    current_x: u16, // UpperLeft, for moving about the map (mainly for View)
    current_y: u16,
}

//impl Deserialize for Map {
//    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//    where
//        D: serde::Deserializer<'de> {
//        todo!()
//    }
//
//    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error>
//    where
//        D: serde::Deserializer<'de>,
//    {
//        // Default implementation just delegates to `deserialize` impl.
//        *place = try!(Deserialize::deserialize(deserializer));
//        Ok(())
//    }
//}
//
//impl Serialize for Map {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//        S: serde::Serializer,
//    {
//        let mut state = serializer.serialize_struct("Map", 5)?; // currently, only 2 fields
//        state.serialize_field("grid", &self.grid)?;
//        state.serialize_field("width", &self.width)?;
//        state.serialize_field("height", &self.height)?;
//        state.serialize_field("current_x", &self.current_x)?;
//        state.serialize_field("current_y", &self.current_y)?;
//        state.end()
//    }
//}
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
        if width == 0 {
            return Err("Width request was 0".to_owned());
        }
        if width as u16 > self.width {
            return Err(format!(
                "Width request was {}, but max width is {}",
                width, self.width
            ));
        }
        if map_y > self.height {
            return Err(format!(
                "Map Y {} exceeds the boundary of max height is {}",
                map_y, self.height
            ));
        }
        if map_x > self.width {
            return Err(format!(
                "Map X {} exceeds the boundary of max width is {}",
                map_x, self.width
            ));
        }
        let row = &self.grid[map_y as usize];
        let islice = row.iter().skip(map_x as usize).take(width as usize);

        let mut vslice: Vec<MapCell> = Vec::new();
        islice.for_each(|mc| vslice.push(mc.clone()));
        if vslice.len() == 0 {
            return Err("Somehow, the slice of requested dimension is empty!".to_owned());
        }
        return Ok(vslice); // clone since we don't have Copy
    }

    pub fn set(self: &mut Self, map_x: u16, map_y: u16, cell: MapCell) -> Result<(), String> {
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
        return Ok(());
    }
    pub fn set_view(
        self: &mut Self,
        view_offset_x: u8,
        view_offset_y: u8,
        cell: MapCell,
    ) -> Result<(), String> {
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
    ) -> Result<(), String> {
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
        return Ok(());
    }

    // convert 2D to single array strided
    pub fn build_view(
        self: &Self,
        view_offset_x: u8,
        view_offset_y: u8,
        view_width: u8,
        view_height: u8,
    ) -> Result<Vec<TcellValue>, String> {
        // Map:((5, 205)) - World:(5, 205) Cursor:(0, 0) Pos:(5, 205) Val:0 - Mouse:(1017, 618) - Keys:[PageDown]
        // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: "Map Y 205 exceeds the boundary of max height is 200"', src\map.rs:313:75
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
            if perRowSlice.len() != view_width as usize {
                return Err("Why did we not get the row?".to_owned());
            }
            slices.push(perRowSlice);
        }
        if slices.len() != view_height as usize {
            return Err("Why did we not get the view height?".to_owned());
        }

        let mut retSlicesFlatten: Vec<TcellValue> = Vec::new();
        for vRow in slices {
            let mut vals: Vec<TcellValue> = Vec::new();
            if (vRow.len() == 0) || (vRow.len() != view_width as usize) {
                return Err("Why did we not get the row?".to_owned());
            }
            for vc in vRow {
                let mut topMost: TcellValue = 0;
                if vc.layers.len() > 0 {
                    // this this be reversed iteratations where tail of the queue is the topmost?
                    topMost = vc.layers.first().unwrap().val; // we've already tested len() > 0, so safe to unwrap here
                }
                vals.push(topMost);
            }
            retSlicesFlatten.append(&mut vals);
        }

        return Ok(retSlicesFlatten);
    }

    // NOTE: There will NOT be any I/O here, we just transform it into serializable data format (for now, JSON)
    // and it will be up to the caller to I/O (persist) it
    pub fn serialize_for_save(self: &Self) -> Result<Vec<u8>, String> {
        let mut dest_buffer = Vec::new();
        match self.serialize(&mut rmp_serde::Serializer::new(&mut dest_buffer)) {
            Ok(s) => Ok(dest_buffer),
            Err(e) => Err(e.to_string()),
        }
    }

    // NOTE: For now, because we're deserializing from JSON, we receive it as String type
    pub fn deserialize_for_load(bin_data: &Vec<u8>) -> Result<Map, String> {
        //let inbuf = bin_data.as_slice();
        //let mut dest_buff = rmp_serde::Deserializer::from_slice(&mut inbuf);
        //match rmp_serde::from_slice(&mut bin_data.as_slice()) {
        //    Ok(m) => match rmp_serde::Deserializer::new(m) {
        //       zzz => zzz.,
        //    },
        //    Err(ee) => Err(ee.to_string()),
        //}
        let mut inbuf = bin_data.as_slice();
        let dest_buff: Result<Map, _> = rmp_serde::from_slice(&mut inbuf);
        //let des = rmp_serde::Deserializer::new(dest_buff.unwrap());
        match dest_buff {
            Ok(m) => Ok(m),
            Err(e) => Err(e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    //use core::slice::SlicePattern;

    use super::*;
    use serde_test::{assert_tokens, Token};

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
        let theResult = theMap.grid[posX as usize][posY as usize]
            .set(layerID, newVal)
            .unwrap(); // should throw with Unwrap()
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

    #[test]
    fn test_serialize_deserialize() {
        let mut theMap = Map::create(64, 128).unwrap(); // gotta make it mutable if we're going to allow update
        let posX = 0;
        let posY = 0;
        let layerID = 0;
        let newVal = 2;
        let theResult = theMap.grid[posX as usize][posY as usize]
            .set(layerID, newVal)
            .unwrap(); // should throw with Unwrap()

        let mut buf = Vec::new();
        theMap.serialize(&mut Serializer::new(&mut buf)).unwrap();

        let mut de = Deserializer::new(&buf[..]);
        let my_struct_deserialized = Map::deserialize(&mut de).unwrap();

        assert_eq!(theMap, my_struct_deserialized);
        assert_eq!(
            my_struct_deserialized.grid[posX as usize][posY as usize].layers[layerID as usize].val,
            newVal
        );
    }
}
