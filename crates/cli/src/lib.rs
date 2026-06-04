use clap::{Arg, ArgAction, Command, value_parser};
use dotenv::dotenv;
#[cfg(feature = "include_gui")]
use mpalsts_gui::run_with_state as _gui_run_with_state;
use mpalsts_shared::{
	AppState, MAX_LUMENS_THRESHOLD, MAX_SECONDS_THRESHOLD, MIN_LUMENS_THRESHOLD,
	MIN_SECONDS_THRESHOLD,
};
use thiserror::Error;
use tracing::{trace, warn};

use crate::runtime::{RuntimeMode, runtime};

mod runtime;

const DAEMON_ARG: &str = "daemon";
const SAVE_ARG: &str = "save";
const CONFIG_COMMAND: &str = "config";
const LUMENS_THRESHOLD_ARG: &str = "lumens-threshold";
const TIME_THRESHOLD_ARG: &str = "time-threshold";
#[cfg(target_os = "linux")]
const SENSOR_ARG: &str = "sensor";
#[cfg(target_os = "linux")]
const LIGHT_THEME_ARG: &str = "light-theme";
#[cfg(target_os = "linux")]
const DARK_THEME_ARG: &str = "dark-theme";

fn daemon_run(state: AppState) -> Result<(), CliError> {
	trace!("Running in daemon mode");

	runtime(Some(RuntimeMode::Daemon), state)?;
	return Ok(());
}

#[cfg(feature = "include_gui")]
fn gui_run(state: AppState) -> Result<(), CliError> {
	trace!("Running in GUI mode");

	_gui_run_with_state(state);
	return Ok(());
}

