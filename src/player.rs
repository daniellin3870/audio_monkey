use std::io::BufReader;
use std::fs::File;
use std::path::PathBuf;

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
		let audio_file = File::open(audio.get_path()).map_err(|e| e.to_string())?;
		let audio = Decoder::try_from(audio_file).map_err(|e| e.to_string())?;
		self.player.append(audio);
		Ok(())
	}

	pub fn play_audio(&mut self, audio: Audio) -> Result<(), String> {
		let audio_file = File::open(audio.get_path())
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
				println!("unable to play {0}, skipping.", song.get_name());
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

#[derive(Serialize, Deserialize)]
pub struct Audio {
	name: String,
	duration: u64, // seconds
	path: PathBuf,
}

impl Audio {
	pub fn new(path: PathBuf) -> Result<Self, String> {
		if !path.exists() {
			return Err("No file found at path".to_string());
		}
		let name = String::from(path.file_stem().unwrap().to_str().unwrap());
		let duration = Decoder::try_from(
			File::open(&path).unwrap()).unwrap().total_duration().unwrap().as_secs();

		Ok(Audio {
			name,
			duration,
			path,
		})
	}
	pub fn get_name(&self) -> &String {
		&self.name
	}
	pub fn get_duration(&self) -> &u64 {
		&self.duration
	} 
	pub fn get_path(&self) -> &PathBuf {
		&self.path
	} 
}

impl From<Audio> for JsonValue {
	fn from(audio: Audio) -> Self {
		object!(
			name: audio.get_name().clone(),
			duration: *audio.get_duration(),
			path: audio.get_path()
				.clone()
				.to_str()
				.unwrap_or("invalid path")
				.to_string()
		)
	}
}

//impl Into<JsonValue> for Audio {
//	fn into(self) -> JsonValue {
//		let audio = self;
//		object!(
//			name: audio.get_name().clone(),
//			duration: *audio.get_duration(),
//			path: audio.get_path()
//				.clone()
//				.to_str()
//				.unwrap_or("invalid path")
//				.to_string()
//		)
//		
//	}
//}

impl Clone for Audio {
	fn clone(&self) -> Audio {
		Audio {
			name: self.name.clone(),
			duration: self.duration,
			path: self.path.clone(),
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct Playlist {
	name: String,
	count: u64,
	pub songs: Vec<Audio>,
}

impl Playlist {
	pub fn new(name: String, songs: Vec<Audio>) -> Self {
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
	pub fn get_name(&self) -> &String {
		&self.name
	}
	pub fn get_count(&self) -> &u64 {
		&self.count
	}
	pub fn get_songs(&self) -> &Vec<Audio> {
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

impl From<Playlist> for JsonValue {
	fn from(pl: Playlist) -> Self {
		object!(
			name:  pl.get_name().clone(),
			count: *pl.get_count(),
			songs: *pl.get_songs().clone()
		)
	}
}

