use clap::{ValueEnum, Parser, Subcommand}; 

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::player::{Audio, Player, Playlist};
use crate::cfg::Config;

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
		path: Option<String>
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
		option: ConfigOption,
	},
	Playlist {
		#[command(subcommand)]
		option: PlaylistOptions,
	},
	Exit
}


#[derive(Subcommand, Clone, Debug)]
enum ConfigOption {
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
	Volume,
	PlaybackSpeed,
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
		song: String
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
		playlist: String 
	},
}


pub struct AppState<'a> {
	pub player: &'a mut Player,
	pub config: &'a mut Config,
	pub all:    &'a mut Vec<Playlist>,
}

impl<'a> AppState<'a> {
	//TODO: add functionality for relative and absolute paths
	pub fn add_audio<P: AsRef<Path>>(&mut self, playlist: String, songs: Vec<P>) -> Result<(), String> {
		let mut results: Vec<Audio> = Vec::new();
		for song in songs {
			let song = song.as_ref();
			let path: PathBuf;

			if song.is_relative() {
				path = Path::new(&self.config.player.music_directory)
					.join(song);
			}
			else {
				path = song.to_path_buf();
			}

			let result = search_audio(path);
			if let Err(e) = result {
				println!("{}", e);
				continue;
			}
			results.push(result?);			
			println!("Added '{}' to '{}'", song.to_string_lossy(), &playlist)
		}

		let list = self.search_playlist_mut(&playlist)?;

		list.add_songs(results);
		Ok(())
	}
	pub fn sub_audio<P: AsRef<Path>>(&mut self, playlist: String, song: P) -> Result<(), String> {
		let song = song.as_ref();
		let list = self.search_playlist_mut(&playlist)?;
		for i in 0..list.songs.len() {
			if list.songs[i].path() == song {
				list.songs.remove(i);
				println!("removed {}", song.display());
				return Ok(());
			}	
		}
		let song = song.display();
		Err(format!("'{song}' not found in '{playlist}'"))
	}
	pub fn set_playlist_name(&mut self, name: String, new_name: String) -> Result<(), String> {
		Ok(self.search_playlist_mut(name)?
			.set_name(new_name))
		//for list in self.all.iter_mut() {
		//	if list.name() == &name {
		//		list.set_name(new_name);
		//		return Ok(());
		//	}
		//}
		//Err(format!("playlist \"{name}\" not found"))
	}
	fn search_playlist<S: AsRef<str>>(&self, playlist: S) -> Result<&Playlist, String> {

		let playlist = playlist.as_ref();
		for list in &*self.all {
			if list.name() == playlist {
				return Ok(&list);
			}
		}
		Err(format!("Playlist '{playlist}' not found"))
	}

	fn search_playlist_mut<S: AsRef<str>>(&mut self, playlist: S) -> Result<&mut Playlist, String> {
		let all = &mut self.all;

		let playlist = playlist.as_ref();
		for list in all.iter_mut() {
			if list.name() == &playlist{
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

pub fn parse(cmd: &str, app: &mut AppState) -> Result<bool, String> {
	
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
		//TODO: add playlist functionality
		Commands::Play{ playlist: _, path } => { 
			if let Some(p) = path {
				let audio = Audio::new(&p)?;
				app.player.play_audio(audio)?;
			}
			else {
				app.player.play(); 
			}
		}
		Commands::Pause => {
			app.player.pause();
		}
		Commands::PlayPause => {
			app.player.playpause();
		} 
		Commands::Download { playlist, format, url, name} => {
			if let Err(e) = download_audio(
				app,
				playlist, 
				format, 
				name,
				url.clone() 
			) {
				println!("Failed to download {} due to {}", 
					url, 
					e
				); 
			}
		}
		Commands::Playlist { option } => {
			parse_playlist_command(app, option, playlist_path)?
		}
		Commands::Config { option } => parse_config_command(app, option, config_path)?,
		Commands::Exit => {
			std::io::stdout().flush().map_err(|e| e.to_string())?;
			return Ok(true);
		}
		
		
		
	} 
	Ok(false)
}

pub fn readline() -> Result<String, String> {
	let mut input = String::new();

	print!("> ");
	std::io::stdout().flush().map_err(|e| e.to_string())?;
	std::io::stdin()
		.read_line(&mut input)
		.map_err(|e| e.to_string())?;
	Ok(input)
}

fn download_audio(app: &AppState, playlist: bool, format: Option<String>, name: Option<String>, url: String) -> Result<(), String> {
	

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

fn get_children<P: AsRef<Path>>(path: P) -> Result<Vec<PathBuf>, String> {
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

	if m / 60 != 0 { 
		return format!("{0}:{1:02}:{2:02}", m / 60, m, s)
	}
	format!("{:02}:{:02}", m, s)
}

#[allow(dead_code, unused_variables)]
fn parse_playlist_command(app: &mut AppState, option: PlaylistOptions, playlist_path: PathBuf) -> Result<(), String> {
	use PlaylistOptions::*;
	//TODO: allow multiple songs for add and sub
	//TODO: finish the rest of the options


	match option {
		Add { playlist, songs } => {
			app.add_audio(playlist, songs)?;
		}	
		Sub { playlist, song } => {
			app.sub_audio(playlist, song)?;
		}
		Rename { playlist, new_name } => app.set_playlist_name(playlist, new_name)?,
		Create { name } => {
			if app.playlist_exists(&name) {
				return Err(format!("'{name}' already exists"));
			}
			let mut playlist = Playlist::default();
			playlist.set_name(name);
			app.all.push(playlist);
		}
		Save => {
			crate::data::save(&playlist_path, &app.all)?;
		},
		List { playlist } => {

		}
	}
	Ok(())
}

fn search_audio<P: AsRef<Path>>(path: P) -> Result<Audio, String> {
	let path = path.as_ref();	
	if !path.exists() {
		return Err(String::from("path does not exist"));
	}
	if !path.is_file() {
		return Err(String::from("path is not a file"));
	}
	
	Audio::new(path)

}

fn parse_config_command(app: &mut AppState, option: ConfigOption, dir: PathBuf) -> Result<(), String> {
	use ConfigOption::*;
	match option {
		Get => {
			//TODO: make it print prettier
			println!("{:#?}", app.config);
			Ok(())
		}
		Save => {
			crate::cfg::save(dir, app.config) 
		}
		Set { key, value } => {
			config_set(app, key, value)
		}
	}
}

fn config_set(app: &mut AppState, key: ConfigKey, value: String) -> Result <(), String> {
	use ConfigKey::*;
	use crate::cfg::Color;
	use std::str::FromStr;

	let player = &mut app.config.player; 
	let downloader = &mut app.config.downloader; 
	let color = &mut app.config.color; 

	match key {
		MusicDirectory => player.music_directory = value,
		Volume         => player.volume = value.parse::<f64>().map_err(|e| e.to_string())?,
		PlaybackSpeed  => player.playback_speed = value.parse::<f64>().map_err(|e| e.to_string())?,
		DownloadPath   => downloader.download_path = value,
		Options        => downloader.options = value,
		Format         => downloader.format = value,
		Background     => color.background = Color::from_str(&value)?
	}	
	Ok(())
}

