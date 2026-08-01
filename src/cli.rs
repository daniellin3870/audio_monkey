use clap::{Parser, Subcommand, Args};
use std::io::Write;

use crate::player::{self, Audio, Player};
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
	List,
	Set {
		option: String,
		value: String,
	},
	Exit
}

pub struct AppState<'a> {
	pub player: &'a mut Player,
	pub config: &'a mut Config,
}

pub fn parse(cmd: &str, app: &mut AppState) -> Result<bool, String> {
	
	let music_dir = &app.config.player.music_directory;
	let download_dir = &app.config.downloader.download_path;
	
	let args = shlex::split(cmd).ok_or("invalid quotes")?;
	let cli = Cli::try_parse_from(args).map_err(|e| e.to_string())?;
	match cli.commands {
		Commands::Play{ path } => { 
			if path == None {
				app.player.play(); 
			}
			else {
				let path = path.unwrap();
				let audio = Audio::new(std::path::PathBuf::from(path))?;
				app.player.play_audio(audio)?;
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
		Commands::List => { 
			let names = get_names_from_dir(music_dir)?;
			for name in names {
				println!("{name}");
			}
		}
		Commands::Set { option: _, value: _ } => todo!(),
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

	let output = std::process::Command::new("yt-dlp")
		.args(args)
		.status()
		.map_err(|e| e.to_string())?;

	Ok(())
}

fn get_names_from_dir(path: &String) -> Result<Vec<String>, String> {
	let path = std::path::PathBuf::from(path);
	
	let dir_entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;

	let mut file_names: Vec<String> = Vec::new();
	
	for entry in dir_entries {
		let entry = entry.map_err(|e| e.to_string())?;
		let name = entry.file_name().into_string().unwrap_or("INVALID FILE NAME".to_string());
		file_names.push(name);
	}
	
	Ok(file_names)
}
