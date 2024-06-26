//#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use crate::car::Car;
use crate::game::Game;

mod action;
mod car;
mod game;
mod log;
mod runtime;

fn main() {
    let mut game = Game::new();

    let base_car = std::fs::read_to_string("scripts/base.lua").unwrap();

    game.register(Car::new(base_car.clone(), "Alice".to_string()));
    game.register(Car::new(base_car.clone(), "Bob".to_string()));
    game.register(Car::new(base_car.clone(), "Charlie".to_string()));

    game.race();
}
