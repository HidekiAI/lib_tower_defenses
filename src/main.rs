include!(concat!(env!("OUT_DIR"), "/hello.rs")); // see build.rs
use device_query::{DeviceQuery, DeviceState, Keycode};
//use lib_tower_defense::{entity_system, resource_system, sprite_system};
use lib_tower_defense::{entity_system, sprite_system};

use std::{
    collections::HashSet,
    io,
    process::{Command, Output},
    thread, time,
};
use terminal_size::{terminal_size, Height, Width}; // need this to clear screen logically, since we cannot call 'tput cols' and 'tput lines'

pub use lib_tower_defense::entity_system::*;
pub use lib_tower_defense::map::*;
pub use lib_tower_defense::resource_system::*;
pub use lib_tower_defense::sprite_system::*;

const SAMPLE_VIEW_WIDTH: u8 = 80;
const SAMPLE_VIEW_HEIGHT: u8 = 40;
const SAMPLE_MAP_WIDTH: u16 = SAMPLE_VIEW_WIDTH as u16 * 5;
const SAMPLE_MAP_HEIGHT: u16 = SAMPLE_VIEW_HEIGHT as u16 * 5;
const LAYER_CHARS: [u8; 16] = *b" .,-~:;&=!*[#QW@"; // for debuggin, replace ' ' (space) with '.' if needed
const STR_ESCAPE: &str = "\x1b";

// NOTE: Probably not a useful function because on Windows, it defaults to PowerShell and so doing commands like 'ls -ltArh'
// will not work.  Also, we cannot tell if bash.exe is installed on the Windows,  and even so, passing as command with args:
// "bash -c 'ls -ltArh'" will not work (PowerShell will never be able to locate bash.exe)
// also, recommend doing the '#[cfg(target_os = "linux")]' and '#[cfg(not(target_os = "linux"))]' to
// conditionally call one route for linux/bash and other for powershell...
fn _do_process(str_cmd: &str) -> Result<(String, String), String> {
    //let output = Command::new(str_cmd[0])
    //    .arg(str_cmd[1])
    //    .output()
    //    .expect("Failed to execute command ");
    //let result = String::from_utf8(output.stdout).unwrap();
    //println!("Result: {}", result);

    //let str_cmd : &str= "tput cols";
    let mut cmd = Command::new(str_cmd);
    match cmd.output() {
        Ok(Output {
            stderr: _cmd_errors,
            stdout: cmd_outputs,
            status: exit_status,
        }) => {
            if exit_status.success() {
                //println!("{:?}", out);
                match String::from_utf8(cmd_outputs.clone()) {
                    Ok(res) => {
                        let ss = format!("{:?}", cmd_outputs);
                        return Ok((res, ss));
                    }
                    Err(why) => Err(format!("error converting result to utf8: {}", why)),
                }
            } else {
                Err(format!("error executing command '{}'", str_cmd))
            }
        }
        Err(why) => Err(format!("error executing command '{}': {}", str_cmd, why)),
    }
}
fn get_term_dimension() -> (i32, i32) {
    //#[cfg(target_os = "linux")] // 'bash -c' won't work on Windows
    //let (cols, _) = do_process("bash -c '/usr/bin/tput cols'").unwrap();
    //let (lines, _) = do_process("bash -c '/usr/bin/tput lines'").unwrap();
    //let i_cols = cols.trim().to_uppercase().parse::<i32>().unwrap();
    //let i_lines = lines.trim().to_uppercase().parse::<i32>().unwrap();

    let size = terminal_size();
    if let Some((Width(w), Height(h))) = size {
        println!("Your terminal is {} cols wide and {} lines tall", w, h);
        return (w as i32, h as i32);
    } else {
        println!("Unable to get terminal size");
        return (SAMPLE_VIEW_WIDTH as i32, SAMPLE_VIEW_HEIGHT as i32);
    }
}
fn make_line_ch(line_width: u8, line_char: char) -> String {
    return std::iter::repeat(line_char)
        .take(line_width as usize)
        .collect::<String>();
}
fn make_line_str(line_width: u8, line_char: String) -> String {
    return std::iter::repeat(line_char)
        .take(line_width as usize)
        .collect::<String>();
}
fn make_line_from_slice(char_line: &[char]) -> String {
    return String::from_utf8(char_line.iter().map(|&x| x as u8).collect()).unwrap();
}
fn home_screen() -> () {
    //let str_home = || {
    //    return format!("{}[H", STR_ESCAPE); // ESC[H = move cursor back to HOME position (done per each gameloop) and add 1 line down for status
    //};
    print!("{}[H", STR_ESCAPE); // ESC[H = move cursor back to HOME position (done per each gameloop) and add 1 line down for status
}
// NOTE: the '[2J' mode forces screen to scroll down, so do NOT use it in a loop
fn clear_screen() -> () {
    //let str_cls = || {
    //    return format!("{}[2J", STR_ESCAPE); // ESC[2J = clear entire screen (CLS) because we need to clear last scroll image (also removes last cursor position)
    //};

    print!("{}[2J", STR_ESCAPE); // ESC[2J = clear entire screen (CLS) because we need to clear last scroll image (also removes last cursor position)
    home_screen();
    let (cols, lines) = get_term_dimension();
    let clear_char = format!(
        "{}[48;5;235m {}[0m", // dark-grey (not black, so that it's easier to debug)
        STR_ESCAPE, STR_ESCAPE
    );
    let blank_line_str = make_line_str(cols as u8, clear_char);

    let _blank_line = make_line_ch(cols as u8, ' ');
    //let debug_line = make_line(cols as u8 - 2, '.');
    for _ in 0..lines {
        println!("{}", blank_line_str)
        //println!("^{}$", debug_line)
    }
    home_screen();
}

