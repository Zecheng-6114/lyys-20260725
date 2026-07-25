mod command;
mod container;
mod data;
mod error;
mod game;
mod item;
mod player;
mod shop;

use data::GameData;
use game::Game;

fn main() {
    let data = match GameData::load("data.toml") {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let mut game = Game::new(data);
    game.run();
}
