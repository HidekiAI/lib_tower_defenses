include!(concat!(env!("OUT_DIR"), "/hello.rs"));
use libc::*;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::{
    io::{self, prelude::*, stdin, Read, Write},
    thread, time
};

// see build.rs
use lib_tower_defense::map::Map;

const SAMPLE_VIEW_WIDTH: u8 = 80;
const SAMPLE_VIEW_HEIGHT: u8 = 40;
const SAMPLE_MAP_WIDTH: u16 = SAMPLE_VIEW_WIDTH as u16 * 5;
const SAMPLE_MAP_HEIGHT: u16 = SAMPLE_VIEW_HEIGHT as u16 * 5;

fn render(x: u16, y: u16, view: Vec<i64>) {
    let layer_chars = b"..,-~:;&=!*[<#$@";
    println!("\x1b[H"); // ESC[H = move cursor back to HOME position
    for h in 0..SAMPLE_VIEW_HEIGHT {
        for w in 0..SAMPLE_VIEW_WIDTH {
            let i = (h as usize * SAMPLE_VIEW_WIDTH as usize) + w as usize; // NOTE: will get overflow if you do not explicitly cast here
            let v = view[i] as usize;
            //print!("{}", layer_chars[v]);
            unsafe {
                libc::putchar(layer_chars[v].into());
            }
        }
        println!("");
    }
}

fn main() {
    println!("\x1b[2J"); // ESC[2J = clear entire screen (CLS)
    println!("main (text view): This is a shell U.I. via TEXT based");
    let device_state = DeviceState::new();
    //let ms = time::Duration::from_millis(300); // 1/3 second
    let ms = time::Duration::from_millis(3); // 1/3 second
    let mut theMap = Map::create(SAMPLE_MAP_WIDTH, SAMPLE_MAP_HEIGHT).unwrap();
    let mut view_x: u16 = 0;
    let mut view_y: u16 = 0;
    let mut cursor_x: u8 = 0;
    let mut cursor_y: u8 = 0;
    let move_step_x: u8 = 5;
    let move_step_y: u8 = 5;
    loop {
        let view = theMap
            .build_view(0, 0, SAMPLE_VIEW_WIDTH, SAMPLE_VIEW_HEIGHT)
            .unwrap();
        render(view_x, view_y, view);
        let keys: Vec<Keycode> = device_state.get_keys();
        if !keys.is_empty() {
            match keys.first() {
                Some(Keycode::Escape) => {
                    println!("escape");
                    break;
                }
                Some(Keycode::Q) => {
                    println!("quit");
                    break;
                }
                Some(Keycode::PageUp) => {
                    if view_y > move_step_y as u16 {
                        view_y = view_y - move_step_y as u16;
                    }
                }
                Some(Keycode::Up) => {
                    if cursor_y > 0 {
                        cursor_y = cursor_y - 1;
                    } else if view_y > move_step_y as u16 {
                        // rather than moving the cursor up, we'd scroll UP because we're at the edge
                        view_y = view_y - move_step_y as u16;
                    }
                }
                Some(Keycode::PageDown) => {
                    if view_y < theMap.get_height() + SAMPLE_MAP_HEIGHT + move_step_y as u16 {
                        view_y = view_y + move_step_y as u16;
                    }
                }
                Some(Keycode::Down) => {
                    if cursor_y < SAMPLE_VIEW_HEIGHT {
                        cursor_y = cursor_y + 1;
                    } else if view_y < theMap.get_height() + SAMPLE_MAP_HEIGHT {
                        view_y = view_y + move_step_y as u16;
                    }
                }
                Some(Keycode::LeftBracket) => {
                    if view_x > move_step_x as u16 {
                        view_x = view_x - move_step_x as u16;
                    }
                }
                Some(Keycode::Left) => {
                    if cursor_x > 0 {
                        cursor_x = cursor_x - 1;
                    } else if view_x > move_step_x as u16 {
                        // rather than moving the cursor left, we'd scroll view left
                        view_x = view_x - move_step_x as u16;
                    }
                }
                Some(Keycode::RightBracket) => {
                    if view_x < theMap.get_width() + SAMPLE_MAP_WIDTH + move_step_x as u16 {
                        view_x = view_x + move_step_x as u16;
                    }
                }
                Some(Keycode::Right) => {
                    if cursor_x < SAMPLE_VIEW_WIDTH {
                        cursor_x = cursor_x + 1;
                    } else if view_x < theMap.get_width() + SAMPLE_MAP_WIDTH {
                        view_x = view_x + move_step_x as u16;
                    }
                }
                Some(Keycode::Space) => {
                    let pos_x = view_x + cursor_x as u16;
                    let pos_y = view_y + cursor_y as u16;
                    let mut currentCell = theMap.get_cell(pos_x, pos_y).unwrap();
                    currentCell.set(1, 1);
                    theMap.set(pos_x, pos_y, currentCell);
                }
                _ => (),
            }
        }
        thread::sleep(ms);
    }
}
