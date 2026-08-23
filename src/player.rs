use std::io::BufReader;
use std::fs::File;
use std::path::{Path, PathBuf};

use rodio::{Decoder, MixerDeviceSink, source::Source};
use serde::{Serialize, Deserialize};

use json::{object, JsonValue};

pub struct Player {
	stream_handle: MixerDeviceSink,
	player: rodio::Player,
}

impl Player {
	pub fn new() -> Self {
		
		let stream_handle = rodio::DeviceSinkBuilder::open_default_sink()
			.expect("open default audio stream");
		
		let player = rodio::Player::connect_new(stream_handle.mixer());
		Player {
			stream_handle,
			player,
		}
	}

	pub fn queue_audio(&self, audio: &Audio) -> Result<(), String> {
		let audio_file = File::open(audio.path()).map_err(|e| e.to_string())?;
		let audio = Decoder::try_from(audio_file).map_err(|e| e.to_string())?;
		self.player.append(audio);
		Ok(())
	}

	pub fn play_audio(&mut self, audio: Audio) -> Result<(), String> {
		let audio_file = File::open(audio.path())
			.map_err(|e| e.to_string())?;
		let player = rodio::play(
			self.stream_handle.mixer(), 
			BufReader::new(audio_file))
			.map_err(|e| e.to_string())?; 
		self.player = player;
		Ok(())
	}

	pub fn set_volume(&self, volume: f32) {
		self.player.set_volume(volume);
	}
	
	pub fn set_speed(&self, speed: f32) {
		self.player.set_speed(speed);
	}
	
	pub fn playpause(&self) {
		if self.player.is_paused() { self.player.play(); }
		else { self.player.pause(); }
	}
	
	pub fn play(&self) {
		if self.player.empty() { 
			println!("Nothing in queue"); 
			return;	
		}
		self.player.play();
	}

	pub fn pause(&self) {
		if self.player.empty() { 
			println!("Nothing in queue"); 
			return;	
		}
		self.player.pause();
	}
	
	pub fn drop(self) {
		drop(self.player);
	}
	
	pub fn playlist(&mut self, playlist: Playlist) -> Result<(), String> {
		for song in playlist.songs {
			if let Err(e) = self.queue_audio(&song) {
				println!("unable to play {0}, skipping.", song.name());
				print!("{:#?}", e);
				continue;
			}
		}	
		Ok(())
	}

}

impl Default for Player {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Audio {
	name: String,
	duration: u64, // seconds
	path: PathBuf,
}

impl Audio {
	pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
		let path = path.as_ref();
		if !path.exists() {
			return Err("No file found at path".to_string());
		}
		let name = String::from(path.file_stem().unwrap().to_str().unwrap());
		let duration = Decoder::try_from(
			File::open(&path).unwrap()).unwrap().total_duration().unwrap().as_secs();

		Ok(Audio {
			name,
			duration,
			path: path.to_owned(),
		})
	}
	pub fn name(&self) -> &str {
		&self.name
	}
	pub fn duration(&self) -> u64 {
		self.duration
	} 
	pub fn path(&self) -> &Path {
		&self.path
	} 
}

impl From<&Audio> for JsonValue {
	fn from(audio: &Audio) -> Self {
		object!(
			name: audio.name(),
			duration: audio.duration(),
			path: audio.path()
				.to_str()
				.unwrap_or_else(|| "invalid path")
				.to_string()
		)
	}
}

impl From<Audio> for JsonValue {
	fn from(audio: Audio) -> Self {
		JsonValue::from(&audio)
	}
}

impl From<JsonValue> for Audio {
	fn from(v: JsonValue) -> Self {
		Audio::from(&v)
	}
}

impl From<&JsonValue> for Audio {
	fn from(v: &JsonValue) -> Self {
		Audio::new(v["path"].as_str().unwrap())
			.expect("bad JsonValue to Audio conversion")
	}
}
impl Clone for Audio {
	fn clone(&self) -> Audio {
		Audio {
			name: self.name.clone(),
			duration: self.duration,
			path: self.path.clone(),
		}
	}
}

#[derive(Clone, Debug,Serialize, Deserialize)]
pub struct Playlist {
	name: String,
	count: u64,
	pub songs: Vec<Audio>,
}

impl Playlist {
	pub fn new<S: AsRef<str>>(name: S, songs: Vec<Audio>) -> Self {
		let name = name.as_ref().to_string();

		Playlist {
			name,
			count: songs.len() as u64,
			songs 
		}
	}
	pub fn set_name(&mut self, name: String) {
		self.name = name;
	}
	pub fn add_songs(&mut self, songs: Vec<Audio>) {
		for audio in songs {
			self.songs.push(audio);
		}
		self.count = self.songs.len() as u64;
	}
	pub fn sub_songs(&mut self, songs: Vec<String>) {
		use std::collections::HashMap;
		let mut map: HashMap<String, Audio> = HashMap::new();

		for song in &self.songs {
			map.insert(song.name().to_owned(), song.clone());
		}	

		for song in &songs {
			map.remove(song);
		}

		self.songs = map.into_values().collect();

	}
	pub fn name(&self) -> &str {
		&self.name
	}
	pub fn count(&self) -> u64 {
		self.count
	}
	pub fn songs(&self) -> &Vec<Audio> {
		&self.songs
	}
}

impl Default for Playlist {
	fn default() -> Playlist {
		Playlist {
			name: String::new(),
			count: 0,
			songs: Vec::new()
		}
	}
}

impl From<&JsonValue> for Playlist {
	fn from(v: &JsonValue) -> Self {
		// iterator over a list of Audio
		let members = v["songs"].members();
		let mut songs = Vec::<Audio>::new();
		for song in members {
			songs.push(song.into());
		}
		
		let name = v["name"].as_str().unwrap_or("");

		Playlist::new(name, songs)
	}
}

impl From<JsonValue> for Playlist {
	fn from(v: JsonValue) -> Self {
		Playlist::from(&v)
	}
}


impl From<&Playlist> for JsonValue {
	fn from(pl: &Playlist) -> Self {
		object!(
			name:  &pl.name()[..],
			count: pl.count(),
			songs: *pl.songs().clone()
		)
	}
}

impl From<Playlist> for JsonValue {
	fn from(pl: Playlist) -> Self {
		JsonValue::from(&pl)
	}
}

