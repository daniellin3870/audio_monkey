use crate::player::Playlist;

use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions};
use std::collections::HashMap;

pub fn init() -> Result<(), String> {
	let mut playlist_path: PathBuf = std::env::home_dir()
		.ok_or("no home directory")
		.unwrap()
		.join(".local/share/audio_monkey");

	let _ = std::fs::create_dir_all(playlist_path.clone());

	playlist_path.push("playlist.json");	

	if let Err(e) = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(playlist_path) 
	{
		if e.kind() == std::io::ErrorKind::AlreadyExists { return Ok(()); }
		else { return Err("Error: {e}".to_string()); }
	}	
	
	Ok(())
}

//pub fn search_playlist(name: String) -> Result<Playlist, String> {
//	Ok(())
//}

pub fn save<P: AsRef<Path>>(playlist_path: P, all: &Vec<Playlist>) -> Result<(), String> {
	use json::JsonValue;

	let playlist_path = playlist_path.as_ref();
	let mut buffer: String = String::new();

	for list in all {
		let entry: JsonValue = list.into();
		buffer.push_str(&entry.dump());
	}

	std::fs::write(playlist_path, buffer).map_err(|e| e.to_string())?;

	Ok(())
	
}

pub fn load<P: AsRef<Path>>(playlist_path: P) -> Result<Vec<Playlist>, String> {
	//TODO: make this work, experiment with the parsing and ish
	let playlist_path = playlist_path.as_ref();

	let mut playlists: Vec<Playlist> = Vec::new();
	
	let buffer = fs::read_to_string(playlist_path)
		.map_err(|e| e.to_string())?;

	let buffer = json::parse(&buffer)
		.map_err(|e| e.to_string())?;

	dbg!(buffer.dump());

	Ok(playlists)
}