fn render(view: Vec<Option<TEntityID>>, cursor_x: u8, cursor_y: u8, status: String) {
    // NOTE: In general, using ncurses is the way to go, but unfortunately,
    // it's not available to Windows, so I'll be using ANSI cursor (XTerm, not VT100)
    // to deal with all the trivial rendering
    let set_cursor = |col_1based: i32, row_1based: i32| {
        // NOTE: both row and col are 1-based
        print!("{}[{};{}H", STR_ESCAPE, row_1based, col_1based); // oddly, ANSI uses row,col (Y,X) instead of (X,Y)
    };
    let print_at = |col: i32, row: i32, s: &String| {
        set_cursor(col, row);
        print!("{}", s);
    };
    let _print_ch_at = |col: i32, row: i32, ch: &char| {
        set_cursor(col, row);
        print!("{}", ch);
    };
    let _str_boldface = |ch: char| -> String {
        return format!("{}[1m{}{}[0m", STR_ESCAPE, ch, STR_ESCAPE);
    };
    let str_reverse = |ch: char| -> String {
        return format!("{}[7m{}{}[0m", STR_ESCAPE, ch, STR_ESCAPE);
    };

    let clear_screen_with_char = |_w: i32, _h: i32, clchar: char, border_chars_lrtb: [char; 4]| {
        let chline: [char; SAMPLE_VIEW_WIDTH as usize] = [clchar; SAMPLE_VIEW_WIDTH as usize];
        let _clear_line_for_view = make_line_from_slice(&chline);
        let clear_line = make_line_from_slice(&chline);
        let top_line =
            make_line_from_slice(&[border_chars_lrtb[2]; SAMPLE_VIEW_WIDTH as usize + 2]); // +2 to adjust for left+right borders
        let bot_line =
            make_line_from_slice(&[border_chars_lrtb[3]; SAMPLE_VIEW_WIDTH as usize + 2]); // +2 to adjust for left+right borders
                                                                                           //clear_screen(); // first, clear the entire xterm
        home_screen();
        for _ in 0..SAMPLE_VIEW_HEIGHT {
            println!(
                "{}{}{}",
                border_chars_lrtb[0], clear_line, border_chars_lrtb[1]
            );
        }
        home_screen();
        println!("{}", top_line);

        println!("{}", bot_line);
    };

    //┌───┐
    //│   │
    //└───┘
    //'┌', '─', '┐', '│', '└', '┘'
    clear_screen_with_char(
        SAMPLE_VIEW_WIDTH as i32,
        SAMPLE_VIEW_HEIGHT as i32,
        ' ',
        ['│', '│', '─', '─'],
    );

    // see colorization chart on https://en.wikipedia.org/wiki/ANSI_escape_code
    let cursor_char = format!("{}[47m^{}[0m", STR_ESCAPE, STR_ESCAPE);
    print_at(0, SAMPLE_VIEW_HEIGHT as i32 + 4, &status);
    home_screen();
    let view_x_offset = 1; // shift one right, so we can have a border on left edge
    let view_y_offset = 1; // shift 3 down so we can place stats and border at top

    for h_index in 0..SAMPLE_VIEW_HEIGHT {
        for w_index in 0..SAMPLE_VIEW_WIDTH {
            let flattened_view_index =
                (h_index as usize * SAMPLE_VIEW_WIDTH as usize) + w_index as usize; // NOTE: will get overflow if you do not explicitly cast here
            let val_from_view = match view[flattened_view_index] {
                Some(vfv) => vfv as usize,
                None => 0,
            };
            let ch: char = LAYER_CHARS[val_from_view % LAYER_CHARS.len()] as char;

            // NOTE: cursor postion is 1-based
            let view_render_y = h_index + 1;
            let view_render_x = w_index + 1;
            // for now, just assume the val data is the index to the array...
            if view_render_y == cursor_y && view_render_x == cursor_x {
                if val_from_view == 0 {
                    print_at(
                        (view_render_x + view_x_offset) as i32,
                        (view_render_y + view_y_offset) as i32,
                        &cursor_char,
                    );
                } else {
                    print_at(
                        (view_render_x + view_x_offset) as i32,
                        (view_render_y + view_y_offset) as i32,
                        &str_reverse(ch),
                    );
                }
            } else if val_from_view == 0 {
                let dark_grey = format!(
                    "{}[48;5;234m {}[0m", // dark-grey (not black, so that it's easier to debug)
                    STR_ESCAPE, STR_ESCAPE
                );
                print_at(
                    (view_render_x + view_x_offset) as i32,
                    (view_render_y + view_y_offset) as i32,
                    &dark_grey,
                );
            } else {
                let ch_as_int_mod_9 = format!(
                    "{}[48;5;{}m{}{}[0m", //
                    STR_ESCAPE,
                    (val_from_view as i32 + view_render_y as i32) % 230,
                    ch,
                    STR_ESCAPE
                );
                print_at(
                    (view_render_x + view_x_offset) as i32,
                    (view_render_y + view_y_offset) as i32,
                    &ch_as_int_mod_9,
                );
            }
        }
    }
    //println!("");
    home_screen();
}

