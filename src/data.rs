use crate::player::Playlist;

use std::path::{Path, PathBuf};
use std::fs::{self, OpenOptions};
use std::collections::HashMap;

use json::{array, JsonValue};

pub fn init() -> Result<(), String> {
	let playlist_path: PathBuf = std::env::home_dir()
		.ok_or("no home directory")
		.unwrap()
		.join(".local/share/audio_monkey/playlist.json");

	let _ = std::fs::create_dir_all(playlist_path.clone());

	if let Err(e) = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(&playlist_path) 
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

	let playlist_path = playlist_path.as_ref();

	let mut data: Vec<JsonValue> = Vec::new();

	for list in all {
		let entry: JsonValue = list.into();
		data.push(entry);
	}

	let data = JsonValue::Array(data);

	std::fs::write(playlist_path, data.dump()).map_err(|e| e.to_string())?;

	Ok(())
	
}

pub fn load<P: AsRef<Path>>(playlist_path: P) -> Result<Vec<Playlist>, String> {
	let playlist_path = playlist_path.as_ref();

	let buffer = fs::read_to_string(playlist_path)
		.map_err(|e| e.to_string())?;

	dbg!(&buffer);

	let buffer = json::parse(&buffer)
		.map_err(|e| e.to_string())?;

	dbg!(&buffer);

	let iterator = buffer.members();
	
	dbg!(&iterator);

	let mut playlists: Vec<Playlist> = Vec::new();

	for value in iterator {
		playlists.push(value.into());
	}

	dbg!(&playlists);

	Ok(playlists)
}
