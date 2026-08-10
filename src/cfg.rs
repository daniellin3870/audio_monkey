use std::path::{PathBuf, Path};
use std::fs::{self, OpenOptions};
use std::str::FromStr;
use std::fmt::Display;
use serde::{Deserialize, Deserializer, Serialize, Serializer};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub u8, pub u8, pub u8); // {r, g, b}

impl FromStr for Color {
	
	type Err = String;
	
	fn from_str(hex: &str) -> Result<Self, Self::Err> {
		let hex = hex.trim_start_matches('#');
		if hex.len() != 6 { return Err(format!("invalid hex: {hex}")); }
		
		let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
		let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
		let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;

		Ok(Color(r, g, b))
	}
}

impl Display for Color {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
	}
}

impl Serialize for Color {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> Deserialize<'de> for Color {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		s.parse().map_err(serde::de::Error::custom)
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
	pub player: PlayerConfig,
	pub downloader: DownloaderConfig,
	pub color: ColorConfig
}

//impl std::fmt::Display for Config {
//
//	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//		let player = self.player;
//		let downloader = self.downloader;
//		let color = self.color;
//
//		let buffer = format!()
//	}
//}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlayerConfig {
	pub music_directory: String,
	pub volume: f64,
	pub playback_speed: f64
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloaderConfig {
	pub download_path: String,
	pub options: String,
	pub format: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ColorConfig {
	pub background: Color 
}

pub fn init() -> Result<(), String> {
	let mut config_dir: PathBuf = std::env::home_dir()
		.ok_or("no home directory")
		.unwrap()
		.join(".config/audio_monkey");

	let _ = std::fs::create_dir_all(config_dir.clone());

	config_dir.push("config.toml");	

	if let Err(e) = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(config_dir) 
	{
		if e.kind() == std::io::ErrorKind::AlreadyExists { return Ok(()); }
		else { return Err(format!("Error: {e}").to_string()); }
	}	
	
	Ok(())
}

pub fn save(config_dir: PathBuf, config: &Config) -> Result<(), String> {

	let config = toml::to_string(config).map_err(|e| e.to_string())?;

	std::fs::write(config_dir.as_path(), &config).map_err(|e| e.to_string())

}

pub fn load(config_dir: PathBuf) -> Result<Config, String> {

	let config_contents = fs::read_to_string(config_dir).map_err(|e| e.to_string())?; 
	toml::from_str::<Config>(&config_contents).map_err(|e| e.to_string())
}
#[cfg(test)]
mod tests {
	use super::*;
	

	// tests functionality of Display on Color struct
	#[test]
	fn color_display() {
		let a = Color(0, 0, 0);
		let b = Color(8, 35, 126);
		let c = Color(255, 255, 255);
		
		assert_eq!(a.to_string(), "#000000");
		assert_eq!(b.to_string(), "#08237E");
		assert_eq!(c.to_string(), "#FFFFFF");
		
	}	
	#[test]
	fn color_from_str() {
		assert_eq!(Color::from_str("#000000"), 		 Ok(Color(0,0,0)));
		assert_eq!(Color::from_str("#0F0F0F"), 		 Ok(Color(15,15,15)));
		assert_eq!(Color::from_str("#FFFFFF"), 		 Ok(Color(255,255,255)));
		assert_eq!(Color::from_str("#"), 			 Err("invalid hex: ".to_string()));
		assert_eq!(Color::from_str("#FEFEFEFEFEFE"), Err("invalid hex: FEFEFEFEFEFE".to_string()));
	}
}


