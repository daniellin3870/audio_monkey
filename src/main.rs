pub mod player;
pub mod cli;
pub mod cfg;
pub mod data;

use player::Player;


fn main() -> Result<(), String> {
		
	let config_path = std::env::home_dir()
		.ok_or("no home directory")
		.unwrap()
		.join(".config/audio_monkey/config.toml");

	let playlist_path = std::env::home_dir()
		.ok_or("no home directory")
		.unwrap()
		.join(".config/audio_monkey/config.toml");

	data::init()?;
	cfg::init()?;
	let mut config = cfg::load(config_path)?;
	
	let mut player: Player = Player::new();

	let mut all = data::load(playlist_path)?;

	let mut app = cli::AppState {
		player: &mut player,
		config: &mut config,
		all: &mut all
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
