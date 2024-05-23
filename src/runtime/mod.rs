use crate::game::Game;

mod lua;
mod process_based;

pub trait Runner {
    fn run(&mut self, game: &mut Game) -> Result<(), String>;
}