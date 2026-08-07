use crate::player::Playlist;

use std::path::PathBuf;
use std::fs::OpenOptions;

use json::object;



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

pub fn save(dir: String, all: Vec<Playlist>) -> Result<(), String> {
	let mut buffer: String = String::new();

	for list in all {
		let entry = object!(
			name:  list.get_name().clone(),
			count: *list.get_count(),
			songs: *list.get_songs().clone()
		);

		buffer.push_str(&entry.dump());
	}

	std::fs::write(dir, buffer).map_err(|e| e.to_string())?;

	Ok(())
	
}
