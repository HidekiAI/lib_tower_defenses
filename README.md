# lib_tower_defenses

WIP (WORK IN PROGRESS) - Currently residing here as public so that I can showcase myself during this period of which I am now looking for new employment opportunities.  As soon as I am employed, I MAY possibly turn this project back to private; once I get it all starting to become more finctional, I may then again return this crate back to public.

RUST based tower-defense lib, main goal design is to be like combination of Starcraft and Mindustry

## To Verify

``` bash
$ cargo test   # build unit-test
<some test results, hopefully, all results to OK>
$ cargo run --bin lib_tower_defense  # test-run TUI version (use '--bin sdl2_view' for GUI)
<milage may differ between O/S, ncurses is only for Linux, thus I'm using pure ANSI terminal (i.e. VT100) commands>
```

## Notes

* Only tested on Debian, need to be verified if it works on Windows, main concern is that it relies on `libc` (see Cargo.toml)
* The main.rs on the root folder is for TUI (Text-based), and intend to make one test-shell for SDL2 if time avail

## Dependencies

See Cargo.toml for details (i.e. `cargo tree` is a nice way to view it)

* SDL2

## Art/Assets

All assets are from <https://opengameart.org/>
Note: Make sure to credit as per described from <https://opengameart.org/content/faq#q-how-to-credit>

* "fantasy_tileset0finished.png" by Hollyhart1 licensed CC-BY 1.0: <https://opengameart.org/content/8-bit-jrpg-tilesets>
* "sara-cal.png" by Mandi Paugh licensed CC-BY 3.0, CC-BY-SA 3.0, GPL 2.0, or GPL 3.0 <https://opengameart.org/content/sara-wizard>
