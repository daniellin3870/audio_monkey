pub mod player;
pub mod cli;
pub mod cfg;
pub mod data;

use player::Player;

use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;
use std::io::{self, Write};

enum Event {
	Input(String),
	SongEnd,
}

fn main() -> Result<(), String> {
		
	let config_path = std::env::home_dir()
		.ok_or("no home directory")
		.unwrap()
		.join(".config/audio_monkey/config.toml");

	let playlist_path = std::env::home_dir()
		.ok_or("no home directory")
		.unwrap()
		.join(".local/share/audio_monkey/playlist.json");

	data::init()?;
	cfg::init()?;

	let mut config = cfg::load(config_path)?;
	
	let mut player: Player = Player::new();

	player.set_volume(config.player.default_volume);
	player.set_speed(config.player.default_playback);

	let mut all = data::load(playlist_path)?;

	let mut app = cli::AppState {
		player: &mut player,
		config: &mut config,
		all: &mut all
	};	

	//TODO: rewrite to event-driven REPL
	// needs a worker thread and background thread
	// needs central channel which collects events
	// from the two threads

	let (tx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();

	let input_tx = tx.clone();
	// input loop thread
	thread::spawn(move || {
		loop {
			let line = cli::readline();
			if let Ok(line) = line {
				let line = line.trim();
				if line.is_empty() { continue; }

				let _ = input_tx.send(Event::Input(line.to_string()));
			}

		}	
	});

	pp();
	while let Ok(event) = rx.recv() {
		match event {
			Event::Input(line) => {
				let parsed = cli::parse(&line, &mut app);
				if let Ok(quit) = parsed {
					if quit { break; }
				}
				else if let Err(e) = parsed {
					println!("{e}");
				}
			}
			Event::SongEnd => todo!() // app.player.playlist() ?
		}
		pp();
	}

	//match cli::parse(line, &mut app) {
	//	Ok(quit) => {
	//		if quit { break; }
	//		
	//	}
	//	Err(e) => {
	//		println!("{e}")
	//	}
	//}


	player.drop();
	Ok(())
}

fn pp() {
	print!("> ");
	io::stdout().flush();
}