#[derive(PartialEq)] // need PartialEq so we can test outside the 'match{}' block via 'if' statement
enum BreakLoopType {
    NoBreak = 0,
    QuitWithoutSave = 1,
    SaveAndExit = 2,
    ApplicationError,
}
fn make_fake_sprite_resource() -> TResourceID {
    let temp_sprite_resource_id = 42;

    return temp_sprite_resource_id;
}
fn make_fake_sprites(temp_sprite_resource_id: TResourceID) -> HashSet<TSpriteSubGroupID> {
    return sprite_system::add(&temp_sprite_resource_id, deserialize_sprite).unwrap();
}
fn deserialize_sprite(temp_sprite_resource_id: &TResourceID) -> Vec<Sprite> {
    return Vec::<Sprite>::new(); // NO sprites for TUI version, return empty list?
}

fn main() {
    clear_screen();
    //let ms = time::Duration::from_millis(300); // 1/3 second
    let ms = time::Duration::from_millis(50);

    let temp_sprite_resource_id = make_fake_sprite_resource();
    //let temp_sprite_id = make_fake_sprite(temp_sprite_resource_id); // when we have a sprite as a resource, update this...

    let file_paths = "./test.save.bin".to_owned();
    let mut map_from_local_life: Map;
    let (map_resource, the_map) = match Resource::new(file_paths.clone(), false) {
        Ok(res_id) => {
            let res_result = Resource::try_get(res_id);
            let ret_tuple_result: Result<(Option<Resource>, &mut Map), String> = match res_result {
                Some(res) => match res.read_data(Map::deserialize_for_load) {
                    Ok(m) => {
                        if m.get_width() != SAMPLE_MAP_WIDTH {
                            panic!("Invalid data");
                        }
                        if m.get_height() != SAMPLE_MAP_HEIGHT {
                            panic!("Invalid data");
                        }
                        map_from_local_life = m;
                        Ok((Some(res), &mut map_from_local_life))
                    }
                    Err(e) => {
                        // if file exists, throw a panic!(), else assume bin data does not exist, and create a brand new map
                        println!("Error: {}", e);
                        Err(e)
                    }
                },
                None => {
                    let ret_err = format!(
                        "Was able to locate file '{}' (resource ID={})",
                        file_paths.clone(),
                        res_id
                    );
                    println!("{}", ret_err);
                    Err(ret_err)
                }
            };
            ret_tuple_result
        }
        Err(io_error) => {
            // if file exists, throw a panic!(), else assume bin data does not exist, and create a brand new map
            let str_io_error = io_error.to_string();
            let ret_err = Err::<(_, _), String>(str_io_error.to_owned());
            println!("Error: {}", str_io_error);

            let ret_error = match io_error.downcast_ref::<io::Error>() {
                Some(io_casted_error) => match io_casted_error.kind() {
                    io::ErrorKind::NotFound => {
                        // create a NEW map instead since we could not load it...
                        match Map::create(SAMPLE_MAP_WIDTH, SAMPLE_MAP_HEIGHT) {
                            Ok(new_map) => {
                                map_from_local_life = new_map;
                                Ok((None, &mut map_from_local_life))
                            }
                            Err(e_map) => Err(e_map),
                        }
                    }
                    _ => ret_err,
                },
                _ => ret_err,
            };
            ret_error
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

    //let cell = MapCell {
    //    layers: vec![CellLayer {
    //        id: 1,
    //        val: 'x' as lib_tower_defense::map::TCellValue,
    //    }],
    //};
    //for x in 0..32 {
    //    the_map.set(x, 0, cell.clone()).unwrap();
    //    the_map.set(x, 32, cell.clone()).unwrap();
    //}
    //for y in 1..31 {
    //    the_map.set(0, y, cell.clone()).unwrap();
    //    the_map.set(32, y, cell.clone()).unwrap();
    //}

    let mut break_loop = BreakLoopType::NoBreak;
    'main_game_outer_loop: loop {
        let view = the_map
            .build_view(0, 0, SAMPLE_VIEW_WIDTH, SAMPLE_VIEW_HEIGHT)
            .unwrap();
        if view.len() == 0 {
            break_loop = BreakLoopType::ApplicationError;
        }

        //'main_game_loop: loop {
        //for event in event_pump.poll_iter() {
        //    match event {
        //        Event::Quit { .. }
        //        | Event::KeyDown {
        //            keycode: Some(Keycode::Escape),
        //            ..
        //        } => break 'main_game_loop,
        //        _ => {}
        //    }
        //}

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
                        break 'main_game_outer_loop;
                    }
                    Keycode::Q => {
                        // Prompt to quit without save
                        println!("quit");
                        break_loop = BreakLoopType::QuitWithoutSave;
                        break 'main_game_outer_loop;
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
                                false => {
                                    let sprites_added = sprite_system::add(
                                        &temp_sprite_resource_id,
                                        deserialize_sprite,
                                    )
                                    .to_owned()
                                    .unwrap();

                                    MapCell {
                                        layers: sprites_added
                                            .iter()
                                            .map(|sid_added| {
                                                CellLayer::new(
                                                    0,
                                                    entity_system::add(sid_added, 0x80).unwrap(),
                                                )
                                            })
                                            .collect(),
                                    }
                                }
                            };

                        for layer in cursor_position_cell.layers.clone() {
                            let temp_cycle_the_value_on_space =
                                match (layer.entity + 1) as usize > LAYER_CHARS.len() {
                                    false => layer.entity + 1,
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
        let possibleLayerTopmost =
            match the_map.get_cell(view_x + cursor_x as u16, view_y + cursor_y as u16) {
                Ok(c) => {
                    let c_layers = c.layers.into_iter();
                    c_layers
                        // sort based on min layer_weight is lighter (bubbles towards top)
                        .min_by_key(|c| match entity_system::try_get(c.entity) {
                            Some(e) => e.layer_weight, // if tied, will only return the first encountered!
                            None => panic!(
                            "either entity_system deadlocked or entity_id={eid} no longer exists",
                            eid = c.entity
                        ), // should never happen (unless try_get was going to deadlocked and returned None), so will panic instead of returning u8::MAX
                        })
                }
                Err(e) => None,
            };

        let make_new_entity = || -> Vec<Entity> {
            // deal with it like an editor, add/create new element here
            let added_sprite_groups: HashSet<TSpriteSubGroupID> =
                make_fake_sprites(temp_sprite_resource_id);
            let entities = added_sprite_groups
                .into_iter() // convert
                .map(|sub_group_id| {
                    let added_entity_id = entity_system::add(&sub_group_id, 0x80).unwrap();
                    let r = entity_system::try_get(added_entity_id).unwrap();
                    return r.to_owned();
                });
            return entities.collect();
        };
        let entity_as_the_val = match possibleLayerTopmost
            .map(|cl| entity_system::try_get(cl.entity))
            .flatten()
        {
            Some(e) => Some(e.clone()),
            None =>
            // deal with it like an editor, add/create new element here
            {
                make_new_entity().first().map(|e| *e)
            }
        }
        .unwrap();

        let mut keys_input = "[".to_owned();
        for k in test_keys {
            keys_input.push_str(&k.to_string());
        }
        keys_input.push_str("]");
        let status = format!(
            "Map:{:?} - World:({}, {}) Cursor:({}, {}) Pos:({}, {}) Val:(EID:{}; SID:{})- Mouse:{:?} - Keys:{}\nCursor keys, PgUp, PgDn, '[', ']', 'space', 'Q', and Esc",
            the_map.get_upper_left(),
            view_x,
            view_y,
            cursor_x,
            cursor_y,
            view_x + cursor_x as u16,
            view_y + cursor_y as u16,
            entity_as_the_val.id,
            entity_as_the_val.sprites,
            mouse.coords,
            keys_input
        );
        // sleep mainly so that we can yield the app and let other processes run...
        thread::sleep(ms);
        match break_loop {
            BreakLoopType::QuitWithoutSave => break 'main_game_outer_loop,
            BreakLoopType::SaveAndExit => {
                // update data and quit
                let _bytes_written = match map_resource {
                    Some(mut m) => m.write_data(|| the_map.serialize_for_save()).unwrap(),
                    _ =>
                    // try to create new resource and attempt to save it?
                    {
                        Vec::new()
                    }
                };
                break 'main_game_outer_loop;
            }
            BreakLoopType::ApplicationError => break 'main_game_outer_loop,
            BreakLoopType::NoBreak => (),
        }
        render(view, cursor_x, cursor_y, status);
    }
    // need to flush, or else all the key input will queue up on exit of the app
    #[cfg(not(target_os = "linux"))]
    // the Windows/Mac way...
    //io::stdin().flush();
    std::io::stdin().read_line(&mut String::new()).unwrap();
    #[cfg(target_os = "linux")]
    std::io::stdin().read_line(&mut String::new()).unwrap();
    //if cfg!(target_os = "windows") {
    //    //#[cfg(not(target_os = "linux"))]
    //    #[cfg(target_os = "windows")]
    //    io::stdin().flush();
    //} else if cfg!(target_os = "macos") {
    //    //#[cfg(not(target_os = "linux"))]
    //    #[cfg(target_os = "macos")]
    //    io::stdin().flush();
    //} else if cfg!(target_os = "linux") {
    //    #[cfg(target_os = "linux")]
    //    unsafe {
    //        // the Linux way...
    //        libc::fflush(libc::stdin());
    //    }
    //}
}
