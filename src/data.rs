use crate::player::Playlist;

use std::path::PathBuf;
use std::fs::{self, OpenOptions};
use std::collections::HashMap;

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

//pub fn search_playlist(name: String) -> Result<Playlist, String> {
//	Ok(())
//}

pub fn save(dir: PathBuf, all: Vec<Playlist>) -> Result<(), String> {
	use json::JsonValue;

	let mut buffer: String = String::new();

	for list in all {
		let entry: JsonValue = list.into();
		buffer.push_str(&entry.dump());
	}

	std::fs::write(dir, buffer).map_err(|e| e.to_string())?;

	Ok(())
	
}

pub fn load(dir: PathBuf) -> Result<Vec<Playlist>, String> {
	todo!();
	//TODO: make this work, experiment with the parsing and ish

	let mut playlists: Vec<Playlist> = Vec::new();
	
	let buffer = fs::read_to_string(dir)
		.map_err(|e| e.to_string())?;

	let buffer = json::parse(&buffer)
		.map_err(|e| e.to_string())?;

	dbg!(buffer.dump());

	Ok(playlists)
}
