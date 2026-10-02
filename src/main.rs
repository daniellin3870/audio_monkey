pub mod player;
pub mod cli;
pub mod cfg;
pub mod data;

use player::Player;

use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::Arc;
use std::{time::Duration, thread};
use std::io::{self, Write};

enum Event {
	Input(String),
	QueueEnd,
	AudioPos(Duration),
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
	player.looping = config.player.default_loop;

	let mut all = data::load(playlist_path)?;

	let mut app = cli::AppState {
		player: &mut player,
		config: &mut config,
		all: &mut all,
		current_playlist: None,
	};	

	let (tx, rx): (Sender<Event>, Receiver<Event>) = mpsc::channel();

	// input loop thread
	let input_tx = tx.clone();
	thread::spawn(move || {
		loop {
			let line = cli::readline();
			if let Ok(line) = line {
				let line = line.trim();
				if line.is_empty() { pp(); continue; }
				let _ = input_tx.send(Event::Input(line.to_string()));
			}

		}	
	});

	// song state thread
	let song_tx = tx.clone();
	let monitor_player = Arc::clone(&app.player.player);
	thread::spawn(move || {
		let mut playing = false;
		loop {
			thread::sleep(Duration::from_millis(100));
			if monitor_player.empty() { 
				if playing {
					playing = false;
					let _ = song_tx.send(Event::QueueEnd);
				}
			}
			else {
				playing = true;
				// TODO: replace this later with update screen or sm
				let pos = monitor_player.get_pos();
				let _ = song_tx.send(Event::AudioPos(pos));
			}	
		}
	});

	pp();

	while let Ok(event) = rx.recv() {
		type E = Event;
		match event {
			E::Input(line) => {
				let parsed = cli::parse(&line, &mut app);
				if let Ok(quit) = parsed {
					if quit { break; }
				}
				else if let Err(e) = parsed {
					println!("{e}");
				}
			},
			E::QueueEnd => {
				if let Some(list) = &app.current_playlist
				{
					if !app.player.looping { 
						app.current_playlist = None;
						continue; 
					}

					app.player.playlist(&list);
				} 
			}
			E::AudioPos(_pos) => {

			}
		}
		pp();
	}

	player.drop();
	Ok(())
}

fn pp() {
	print!("> ");
	let _ = io::stdout().flush();
}
