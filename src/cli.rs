use clap::{ValueEnum, Parser, Subcommand}; 
use rand::seq::SliceRandom;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use crate::player::{Audio, Player, Playlist};
use crate::cfg::Config;

type Result<T> = std::result::Result<T, String>;

#[derive(Debug, Parser)]
#[command(multicall = true)]
pub struct Cli {
	#[command(subcommand)]
	commands: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
	Play { 
		#[arg(
			short = 'p',
			default_value_t = false
		)]
		playlist: bool,
		#[arg(
			short = 's',
			requires("playlist")
		)]
		shuffle: bool,
		value: Option<String>
	},
	Pause,
	#[command(name = "playpause", alias = "pp")]
	PlayPause,
	Download {
		#[arg(
			short = 'p',
			default_value_t = false
		)]
		playlist: bool,
		#[arg(short = 'f')]
		format: Option<String>,
		#[arg(short = 'n')]
		name: Option<String>,
		url: String,
	},
	Config {
		#[command(subcommand)]
		option: ConfigOptions,
	},
	Playlist {
		#[command(subcommand)]
		option: PlaylistOptions,
	},
	Duration {
		path: String,
	},
	Skip,
	Volume {
		value: f32,
	},
	Loop {
		#[arg(
			action = clap::ArgAction::Set,
			value_parser = clap::builder::BoolishValueParser::new()
		)]
		enabled: bool,
	},
	Exit
}


#[derive(Subcommand, Clone, Debug)]
enum ConfigOptions {
	Get,
	Save,
	Set {
		#[arg(value_enum)]
		key: ConfigKey,
		value: String
	}
}

#[derive(ValueEnum, Clone, Debug)]
enum ConfigKey {
	MusicDirectory,
	//TODO: make loop, playbackspeed do something
	DefaultVolume,
	DefaultPlayback,
	DefaultLoop,
	DefaultShuffle,
	DownloadPath,
	Options,
	Format,
	Background
}


#[derive(Subcommand, Clone, Debug)]
enum PlaylistOptions {
	Add {
		playlist: String,
		songs: Vec<String>
	},
	Sub {
		playlist: String,
		songs: Vec<String>
	},
	Rename {
		playlist: String,
		new_name: String
	},
	Create {
		name: String
	},
	Save,
	List { 
		#[arg(
			short = 'v',
			default_value_t = false
		)]
		verbose: bool,
		playlist: String 
	},
}


pub struct AppState<'a> {
	pub player: &'a mut Player,
	pub config: &'a mut Config,
	pub all:    &'a mut Vec<Playlist>,
	pub current_playlist: Option<Playlist>,
}

impl<'a> AppState<'a> {
	pub fn add_audio<P: AsRef<Path>>(&mut self, playlist: String, song_paths: Vec<P>) -> Result<()> {
		let mut results: Vec<Audio> = Vec::new();
		for path in song_paths {
			let path = path.as_ref();
			let new_path: PathBuf;

			if path.is_relative() {
				new_path = Path::new(&self.config.player.music_directory)
					.join(path);
			}
			else {
				new_path = path.into();
			}

			let result = search_audio(new_path);
			if let Err(e) = result {
				println!("{}", e);
				continue;
			}
			results.push(result?);			
			println!("Added '{}' to '{}'", path.to_string_lossy(), &playlist)
		}

		let list = self.search_playlist_mut(&playlist)?;

		list.add_songs(results);
		Ok(())
	}
	pub fn sub_audio<P: AsRef<str>>(&mut self, playlist: P, songs: &Vec<String>) -> Result<()> {
		let playlist = playlist.as_ref();
		let list = self.search_playlist_mut(&playlist)?;

		list.sub_songs(songs);

		Ok(())
	}
	pub fn set_playlist_name(&mut self, name: String, new_name: String) -> Result<()> {
		Ok(self.search_playlist_mut(name)?
			.set_name(new_name))
	}
	fn search_playlist<S: AsRef<str>>(&self, playlist: S) -> Result<&Playlist> {

		let playlist = playlist.as_ref();
		for list in &*self.all {
			if list.name() == playlist {
				return Ok(&list);
			}
		}
		Err(format!("Playlist '{playlist}' not found"))
	}

	fn search_playlist_mut<S: AsRef<str>>(&mut self, playlist: S) -> Result<&mut Playlist> {
		let all = &mut self.all;

		let playlist = playlist.as_ref();
		for list in all.iter_mut() {
			if list.name() == playlist {
				return Ok(&mut *list);
			}
		}
		Err(format!("Playlist '{playlist}' not found"))
	}

