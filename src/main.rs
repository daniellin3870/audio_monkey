use std::{path::Path, io::{self, Write}};

pub mod player;
pub mod cli;
pub mod cfg;
pub mod data;

use player::{Player, Audio};
use cli::Cli;

fn main() -> Result<(), String> {
	
		
	data::init()?;
	cfg::init()?;
	let mut config = cfg::load()?;

	
	let mut player: Player = Player::new();
	
	let mut app = cli::AppState {
		player: &mut player,
		config: &mut config
	};

	loop {
		let line = cli::readline()?;
		let line = line.trim();
		if line.is_empty() { continue; }

		match cli::parse(line, &mut app) {
			Ok(quit) => {
				if quit { break; }
				
			}
			Err(e) => {
				println!("{e}")
			}
		}
	}	


	player.drop();
	Ok(())
}
