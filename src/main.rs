//#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use crate::car::Car;
use crate::game::Game;

mod car;
mod game;
mod log;
mod runtime;

fn main() {
    let mut game = Game::new();

    let base_car = std::fs::read_to_string("scripts/base.lua").unwrap();
    game.register(Car::new(base_car.clone()));
    game.register(Car::new(base_car.clone()));
    game.register(Car::new(base_car.clone()));

    game.race();
}
