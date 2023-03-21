include!(concat!(env!("OUT_DIR"), "/hello.rs"));
use device_query::{DeviceEvents, DeviceState};
use device_query::{DeviceQuery, Keycode};
use libc::*;
use std::{
    io::{self, prelude::*, stdin, Read, Write},
    thread, time,
};

// see build.rs
use lib_tower_defense::map::{CellLayer, Map, MapCell};

const SAMPLE_VIEW_WIDTH: u8 = 80;
const SAMPLE_VIEW_HEIGHT: u8 = 40;
const SAMPLE_MAP_WIDTH: u16 = SAMPLE_VIEW_WIDTH as u16 * 5;
const SAMPLE_MAP_HEIGHT: u16 = SAMPLE_VIEW_HEIGHT as u16 * 5;

fn render(x: u16, y: u16, view: Vec<i64>, cursor_x: u8, cursor_y: u8) {
    let layer_chars = b" .,-~:;&=!*[<#$@"; // for debuggin, replace ' ' (space) with '.' if needed
    let cursor_char = 'o';
    let escape = "\x1b";
    println!("{}[2J", escape); // ESC[2J = clear entire screen (CLS) because we need to clear last scroll image (also removes last cursor position)
    println!("{}[H\n", escape); // ESC[H = move cursor back to HOME position (done per each gameloop) and add 1 line down for status

    for h in 0..SAMPLE_VIEW_HEIGHT {
        for w in 0..SAMPLE_VIEW_WIDTH {
            let i = (h as usize * SAMPLE_VIEW_WIDTH as usize) + w as usize; // NOTE: will get overflow if you do not explicitly cast here

            // for now, just assume the val data is the index to the array...
            let v = view[i] as usize;
            //unsafe {
            //    let ch: c_int = layer_chars[v].into();
            //    // using C lib putchar() for now (performance and conviniences)
            //    libc::putchar(ch);
            //}
            let ch: char = layer_chars[v] as char;
            if h == cursor_y && w == cursor_x {
                if ch == ' ' {
                    print!("{}", cursor_char);
                } else {
                    // bold
                    print!("{}[1m{}{}[0m", escape, ch, escape);
                }
            } else {
                print!("{}", ch);
            }
        }
        println!("");
    }
}

enum break_loop_type {
    no_break = 0,
    quit_without_save = 1,
    save_and_exit = 2,
    application_error,
}
fn main() {
    println!("\x1b[2J"); // ESC[2J = clear entire screen (CLS)
    println!("main (text view): Cursor keys, PgUp, PgDn, '[', ']', 'space', 'Q', and Esc");
    //let ms = time::Duration::from_millis(300); // 1/3 second
    let ms = time::Duration::from_millis(3); // 1/3 second
    let mut theMap = Map::create(SAMPLE_MAP_WIDTH, SAMPLE_MAP_HEIGHT).unwrap();
    let mut view_x: u16 = 0;
    let mut view_y: u16 = 0;
    let mut cursor_x: u8 = 0;
    let mut cursor_y: u8 = 0;
    let move_step_x: u8 = 5;
    let move_step_y: u8 = 5;
    let device_state = DeviceState::new();

    // maybe in future, will use this callback method, but for now, want
    // to keep cursor movement localized only within gameloop...
    //let _guard = device_state.on_mouse_move(|position| {
    //    print!(" Mouse position: {:#?} ", position);
    //});
    //let _guard = device_state.on_mouse_down(|button| {
    //    print!(" Mouse button down: {:#?} ", button);
    //});
    //let _guard = device_state.on_mouse_up(|button| {
    //    print!(" Mouse button up: {:#?} ", button);
    //});
    //let _guard = device_state.on_key_down(|key| {
    //    print!(" Keyboard key down: {:#?} ", key);
    //});
    //let _guard = device_state.on_key_up(|key| {
    //    print!(" Keyboard key up: {:#?} ", key);
    //});

    let mut break_loop = break_loop_type::no_break;
    loop {
        let view = theMap
            .build_view(0, 0, SAMPLE_VIEW_WIDTH, SAMPLE_VIEW_HEIGHT)
            .unwrap();
        if view.len() == 0 {
            break_loop = break_loop_type::application_error;
            break;
        }
        render(view_x, view_y, view, cursor_x, cursor_y);

        let mouse = device_state.get_mouse();
        let keys: Vec<Keycode> = device_state.get_keys();
        let test_keys = keys.clone();
        if !keys.is_empty() {
            for k in keys {
                match k {
                    Keycode::Escape => {
                        // Prompt to save data
                        println!("escape");
                        break_loop = break_loop_type::save_and_exit;
                        break;
                    }
                    Keycode::Q => {
                        // Prompt to quit without save
                        println!("quit");
                        break_loop = break_loop_type::quit_without_save;
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

                        if bot_y < theMap.get_height() {
                            view_y = top_y;
                        }
                    }
                    Keycode::Down => {
                        let top_y = view_y + move_step_y as u16;
                        let bot_y = top_y + SAMPLE_VIEW_HEIGHT as u16;

                        if cursor_y < SAMPLE_VIEW_HEIGHT {
                            cursor_y = cursor_y + 1;
                        } else if bot_y < theMap.get_height() {
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

                        if right_x < theMap.get_width() {
                            view_x = left_x;
                        }
                    }
                    Keycode::Right => {
                        let left_x = view_x + move_step_x as u16;
                        let right_x = left_x + SAMPLE_VIEW_WIDTH as u16;

                        if cursor_x < SAMPLE_VIEW_WIDTH {
                            cursor_x = cursor_x + 1;
                        } else if right_x < theMap.get_width() {
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
                            match theMap.get_cell(pos_x, pos_y).unwrap().first().is_some() {
                                true => theMap.get_cell(pos_x, pos_y).unwrap(),
                                false => MapCell {
                                    layers: vec![CellLayer::new(0, 0); 1],
                                },
                            };

                        for layer in cursor_position_cell.layers.clone() {
                            let temp_cycle_the_value_on_space = match layer.val + 1 > 8 {
                                false => layer.val + 1,
                                true => 0,
                            };
                            // Note: set() should add if missing, but in this case, we're iterating through existing Layers, so it assumes it's always an update/repleace
                            cursor_position_cell.set(layer.id, temp_cycle_the_value_on_space);
                        }

                        theMap.set(pos_x, pos_y, cursor_position_cell);
                    }
                    _ => (),
                }
            }
            // update map position based on key pressed
            theMap.set_upper_left(view_x, view_y);
        }

        // for now, only update text if key is pressed
        let the_val = match theMap
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
        println!(
            "\x1b[HMap:{:?} - World:({}, {}) Cursor:({}, {}) Pos:({}, {}) Val:{} - Mouse:{:?} - Keys:{}                    ",
            theMap.get_upper_left(),
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
            break_loop_type::quit_without_save => break,
            break_loop_type::save_and_exit => break,
            _ => (),
        }
    }
}