	fn playlist_exists<S: AsRef<str>>(&self, playlist: S) -> bool {
		if let Ok(_) = self.search_playlist(playlist) {
			return true;
		}
		false
	}
}

pub fn parse(cmd: &str, app: &mut AppState) -> Result<bool> {
	
	type C = Commands;

	let music_dir = &app.config.player.music_directory;
	//let download_dir = &app.config.downloader.download_path;
	let config_path: PathBuf = std::env::home_dir()
		.ok_or_else(||"no home directory")?
		.join(".config/audio_monkey/config.toml");

	let playlist_path: PathBuf = std::env::home_dir()
		.ok_or_else(||"no home directory")?
		.join(".local/share/audio_monkey/playlist.json");
	
	let args = shlex::split(cmd).ok_or("invalid quotes")?;
	let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
	match cli.commands {
		C::Play{ playlist, shuffle, value } => { 
			//TODO: make loop work
			if let Some(p) = value {
				if playlist {
					let list = app.search_playlist(p)?;
					let mut new_list = list.clone();

					if shuffle != app.config.player.default_shuffle {
						new_list.songs.shuffle(&mut rand::rng());
					}
					app.player.playlist(&new_list);
					app.current_playlist = Some(new_list);

				}
				else {
					let p = Path::new(&p);
					let audio: Audio;
					if p.is_relative() {
						let path = PathBuf::from(&app.config.player.music_directory)
							.join(&p);
						audio = search_audio(path)?;
					}
					else {
						audio = search_audio(p)?;
					}
					app.player.play_audio(&audio)?;
				}
			}
			else {
				app.player.play(); 
			}
		}
		C::Pause => {
			app.player.pause();
		}
		C::PlayPause => {
			app.player.playpause();
		} 
		C::Download { playlist, format, url, name } => {
			if let Err(e) = download_audio(
				app,
				playlist, 
				format, 
				name,
				url.clone() 
			) {
				println!("Failed to download {} due to {}", url, e); 
			}
		}
		C::Playlist { option } => {
			parse_playlist_command(app, option, playlist_path)?
		}
		C::Config { option } => parse_config_command(app, option, config_path)?,
		C::Duration { path } => {
			let path = Path::new(&path);
			let new_path: PathBuf;

			let music_dir = &app.config.player.music_directory;

			if path.is_relative() {
				new_path = Path::new(music_dir).join(path);
			}
			else {
				new_path = path.into();
			}

			let audio: Audio = search_audio(new_path)?;

			println!("{}", format_from_secs(audio.duration()));

		}
		C::Skip => app.player.skip(),
		C::Volume { value } => {
			if 0.0 > value || value > 1.0 {
				return Err("Value must be between 0.0 and 1.0".to_string());
			}
			app.player.set_volume(value);
		}
		C::Loop { enabled } => app.player.looping = enabled,
		C::Exit => {
			std::io::stdout().flush().map_err(|e| e.to_string())?;
			return Ok(true);
		}
		
		
		
	} 
	Ok(false)
}

pub fn readline() -> Result<String> {
	let mut input = String::new();

	std::io::stdout().flush().map_err(|e| e.to_string())?;
	std::io::stdin()
		.read_line(&mut input)
		.map_err(|e| e.to_string())?;
	Ok(input)
}

fn download_audio(app: &AppState, playlist: bool, format: Option<String>, name: Option<String>, url: String) -> Result<()> {
	

	let download_path: String = app.config.downloader.download_path.clone() + "/";
	let default_format: String = app.config.downloader.format.clone();
	let name: &str = &(download_path+&name.unwrap_or(String::from("%(title)s.%(ext)s")));
	let format: &str = &format.unwrap_or(default_format);
	
	let mut args = vec![
		"-x", 
		"--audio-format", 
		format, 
		"-o",
		name
	];

	if playlist {
		args.push("--yes-playlist");
	}
	else {
		args.push("--no-playlist");
	}
	
	args.push(&url);

	std::process::Command::new("yt-dlp")
		.args(args)
		.status()
		.map_err(|e| e.to_string())?;

	Ok(())
}

