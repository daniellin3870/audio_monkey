use crate::player::Playlist;
use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use serde::{Deserialize, Serialize};

pub fn init() -> Result<(), String> {
	let mut playlist_dir: PathBuf = std::env::home_dir()
		.ok_or("no home directory")
		.unwrap()
		.join(".local/share/audio_monkey");

	let _ = std::fs::create_dir_all(playlist_dir.clone());

	playlist_dir.push("playlist.json");	

	if let Err(e) = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(playlist_dir) 
	{
		if e.kind() == std::io::ErrorKind::AlreadyExists { return Ok(()); }
		else { return Err("Error: {e}".to_string()); }
	}	
	
	Ok(())
}

