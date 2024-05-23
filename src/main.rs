//#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use crate::car::Car;
use crate::game::Game;

//mod script_based;
mod car;
mod game;
mod log;
mod runtime;

fn main() {
    let mut game = Game::new();

    let lua_script_1 = std::fs::read_to_string("src/base.lua").unwrap();
    game.register(Car::new(lua_script_1.clone()));
    game.register(Car::new(lua_script_1.clone()));
    game.register(Car::new(lua_script_1.clone()));

    game.race();
}
