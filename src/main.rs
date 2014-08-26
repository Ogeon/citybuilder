#![feature(struct_variant)]

extern crate rsfml;

mod game;
mod start_state;
mod edit_state;

//For SFML on OS X
#[cfg(target_os="macos")]
#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    native::start(argc, argv, main)
}

fn main() {
    let mut game = game::Game::new().expect("unable to create game window");
    let state = start_state::StartState::new(&game).expect("unable to create start view");
    game.push_state(box state as Box<game::GameState>);
    game.game_loop();
}