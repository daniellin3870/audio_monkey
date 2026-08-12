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
	List {
		#[arg(
			short = 'v',
			default_value_t = false
		)]
		verbose: bool,
		playlist: Option<String>
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


#[derive(Subcommand, Clone, Debug, PartialEq)]
enum PlaylistOptions {
	Add {
		song: String
	},
	Sub {
		song: String
	},
	Load {
		name: String	
	},
	Rename {
		name: String
	},
	Create {
		name: String
	},
	Save,

}

pub struct AppState<'a> {
	pub player: &'a mut Player,
	pub config: &'a mut Config,
	pub loaded: Option<Playlist>
}

impl<'a> AppState<'a> {
	fn load(&mut self, playlist: Playlist) {
		self.loaded = Some(playlist);
	}
}

pub fn parse(cmd: &str, app: &mut AppState) -> Result<bool, String> {
	
	let music_dir = &app.config.player.music_directory;
	//let download_dir = &app.config.downloader.download_path;
	let config_path: PathBuf = std::env::home_dir()
		.ok_or_else(||"no home directory")?
		.join(".config/audio_monkey/config.toml");
	
	let args = shlex::split(cmd).ok_or("invalid quotes")?;
	let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
	match cli.commands {
		Commands::Play{ path } => { 
			if let Some(p) = path {
				let audio = Audio::new(std::path::PathBuf::from(p))?;
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
		Commands::List { verbose, playlist: _ } => { 
			let songs = get_songs(verbose, None, music_dir)?;
			println!("{}", songs);
		}
		Commands::Playlist { option: _ } => todo!(),
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

fn get_paths_from_dir(path: &String) -> Result<Vec<PathBuf>, String> {
	let path = std::path::PathBuf::from(path);
	
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

fn get_songs(v: bool, playlist: Option<String>, dir: &String) -> Result<String, String>{
	//TODO: list songs in playlist
	todo!();
	let mut result = String::new();

	//if !playlist.is_none() {
	//	let playlist = Player::search_playlist(playlist.unwrap())?;
	//	
	//	let audios = playlist.get_songs();	

	//	result.push(playlist.get_name() + "\n");
	//	
	//	for audio in audios {
	//		result.push(audio.get_name() + "\n");
	//		if v {
	//			result.push_str(&(
	//				format!("  {}\n  {}", 
	//					format_from_secs(*audio.get_duration()), 
	//					audio.get_path().to_str().unwrap_or("invalid characters")
	//			))); 
	//		}
	//	} 
	//	return Ok(result);
	//}

	let paths = get_paths_from_dir(dir)?;
	
	for path in paths {
		let audio = Audio::new(path)?;
		result.push_str(&(format!("\n{}", audio.get_name())));
		if v {
			result.push_str(&(
				format!("\n  {}\n  {}", 
					format_from_secs(*audio.get_duration()), 
					audio.get_path().to_str().unwrap_or("invalid characters")
			)));
		}
	}

	result.push('\n');

	Ok(result)
}

#[allow(dead_code, unused_variables)]
fn parse_playlist_command(app: &mut AppState, option: PlaylistOptions, value: String) -> Result<(), String> {
	use PlaylistOptions::*;
	//TODO: allow multiple songs for add and sub
	//TODO: optimize with hashmaps and ish
	//TODO: finish the rest of the options
	if app.loaded.is_none() && matches!(&option, PlaylistOptions::Create { name: _ } | PlaylistOptions::Load { name: _ }) {
		return Err(String::from("no playlist loaded"));
	}

	let loaded = app.loaded.as_mut().unwrap();
	match option {
		Add { song } => {
			loaded.songs.push(search_audio(&Path::new(&song))?);
		}	
		Sub { song } => {
			for i in 0..*loaded.get_count() as usize {
				if *loaded.songs[i].get_name() == song {
					loaded.songs.remove(i);
					return Ok(())
				}
			}
		}
		Load { name: _ } => {
			todo!();
		}
		Rename { name } => loaded.set_name(name),
		Create { name } => {
			let mut playlist: Playlist = Playlist::default();
			playlist.set_name(name);
			app.load(playlist);
		}
		Save =>todo!(),
	}
	Ok(())
}

fn search_audio(path: &Path) -> Result<Audio, String> {
	
	if !path.exists() {
		return Err(String::from("path does not exist"));
	}
	if !path.is_file() {
		return Err(String::from("path is not a file"));
	}
	
	Audio::new(path.to_owned())

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
