//extern crate serde;
use crate::entity_system::*;
use serde::Serialize;
use serde_derive::Deserialize;
//use crate::resource_system::Resource;

const MAX_MAP_WIDTH: usize = 1024;
const MAX_MAP_HEIGHT: usize = 1024;
const MAX_LAYERS_PER_CELL: usize = 16;

type TCellID = u8; // private

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct CellLayer {
    pub id: TCellID,
    pub entity: TEntityID, // Note that layer_weight is at the Entity level, though it may somewhat make sense that it's at map, but for collision and other relations, it's better that we just store EntityID here and let the entity_system handle all that...
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
    pub fn new(new_id: TCellID, new_val: TEntityID) -> CellLayer {
        CellLayer {
            id: new_id,
            entity: new_val,
        }
    }
    //pub fn save(self: &Self) -> Result<String, String> {
    //    let s = serde_json::to_string(self).unwrap();
    //    return Ok(s);
    //}
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
    pub fn set(self: &mut Self, new_id: u8, new_val: TEntityID) -> Result<(), String> {
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
//#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
                width as usize
            ];
            height as usize// per row
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
        if map_x > self.width {
            return Err(format!(
                "Cannot assign MapCell to position ([{}], {}) for its X position exceeds {}",
                map_x, map_y, self.width
            ));
        }
        if map_y > self.height {
            return Err(format!(
                "Cannot assign MapCell to position ({}, [{}]) for its Y position exceeds {}",
                map_x, map_y, self.height
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
        if map_x > self.width {
            return Err(format!(
                "Cannot assign MapCell to position ([{}], {}) for its X position exceeds {}",
                map_x, map_y, self.width
            ));
        }
        if map_y > self.height {
            return Err(format!(
                "Cannot assign MapCell to position ({}, [{}]) for its Y position exceeds {}",
                map_x, map_y, self.height
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
        if map_y as usize > self.height as usize {
            return Err(format!(
                "Cannot assign row of MapCells to position ({}, [{}]) for its Y position exceeds {}",
                x_offset, map_y, self.height
            ));
        }
        if (x_offset as usize + row_slice.len()) > self.width as usize {
            return Err(format!(
                "Cannot assign row of MapCells to position ([{}], {}) for its X position exceeds {}",
                x_offset, map_y, self.width
            ));
        }
        for i in 0..(row_slice.len() - 1) {
            self.set(
                x_offset as u16 + i as u16,
                map_y as u16,
                row_slice[i].clone(),
            )
            .unwrap();
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
    ) -> Result<Vec<Option<TEntityID>>, String> {
        // Map:((5, 205)) - World:(5, 205) Cursor:(0, 0) Pos:(5, 205) Val:0 - Mouse:(1017, 618) - Keys:[PageDown]
        // thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: "Map Y 205 exceeds the boundary of max height is 200"', src\map.rs:313:75
        let map_x = self.current_x + view_offset_x as u16;
        let map_y = self.current_y + view_offset_y as u16;
        if map_x > self.width {
            return Err(format!(
                "ViewXTop={} exceeds max width dimension {}",
                map_x, self.width
            ));
        }
        if map_y > self.height {
            return Err(format!(
                "ViewYTop={} exceeds max height dimension {}",
                map_y, self.height
            ));
        }
        if (map_x as usize + view_width as usize) > self.width as usize {
            return Err(format!(
                "ViewXBottom={} (width={}) exceeds max width dimension {}",
                map_x + view_width as u16,
                view_width,
                self.width
            ));
        }
        if (map_y as usize + view_height as usize) > self.height as usize {
            return Err(format!(
                "ViewYBottom={} (height={}) exceeds max height dimension {}",
                map_y + view_height as u16,
                view_height,
                self.height
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
        for h_index in 0..view_height {
            let per_row_slice = self
                .get_cell_row(map_x, map_y + h_index as u16, view_width)
                .unwrap();
            if per_row_slice.len() != view_width as usize {
                return Err("Why did we not get the row?".to_owned());
            }
            slices.push(per_row_slice);
        }
        if slices.len() != view_height as usize {
            return Err("Why did we not get the view height?".to_owned());
        }

        let mut ret_slices_flatten: Vec<Option<TEntityID>> = Vec::new();
        for row in slices {
            let mut vals: Vec<Option<TEntityID>> = Vec::new();
            if (row.len() == 0) || (row.len() != view_width as usize) {
                return Err("Why did we not get the row?".to_owned());
            }
            for vc in row {
                let mut top_most: Option<TEntityID> = None;
                if vc.layers.len() > 0 {
                    // this this be reversed iteratations where tail of the queue is the topmost?
                    // TODO: Sort by layer-weights and make the lightest weight the topmost instead of pulling the first()
                    top_most = Some(vc.layers.first().unwrap().entity); // we've already tested len() > 0, so safe to unwrap here
                }
                vals.push(top_most);
            }
            ret_slices_flatten.append(&mut vals);
        }

        return Ok(ret_slices_flatten);
    }

    // NOTE: There will NOT be any I/O here, we just transform it into serializable data format (for now, JSON)
    // and it will be up to the caller to I/O (persist) it
    pub fn serialize_for_save(self: &Self) -> Result<Vec<u8>, String> {
        if cfg!(debug_assertions) {
            println!(
                "Begin Serializing...  Map width:{}, Map height:{}",
                self.width, self.height
            )
        }
        let mut dest_buffer = Vec::new();
        // serde::serialize()
        return match self.serialize(&mut rmp_serde::Serializer::new(&mut dest_buffer)) {
            Ok(_s) => {
                if cfg!(debug_assertions) {
                    let _cl = dest_buffer.clone();
                    let _zzz = _cl.iter();
                    let _xxx = _zzz.clone().take(8).to_owned();
                    let _xxx2 = _zzz.clone().rev().take(8).rev().to_owned();
                    let _h: Vec<&u8> = _xxx.collect();
                    let _t: Vec<&u8> = _xxx2.collect();
                    println!(
                        "Serialization: {} bytes -> {:?} ... {:?} - (S)",
                        dest_buffer.len(),
                        _h,
                        _t
                    );
                }

                Ok(dest_buffer.to_owned())
            }
            Err(e) => Err(e.to_string()),
        };
    }

    // NOTE: For now, because we're deserializing from JSON, we receive it as String type
    pub fn deserialize_for_load(bin_data: &Vec<u8>) -> Result<Map, String> {
        if bin_data.len() == 0 {
            return Err("bin_data buffer is 0 bytes".to_owned());
        }

        if cfg!(debug_assertions) {
            println!("Begin Deserializing...");
            let _cl = bin_data.clone();
            let _zzz = _cl.iter();
            let _xxx = _zzz.clone().take(8).to_owned();
            let _xxx2 = _zzz.clone().rev().take(8).rev().to_owned();
            let _h: Vec<&u8> = _xxx.collect();
            let _t: Vec<&u8> = _xxx2.collect();
            println!(
                "Deserialization: {} bytes -> {:?} ... {:?} - (D)",
                bin_data.len(),
                _h,
                _t
            );
        }
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
        return match dest_buff {
            Ok(m) => {
                if cfg!(debug_assertions) {
                    println!(
                        "Deserialization: Map width:{:?}, Map height:{:?}",
                        m.width, m.height
                    );
                }
                Ok(m)
            }
            Err(e) => Err(e.to_string()),
        };
    }

    pub fn auto_generate(self: &Self) -> Result<Map, String> {
        // TODO: based on map dimension, update grid[][]
        return Err("CODE ME!".to_owned());
    }

    pub fn load(_file_path: &String) -> Result<Map, String> {
        return Err("CODE ME!".to_owned());
    }
    pub fn save(self: &Self, _file_path: &String) -> Result<Map, String> {
        return Err("CODE ME!".to_owned());
    }
}

#[cfg(test)]
mod tests {
    use crate::resource_system::Resource;

    use super::*;
    //use serde_test::{assert_tokens, Token};

    #[test]
    fn create64x128() {
        let the_map = Map::create(64, 128).unwrap();
        assert_eq!(the_map.grid[0][0].layers.len(), 0); // when freshly creaed, each/any layers are empty
    }

    #[test]
    fn can_update_layer() {
        let mut the_map = Map::create(64, 128).unwrap(); // gotta make it mutable if we're going to allow update
        let pos_x = 0;
        let pos_y = 0;
        let layer_id = 0;
        let new_entity_id = 5u16; // faking call to entity_system::add_entity(0, 0).unwrap();
        let _the_result = the_map.grid[pos_y as usize][pos_x as usize]
            .set(layer_id, new_entity_id)
            .unwrap(); // should throw with Unwrap()
        assert_eq!(
            the_map.grid[pos_y as usize][pos_x as usize].layers[layer_id as usize].entity,
            new_entity_id
        );
    }

    #[test]
    fn test_set_map_row() {
        let mut the_map = Map::create(16, 32).unwrap();
        let map_y = 8;
        let x_offset = 4;
        let row_slice = the_map.get_cell_row(x_offset, map_y, 8).unwrap();
        the_map
            .set_row(map_y as u16, x_offset as u8, row_slice)
            .unwrap();
    }

    #[test]
    fn test_view_for_rendering() {
        let map_width = 16;
        let map_height = 32;
        let the_map = Map::create(map_width, map_height).unwrap();
        let view_x = 2;
        let view_y = 2;
        let view_width = 4;
        let view_height = 8;
        let view = the_map
            .build_view(view_x, view_y, view_width, view_height)
            .unwrap();
        assert_ne!(0, view.len());
        for h in 0..view_height as u16 {
            for w in 0..view_width as u16 {
                let i = ((h * view_width as u16) + w) as usize;
                print!(
                    "{}",
                    match view[i] {
                        Some(v) => v,
                        None => 0,
                    }
                );
            }
            println!("");
        }
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut the_map = Map::create(64, 128).unwrap(); // gotta make it mutable if we're going to allow update
        let pos_x = 3;
        let pos_y = 5;
        let layer_id = 0;
        let new_val = 666;
        let _the_result = the_map.grid[pos_y as usize][pos_x as usize]
            .set(layer_id, new_val)
            .unwrap(); // should throw with Unwrap()

        // serialize to buffer
        let my_map_serialized = the_map.serialize_for_save().unwrap();

        // deserialize the buffer that was just serialized
        let my_map_deserialized = Map::deserialize_for_load(&my_map_serialized).unwrap();

        // should be about to just do struct equate
        assert_eq!(the_map, my_map_deserialized);

        // but just in case, we'll also check that the value we set is valid...
        assert_eq!(
            my_map_deserialized.grid[pos_y as usize][pos_x as usize].layers[layer_id as usize]
                .entity,
            new_val
        );
    }

    #[test]
    fn test_serialize_deserialize_io() {
        // lambdas (closures) for I/O
        //let write_data = |fname: &String, the_map: &Map| match the_map.serialize_for_save() {
        //    Ok(result_map) => {
        //        println!("Serialized {} bytes, begin writing...", result_map.len());
        //        let file = File::create(fname)?;
        //        let mut writer = BufWriter::new(file);
        //        //let mut serializer = rmp_serde::Serializer::new(&mut writer);
        //        //let _ = result_map.serialize(&mut serializer).unwrap(); // Result<(), std::error::Error>
        //        match writer.write_all(&result_map) {
        //            Ok(()) => {
        //                let ret_result: Result<Vec<u8>, Box<dyn std::error::Error>> =
        //                    Ok(result_map.to_owned());
        //                ret_result
        //            }
        //            Err(e) => Err::<Vec<u8>, Box<dyn std::error::Error>>(Box::new(e)),
        //        }
        //    }
        //    Err(e) => {
        //        println!("{}", e.to_string());
        //        let ret = Err("Call to Map::serialize_for_save failed".into());
        //        ret
        //    }
        //};
        //let read_data = |fname: &String| {
        //    let file = File::open(fname)?;
        //    let mut reader = BufReader::new(file);
        //    //let file = File::open(fname)?;
        //    //let reader = BufReader::new(file);
        //    //let mut deserializer = Deserializer::new(reader);
        //    //let result: Map = serde::Deserialize::deserialize(&mut deserializer)?;
        //    //return Ok(result);
        //    //let mut deserializer = rmp_serde::Deserializer::new(reader);
        //    //let result: Map = serde::Deserialize::deserialize(&mut deserializer)?;
        //    //let ret: Result<Map, Box<dyn std::error::Error>> = Ok(result);
        //    // read the whole file
        //    let mut vec_buffer = Vec::new();
        //    match reader.read_to_end(&mut vec_buffer) {
        //        Ok(_buff_size) => {
        //            let bin_buffer = vec_buffer.to_owned();
        //            let map_from_serialized_data = Map::deserialize_for_load(&bin_buffer).unwrap();
        //            Ok::<Map, Box<dyn std::error::Error>>(map_from_serialized_data)
        //        }
        //        Err(e) => Err::<Map, Box<dyn std::error::Error>>(Box::new(e)),
        //    }
        //};

        let unit_test_bin_file = "./unit_test_serde.bin".to_owned();
        match std::fs::remove_file(unit_test_bin_file.clone()) {
            Ok(()) => println!(
                "File '{}' deleted prior to beginning the unit-test",
                unit_test_bin_file
            ),
            Err(_e) => (),
        }

        let mut res = match Resource::new(unit_test_bin_file.clone(), false) {
            Ok(res_id) => Resource::try_get(res_id).unwrap(),
            Err(_e) => {
                Resource::try_get(Resource::create(unit_test_bin_file.clone(), true).unwrap())
                    .unwrap()
            }
        };
        let mut the_map = Map::create(64, 128).unwrap(); // gotta make it mutable if we're going to allow update
        let pos_x = 3;
        let pos_y = 5;
        let layer_id = 0;
        let new_val = 666;
        let _the_result = the_map.grid[pos_y as usize][pos_x as usize]
            .set(layer_id, new_val)
            .unwrap(); // should throw with Unwrap()

        // serialize to buffer
        let _rs = res.write_data(|| the_map.serialize_for_save());
        let my_map_serialized = res.write_data(|| the_map.serialize_for_save()).unwrap();
        assert_ne!(0, my_map_serialized.len());
        let map_from_serialized_data = Map::deserialize_for_load(&my_map_serialized).unwrap();
        assert_eq!(the_map, map_from_serialized_data);

        // deserialize the buffer that was just serialized
        let my_map_deserialized = res.read_data(Map::deserialize_for_load).unwrap();

        std::fs::remove_file(unit_test_bin_file.clone()).unwrap();

        // should be about to just do struct equate
        assert_eq!(the_map, my_map_deserialized);

        // but just in case, we'll also check that the value we set is valid...
        assert_eq!(
            my_map_deserialized.grid[pos_y as usize][pos_x as usize].layers[layer_id as usize]
                .entity,
            new_val
        );
    }
}