#[derive(Debug, Error)]
pub enum CliError {
	#[error("config error: {0}")]
	ConfigError(#[from] mpalsts_shared::AppStateError),

	#[error("runtime error: {0}")]
	RuntimeError(#[from] runtime::RuntimeError),

	#[error("invalid {name}: {value}; expected a value from {min} to {max}")]
	InvalidThreshold {
		name: &'static str,
		value: f32,
		min: f32,
		max: f32,
	},

	#[error("unexpected error: {0}")]
	UnexpectedError(#[from] anyhow::Error),
}

fn validate_threshold(name: &'static str, value: f32, min: f32, max: f32) -> Result<f32, CliError> {
	if (min..=max).contains(&value) {
		return Ok(value);
	}

	return Err(CliError::InvalidThreshold {
		name,
		value,
		min,
		max,
	});
}

fn apply_cli_settings(state: &mut AppState, matches: &clap::ArgMatches) -> Result<(), CliError> {
	if let Some((CONFIG_COMMAND, _)) = matches.subcommand() {
		return Ok(());
	}

	if let Some(lumens_threshold) = matches.get_one::<f32>(LUMENS_THRESHOLD_ARG).copied() {
		state.lumens_threshold = validate_threshold(
			"lumens threshold",
			lumens_threshold,
			MIN_LUMENS_THRESHOLD,
			MAX_LUMENS_THRESHOLD,
		)?;
	}

	if let Some(seconds_threshold) = matches.get_one::<f32>(TIME_THRESHOLD_ARG).copied() {
		state.seconds_threshold = validate_threshold(
			"time threshold",
			seconds_threshold,
			MIN_SECONDS_THRESHOLD,
			MAX_SECONDS_THRESHOLD,
		)?;
	}

	#[cfg(target_os = "linux")]
	{
		if let Some(sensor) = matches.get_one::<String>(SENSOR_ARG) {
			state.linux.sensor = Some(sensor.clone());
		}

		if let Some(light_theme) = matches.get_one::<String>(LIGHT_THEME_ARG) {
			state.linux.light_theme = Some(light_theme.clone());
		}

		if let Some(dark_theme) = matches.get_one::<String>(DARK_THEME_ARG) {
			state.linux.dark_theme = Some(dark_theme.clone());
		}
	}

	return Ok(());
}

fn config_subcommand() -> Command {
	let mut command = Command::new(CONFIG_COMMAND)
		.about("Print current config values")
		.subcommand_required(true)
		.arg_required_else_help(true)
		.subcommand(Command::new(LUMENS_THRESHOLD_ARG).about("Print the ambient light threshold"))
		.subcommand(Command::new(TIME_THRESHOLD_ARG).about("Print the time threshold"));

	#[cfg(target_os = "linux")]
	{
		command = command
			.subcommand(Command::new(SENSOR_ARG).about("Print the configured Linux sensor"))
			.subcommand(Command::new(LIGHT_THEME_ARG).about("Print the configured Linux light theme"))
			.subcommand(Command::new(DARK_THEME_ARG).about("Print the configured Linux dark theme"));
	}

	return command;
}

#[cfg(target_os = "linux")]
fn print_optional_config_value(value: Option<&String>, fallback: &str) {
	match value {
		Some(value) => println!("{value}"),
		None => println!("{fallback}"),
	}
}

fn print_config_value(state: &AppState, matches: &clap::ArgMatches) -> bool {
	let Some((CONFIG_COMMAND, config_matches)) = matches.subcommand() else {
		return false;
	};

	match config_matches.subcommand() {
		Some((LUMENS_THRESHOLD_ARG, _)) => println!("{}", state.lumens_threshold),
		Some((TIME_THRESHOLD_ARG, _)) => println!("{}", state.seconds_threshold),
		#[cfg(target_os = "linux")]
		Some((SENSOR_ARG, _)) => print_optional_config_value(state.linux.sensor.as_ref(), "auto"),
		#[cfg(target_os = "linux")]
		Some((LIGHT_THEME_ARG, _)) => {
			print_optional_config_value(state.linux.light_theme.as_ref(), "auto")
		}
		#[cfg(target_os = "linux")]
		Some((DARK_THEME_ARG, _)) => print_optional_config_value(state.linux.dark_theme.as_ref(), "auto"),
		_ => return false,
	}

	return true;
}

pub fn main() -> Result<(), CliError> {
	dotenv().ok();
	tracing_subscriber::fmt::try_init().map_err(|err| anyhow::anyhow!(err))?;

	let daemon_arg = Arg::new(DAEMON_ARG)
		.short('d')
		.long(DAEMON_ARG)
		.action(ArgAction::SetTrue)
		.default_value(if cfg!(feature = "include_gui") {
			"false"
		} else {
			"true"
		})
		.help(if cfg!(feature = "include_gui") {
			"Run as a daemon"
		} else {
			"Run as a daemon (default for no-GUI builds)"
		})
		.long_help(if cfg!(feature = "include_gui") {
			"Run as a daemon"
		} else {
			"Run as a daemon. This is the default when the binary is compiled without the `include_gui` feature."
		});

	let save_arg = Arg::new(SAVE_ARG)
		.long(SAVE_ARG)
		.action(ArgAction::SetTrue)
		.help("Save CLI settings to the config file for future runs");

	let lumens_threshold_arg = Arg::new(LUMENS_THRESHOLD_ARG)
		.long(LUMENS_THRESHOLD_ARG)
		.alias("lumen-threshold")
		.value_name("LUMENS")
		.value_parser(value_parser!(f32))
		.help(format!(
			"Ambient light threshold in lumens ({MIN_LUMENS_THRESHOLD}-{MAX_LUMENS_THRESHOLD})"
		));

	let time_threshold_arg = Arg::new(TIME_THRESHOLD_ARG)
		.long(TIME_THRESHOLD_ARG)
		.alias("seconds-threshold")
		.alias("second-threshold")
		.value_name("SECONDS")
		.value_parser(value_parser!(f32))
		.help(format!(
			"Time threshold in seconds ({MIN_SECONDS_THRESHOLD}-{MAX_SECONDS_THRESHOLD})"
		));

	let mut cmd = Command::new("mpalsts")
		.arg(daemon_arg)
		.arg(save_arg)
		.arg(lumens_threshold_arg)
		.arg(time_threshold_arg)
		.subcommand(config_subcommand());

	#[cfg(target_os = "linux")]
	{
		let sensor_arg = Arg::new(SENSOR_ARG)
			.long(SENSOR_ARG)
			.value_name("PATH")
			.value_parser(value_parser!(String))
			.help("Linux illumination sensor file to use, for example /sys/bus/iio/devices/iio:device0/in_illuminance_raw");

		let light_theme_arg = Arg::new(LIGHT_THEME_ARG)
			.long(LIGHT_THEME_ARG)
			.value_name("THEME")
			.value_parser(value_parser!(String))
			.help("Linux light theme name to switch to");

		let dark_theme_arg = Arg::new(DARK_THEME_ARG)
			.long(DARK_THEME_ARG)
			.value_name("THEME")
			.value_parser(value_parser!(String))
			.help("Linux dark theme name to switch to");

		cmd = cmd.arg(sensor_arg).arg(light_theme_arg).arg(dark_theme_arg);
	}

	let matches = cmd.get_matches();

	let mut state = match AppState::read_config() {
		Ok(state) => state,
		Err(error) => {
			warn!("failed to read config, using defaults: {error}");
			AppState::default()
		}
	};

	if print_config_value(&state, &matches) {
		return Ok(());
	}

	apply_cli_settings(&mut state, &matches)?;

	if matches.get_flag(SAVE_ARG) {
		state.save_config()?;
	}

	let run_as_daemon = matches.get_flag(DAEMON_ARG);

	#[cfg(feature = "include_gui")]
	{
		if run_as_daemon {
			return daemon_run(state);
		} else {
			return gui_run(state);
		}
	}
	#[cfg(not(feature = "include_gui"))]
	{
		if run_as_daemon {
			return daemon_run(state);
		} else {
			use anyhow::anyhow;

			return Err(anyhow!("Somehow got to an unreachable code path").into());
		}
	}
}