fn get_children<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>> {
	let path = path.as_ref();
	
	let dir_entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;

	let mut file_paths: Vec<PathBuf> = Vec::new();
	
	for entry in dir_entries {
		let entry = entry.map_err(|e| e.to_string())?;
		let path = entry.path();
		file_paths.push(path);
	}
	
	Ok(file_paths)
}

fn format_from_secs(secs: u64) -> String {
	let s: u64 = secs % 60;
	let m: u64 = secs / 60; 
	let h: u64 = m / 60;

	if h != 0 { 
		return format!("{0}:{1:02}:{2:02}", h, m % 60, s)
	}
	format!("{:02}:{:02}", m, s)
}

#[allow(dead_code, unused_variables)]
fn parse_playlist_command(app: &mut AppState, option: PlaylistOptions, playlist_path: PathBuf) -> Result<()> {
	type PO = PlaylistOptions;

	match option {
		PO::Add { playlist, songs } => {
			app.add_audio(playlist, songs)?;
		}	
		PO::Sub { playlist, songs } => {
			app.sub_audio(&playlist, &songs)?;
			println!("removed {:#?} from {}", songs, playlist);
		}
		PO::Rename { playlist, new_name } => {
			app.set_playlist_name(playlist.clone(), new_name.clone())?;
			println!("{} => {}", playlist, new_name);
		}
		PO::Create { name } => {
			if app.playlist_exists(&name) {
				return Err(format!("'{name}' already exists"));
			}
			let mut playlist = Playlist::default();
			playlist.set_name(name.clone());
			app.all.push(playlist);
			println!("Created '{}'", name);
		}
		PO::Save => {
			crate::data::save(&playlist_path, &app.all)?;
			println!("Successfully saved playlists");
		}
		PO::List { verbose, playlist } => {
			let list = app.search_playlist(playlist)?;
			let list_name = list.name();
			let list_count = list.count();
			let mut buffer = format!("{list_name} ({list_count})\n");

			for song in &list.songs {
				let song_name = song.name();
				buffer = buffer + &format!("\t{song_name}\n");
				if verbose {
					let song_duration = format_from_secs(song.duration());
					let song_path = song.path().to_string_lossy();
					buffer = buffer + &format!("\t\t{song_duration}\n\t\t{song_path}\n");
				}
			}
			println!("{}", buffer);
		}
	}
	Ok(())
}

fn search_audio<P: AsRef<Path>>(path: P) -> Result<Audio> {
	let path = path.as_ref();	
	if !path.exists() {
		return Err(String::from("path does not exist"));
	}
	if !path.is_file() {
		return Err(String::from("path is not a file"));
	}
	
	Audio::new(path)

}

fn parse_config_command(app: &mut AppState, option: ConfigOptions, dir: PathBuf) -> Result<()> {

	type CO = ConfigOptions;
	
	match option {
		CO::Get => {
			println!("{}", app.config);
		}
		CO::Save => {
			crate::cfg::save(dir, app.config)?;
			println!("Successfully saved config");
		}
		CO::Set { key, value } => {
			config_set(app, key.clone(), value.clone())?;
			println!("Set {:#?} to {}", key, value);
		}
	}
	Ok(())
}

fn config_set(app: &mut AppState, key: ConfigKey, value: String) -> Result <()> {
	use crate::cfg::Color;
	use std::str::FromStr;

	type CK = ConfigKey;

	let player = &mut app.config.player; 
	let downloader = &mut app.config.downloader; 
	let color = &mut app.config.color; 

	match key {
		CK::MusicDirectory  => player.music_directory = value,
		CK::DefaultVolume   => player.default_volume = value.parse::<f32>().map_err(|e| e.to_string())?,
		CK::DefaultPlayback => player.default_playback = value.parse::<f32>().map_err(|e| e.to_string())?,
		CK::DefaultLoop     => player.default_loop = value.parse::<bool>().map_err(|e| e.to_string())?,
		CK::DefaultShuffle  => player.default_shuffle = value.parse::<bool>().map_err(|e| e.to_string())?,
		CK::DownloadPath    => downloader.download_path = value,
		CK::Options         => downloader.options = value,
		CK::Format          => downloader.format = value,
		CK::Background      => color.background = Color::from_str(&value)?
	}	
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_format() {
		assert_eq!(&format_from_secs(0), "00:00");
		assert_eq!(&format_from_secs(60), "01:00");
		assert_eq!(&format_from_secs(3600), "1:00:00");
		assert_eq!(&format_from_secs(4382), "1:13:02");
	}	

}
