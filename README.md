# lib_tower_defenses
RUST based tower-defense lib, main goal design is to be like combination of Starcraft and Mindustry

## To Verify:
<code>$ cargo test</code>
<code>$ cargo run --bin lib_tower_defense</code>

## Notes:
* Only tested on Debian, need to be verified if it works on Windows, main concern is that it relies on `libc` (see Cargo.toml)
* The main.rs on the root folder is for TUI (Text-based), and intend to make one test-shell for SDL2 if time avail
