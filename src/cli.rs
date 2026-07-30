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
		url: String,
		#[arg(
			default_value_t = "mp3".to_string(),
		)]
		format: String,
	},
	Exit
}

pub struct AppState<'a> {
	pub player: &'a mut Player,
	pub config: &'a mut Config,
}

pub fn parse(cmd: &str, app: &mut AppState) -> Result<bool, String> {
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
		Commands::Pause | Commands::PlayPause | Commands::Download { playlist: _, url: _, format: _ } => todo!(),
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

fn download_audio(app: &AppState, name: Option<String>, url: String, playlist: bool) -> Result<(), String> {
	let mut args = vec!["-x", "--audio-format", &app.config.downloader.format];
	
	let name: &str = &name.unwrap_or(String::new());

	if !name.is_empty() && !playlist {
		
		args.push("-o");
		args.push(name);
	}
	
	if playlist {
		args.push("--yes-playlist");
	}
	
	args.push(&url);

	let mut command = std::process::Command::new("yt-dlp");
	
	for arg in args {
		command.arg(arg);
	}

	let output = command.output().map_err(|e| e.to_string())?;

	if output.status.success() {
		println!("{}", String::from_utf8_lossy(&output.stdout));
	}
	else {
		println!("{}", String::from_utf8_lossy(&output.stderr));
	}

	Ok(())
}

