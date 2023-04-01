include!(concat!(env!("OUT_DIR"), "/hello.rs"));
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::{
    error::Error,
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    thread, time,
};

// see build.rs
pub use lib_tower_defense::map::{CellLayer, Map, MapCell, TCellValue};

const SAMPLE_VIEW_WIDTH: u8 = 80;
const SAMPLE_VIEW_HEIGHT: u8 = 40;
const SAMPLE_MAP_WIDTH: u16 = SAMPLE_VIEW_WIDTH as u16 * 5;
const SAMPLE_MAP_HEIGHT: u16 = SAMPLE_VIEW_HEIGHT as u16 * 5;
const LAYER_CHARS: [u8; 16] = *b"o.,-~:;&=!*[<#$@"; // for debuggin, replace ' ' (space) with '.' if needed
const STR_ESCAPE: &str = "\x1b";

fn render(view: Vec<i64>, cursor_x: u8, cursor_y: u8, status: String) {
    // NOTE: In general, using ncurses is the way to go, but unfortunately,
    // it's not available to Windows, so I'll be using ANSI cursor (XTerm, not VT100)
    // to deal with all the trivial rendering
    let set_cursor = |col: i32, row: i32| {
        print!("{}[{};{}H", STR_ESCAPE, row, col); // oddly, ANSI uses row,col (Y,X) instead of (X,Y)
    };
    let print_at = |col: i32, row: i32, s: &str| {
        set_cursor(col, row);
        print!("{}", s);
    };
    let str_boldface = |ch: char| {
        print!("{}[1m{}{}[0m", STR_ESCAPE, ch, STR_ESCAPE);
    };
    let str_reverse = |ch: char| {
        print!("{}[7m{}{}[0m", STR_ESCAPE, ch, STR_ESCAPE);
    };
    let str_home = || {
        return format!("{}[H\n", STR_ESCAPE); // ESC[H = move cursor back to HOME position (done per each gameloop) and add 1 line down for status
    };
    let str_cls = || {
        return format!("{}[2J", STR_ESCAPE); // ESC[2J = clear entire screen (CLS) because we need to clear last scroll image (also removes last cursor position)
    };

    let cursor_char = format!("{}[47m {}[0m", STR_ESCAPE, STR_ESCAPE);
    print!("{}", str_cls());
    print_at(0, SAMPLE_VIEW_HEIGHT as i32 + 2, &status);
    print!("{}", str_home());
    let view_x_offset = 1;  // shift one right, so we can have a border on left edge
    let view_y_offset = 3;  // shift 3 down so we can place stats and border at top

    for h in 0..SAMPLE_VIEW_HEIGHT {
        for w in 0..SAMPLE_VIEW_WIDTH {
        set_cursor((w + view_x_offset) as i32, (h + view_y_offset) as i32);
            let flattened_view_index = (h as usize * SAMPLE_VIEW_WIDTH as usize) + w as usize; // NOTE: will get overflow if you do not explicitly cast here

            // for now, just assume the val data is the index to the array...
            let v = view[flattened_view_index] as usize;
            let ch: char = LAYER_CHARS[v % LAYER_CHARS.len()] as char;
            if h == cursor_y && w == cursor_x {
                if ch == ' ' {
                    print!("{}", cursor_char);
                } else {
                    str_reverse(ch);
                }
            } else {
                print!("{}", ch);
            }
        }
    }
    println!("");
}

fn write_data(fname: &String, the_map: &Map) -> Result<Vec<u8>, Box<dyn Error>> {
    match the_map.serialize_for_save() {
        Ok(result_map) => {
            println!("Serialized {} bytes, begin writing...", result_map.len());
            let file = File::create(fname)?;
            let mut writer = BufWriter::new(file);
            match writer.write_all(&result_map) {
                Ok(()) => {
                    let ret_result: Result<Vec<u8>, Box<dyn std::error::Error>> =
                        Ok(result_map.to_owned());
                    ret_result
                }
                Err(e) => Err::<Vec<u8>, Box<dyn std::error::Error>>(Box::new(e)),
            }
        }
        Err(e) => {
            println!("{}", e.to_string());
            let ret = Err("Call to Map::serialize_for_save failed".into());
            ret
        }
    }
}

