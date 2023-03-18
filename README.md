# lib_tower_defenses

RUST based tower-defense lib, main goal design is to be like combination of Starcraft and Mindustry

## To Verify

``` bash
$ cargo test   # build unit-test
$ cargo run --bin lib_tower_defense  # test-run TUI version (use '--bin sdl2_view' for GUI)
```

## Notes

* Only tested on Debian, need to be verified if it works on Windows, main concern is that it relies on `libc` (see Cargo.toml)
* The main.rs on the root folder is for TUI (Text-based), and intend to make one test-shell for SDL2 if time avail