fn read_data(fname: &String) -> Result<Map, Box<dyn Error>> {
    //let file = File::open(fname)?;
    //let reader = BufReader::new(file);
    //let mut deserializer = Deserializer::new(reader);
    //let result: Map = serde::Deserialize::deserialize(&mut deserializer)?;
    //return Ok(result);

    let file = File::open(fname)?;
    let mut reader = BufReader::new(file);
    // read the whole file
    let mut vec_buffer = Vec::new();
    match reader.read_to_end(&mut vec_buffer) {
        Ok(_buff_size) => {
            let bin_buffer = vec_buffer.to_owned();
            let map_from_serialized_data = Map::deserialize_for_load(&bin_buffer).unwrap();
            Ok::<Map, Box<dyn std::error::Error>>(map_from_serialized_data)
        }
        Err(e) => Err::<Map, Box<dyn std::error::Error>>(Box::new(e)),
    }
}

#[derive(PartialEq)] // need PartialEq so we can test outside the 'match{}' block via 'if' statement
enum BreakLoopType {
    NoBreak = 0,
    QuitWithoutSave = 1,
    SaveAndExit = 2,
    ApplicationError,
}

fn main() {
    print!("\x1b[2J"); // ESC[2J = clear entire screen (CLS)
    print!("main (text view): Cursor keys, PgUp, PgDn, '[', ']', 'space', 'Q', and Esc");
    //let ms = time::Duration::from_millis(300); // 1/3 second
    let ms = time::Duration::from_millis(50);
    let file_paths = "./test.save.bin".to_owned();
    let mut the_map = match read_data(&file_paths) {
        Ok(m) => {
            if m.get_width() != SAMPLE_MAP_WIDTH {
                panic!("Invalid data");
            }
            if m.get_height() != SAMPLE_MAP_HEIGHT {
                panic!("Invalid data");
            }
            Ok(m)
        }
        Err(io_error) => {
            // if file exists, throw a panic!(), else assume bin data does not exist, and create a brand new map
            println!("Error: {}", io_error.to_string());

            match io_error.downcast_ref::<io::Error>() {
                Some(io_casted_error) => match io_casted_error.kind() {
                    io::ErrorKind::NotFound => Map::create(SAMPLE_MAP_WIDTH, SAMPLE_MAP_HEIGHT),
                    _ => Err(io_error.to_string()),
                },
                _ => Err(io_error.to_string()),
            }
        }
    }
    .unwrap();

    let mut view_x: u16 = 0;
    let mut view_y: u16 = 0;
    let mut cursor_x: u8 = 0;
    let mut cursor_y: u8 = 0;
    let move_step_x: u8 = 5;
    let move_step_y: u8 = 5;
    let device_state = DeviceState::new();

    let cell = MapCell {
        layers: vec![CellLayer {
            id: 1,
            val: 'x' as lib_tower_defense::map::TCellValue,
        }],
    };
    for y in 0..32 {
        for x in 0..32 {
            the_map.set(x, y, cell.clone()).unwrap();
        }
    }

    let mut break_loop = BreakLoopType::NoBreak;
    loop {
        let view = the_map
            .build_view(0, 0, SAMPLE_VIEW_WIDTH, SAMPLE_VIEW_HEIGHT)
            .unwrap();
        if view.len() == 0 {
            break_loop = BreakLoopType::ApplicationError;
        }

        let mouse = device_state.get_mouse();
        let keys: Vec<Keycode> = device_state.get_keys();
        let test_keys = keys.clone();
        if !keys.is_empty() {
            for k in keys {
                match k {
                    Keycode::Escape => {
                        // Prompt to save data
                        println!("escape");
                        break_loop = BreakLoopType::SaveAndExit;
                        break;
                    }
                    Keycode::Q => {
                        // Prompt to quit without save
                        println!("quit");
                        break_loop = BreakLoopType::QuitWithoutSave;
                        break;
                    }
                    Keycode::PageUp => {
                        if view_y > move_step_y as u16 {
                            view_y = view_y - move_step_y as u16;
                        }
                    }
                    Keycode::Up => {
                        if cursor_y > 0 {
                            cursor_y = cursor_y - 1;
                        } else if view_y > move_step_y as u16 {
                            // rather than moving the cursor up, we'd scroll UP because we're at the edge
                            view_y = view_y - move_step_y as u16;
                        }
                    }
                    Keycode::PageDown => {
                        let top_y = view_y + move_step_y as u16;
                        let bot_y = top_y + SAMPLE_VIEW_HEIGHT as u16;

                        if bot_y < the_map.get_height() {
                            view_y = top_y;
                        }
                    }
                    Keycode::Down => {
                        let top_y = view_y + move_step_y as u16;
                        let bot_y = top_y + SAMPLE_VIEW_HEIGHT as u16;

                        if cursor_y < SAMPLE_VIEW_HEIGHT {
                            cursor_y = cursor_y + 1;
                        } else if bot_y < the_map.get_height() {
                            view_y = top_y;
                        }
                    }
                    Keycode::LeftBracket => {
                        if view_x > move_step_x as u16 {
                            view_x = view_x - move_step_x as u16;
                        }
                    }
                    Keycode::Left => {
                        if cursor_x > 0 {
                            cursor_x = cursor_x - 1;
                        } else if view_x > move_step_x as u16 {
                            // rather than moving the cursor left, we'd scroll view left
                            view_x = view_x - move_step_x as u16;
                        }
                    }
                    Keycode::RightBracket => {
                        let left_x = view_x + move_step_x as u16;
                        let right_x = left_x + SAMPLE_VIEW_WIDTH as u16;

                        if right_x < the_map.get_width() {
                            view_x = left_x;
                        }
                    }
                    Keycode::Right => {
                        let left_x = view_x + move_step_x as u16;
                        let right_x = left_x + SAMPLE_VIEW_WIDTH as u16;

                        if cursor_x < SAMPLE_VIEW_WIDTH {
                            cursor_x = cursor_x + 1;
                        } else if right_x < the_map.get_width() {
                            view_x = left_x;
                        }
                    }
                    Keycode::Home => {
                        view_x = 0;
                        view_y = 0;
                        cursor_x = 0;
                        cursor_y = 0;
                    }
                    Keycode::End => {
                        cursor_x = SAMPLE_VIEW_WIDTH - 0;
                        cursor_y = SAMPLE_VIEW_HEIGHT - 0;
                        view_x = SAMPLE_MAP_WIDTH - 0;
                        view_y = SAMPLE_MAP_HEIGHT - 0;
                    }
                    Keycode::Space => {
                        let pos_x = view_x + cursor_x as u16;
                        let pos_y = view_y + cursor_y as u16;
                        let mut cursor_position_cell =
                            match the_map.get_cell(pos_x, pos_y).unwrap().first().is_some() {
                                true => the_map.get_cell(pos_x, pos_y).unwrap(),
                                false => MapCell {
                                    layers: vec![CellLayer::new(0, 0); 1],
                                },
                            };

                        for layer in cursor_position_cell.layers.clone() {
                            let temp_cycle_the_value_on_space =
                                match (layer.val + 1) as usize > LAYER_CHARS.len() {
                                    false => layer.val + 1,
                                    true => 0,
                                };
                            // Note: set() should add if missing, but in this case, we're iterating through existing Layers, so it assumes it's always an update/repleace
                            cursor_position_cell
                                .set(layer.id, temp_cycle_the_value_on_space)
                                .unwrap();
                        }

                        the_map.set(pos_x, pos_y, cursor_position_cell).unwrap();
                    }
                    _ => (),
                }
            }
            // update map position based on key pressed
            the_map.set_upper_left(view_x, view_y);
        }

        // for now, only update text if key is pressed
        let the_val = match the_map
            .get_cell(view_x + cursor_x as u16, view_y + cursor_y as u16)
            .unwrap()
            .first()
        {
            Some(v) => v.val,
            None => 0,
        };
        let mut keys_input = "[".to_owned();
        for k in test_keys {
            keys_input.push_str(&k.to_string());
        }
        keys_input.push_str("]");
        let status =
        format!(
            "\x1b[HMap:{:?} - World:({}, {}) Cursor:({}, {}) Pos:({}, {}) Val:{} - Mouse:{:?} - Keys:{}                    ",
            the_map.get_upper_left(),
            view_x,
            view_y,
            cursor_x,
            cursor_y,
            view_x + cursor_x as u16,
            view_y + cursor_y as u16,
            the_val,
            mouse.coords, keys_input
        );
        // sleep mainly so that we can yield the app and let other processes run...
        thread::sleep(ms);
        match break_loop {
            BreakLoopType::QuitWithoutSave => break,
            BreakLoopType::SaveAndExit => {
                // update data and quit
                write_data(&file_paths, &the_map).unwrap();
                break;
            }
            BreakLoopType::ApplicationError => break,
            BreakLoopType::NoBreak => (),
        }
        render(view, cursor_x, cursor_y, status);
    }
    
}
