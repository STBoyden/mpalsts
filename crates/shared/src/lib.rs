use std::{
	fs::{self, File},
	io::{self, BufWriter},
	sync::LazyLock,
};

use directories::ProjectDirs;
use ron::{de::SpannedError, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub static PROJECT_DIR: LazyLock<Option<ProjectDirs>> =
	LazyLock::new(|| ProjectDirs::from("com", "stboyden", "mpalsts"));

pub const MIN_LUMENS_THRESHOLD: f32 = 10.;
pub const DEFAULT_LUMENS_THRESHOLD: f32 = 100.;
pub const MAX_LUMENS_THRESHOLD: f32 = 2000.;
pub const MIN_SECONDS_THRESHOLD: f32 = 10.;
pub const DEFAULT_SECONDS_THRESHOLD: f32 = 30.;
pub const MAX_SECONDS_THRESHOLD: f32 = 120.;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppState {
	pub enable_theme_switching: bool,
	pub enable_autostart: bool,
	pub lumens_threshold: f32,
	pub seconds_threshold: f32,

	#[cfg(target_os = "linux")]
	pub linux: LinuxState,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LinuxState {
	pub light_theme: Option<String>,
	pub dark_theme: Option<String>,
	pub sensor: Option<String>,
}

#[cfg(target_os = "linux")]
impl Default for LinuxState {
	fn default() -> Self {
		return Self {
			light_theme: None,
			dark_theme: None,
			sensor: None,
		};
	}
}

#[cfg(target_os = "linux")]
impl LinuxState {
	pub fn new() -> Self {
		return Self::default();
	}
}

impl Default for AppState {
	fn default() -> Self {
		return Self {
			enable_theme_switching: false,
			enable_autostart: false,
			lumens_threshold: DEFAULT_LUMENS_THRESHOLD,
			seconds_threshold: DEFAULT_SECONDS_THRESHOLD,
			#[cfg(target_os = "linux")]
			linux: LinuxState::default(),
		};
	}
}

fn sanitise_threshold(value: f32, default: f32, min: f32, max: f32) -> f32 {
	if !value.is_finite() {
		return default;
	}

	return value.clamp(min, max);
}

#[derive(Debug, Error)]
pub enum AppStateError {
	#[error("failed to config directory")]
	NoConfigDir,

	#[error("file read or write error: {0}")]
	IO(#[from] io::Error),

	#[error("failed to read config: {0}")]
	ReadConfig(#[from] SpannedError),

	#[error("failed to serialise config: {0}")]
	SerialiseConfig(#[from] ron::Error),
}

pub type Result<T> = std::result::Result<T, AppStateError>;

impl AppState {
	pub fn sanitised(mut self) -> Self {
		self.sanitise();
		return self;
	}

	pub fn sanitise(&mut self) {
		self.lumens_threshold = sanitise_threshold(
			self.lumens_threshold,
			DEFAULT_LUMENS_THRESHOLD,
			MIN_LUMENS_THRESHOLD,
			MAX_LUMENS_THRESHOLD,
		);
		self.seconds_threshold = sanitise_threshold(
			self.seconds_threshold,
			DEFAULT_SECONDS_THRESHOLD,
			MIN_SECONDS_THRESHOLD,
			MAX_SECONDS_THRESHOLD,
		);
	}

	pub fn read_config() -> Result<Self> {
		let project_dir = PROJECT_DIR.as_ref().ok_or(AppStateError::NoConfigDir)?;

		let config_dir = project_dir.config_dir();

		fs::create_dir_all(config_dir)?;

		let config_ron = config_dir.join("config.ron");

		let exists = fs::exists(&config_ron);
		if exists.is_err() || !exists? {
			return Ok(Self::default());
		}

		let config_ron = File::open(&config_ron)?;
		let config: AppState = ron::de::from_reader(config_ron)?;

		return Ok(config.sanitised());
	}

	pub fn save_config(&self) -> Result<()> {
		let project_dir = PROJECT_DIR.as_ref().ok_or(AppStateError::NoConfigDir)?;

		let config_dir = project_dir.config_dir();
		fs::create_dir_all(config_dir)?;

		let config = self.clone().sanitised();
		let config_ron = config_dir.join("config.ron");
		let config_ron = BufWriter::new(File::create(&config_ron)?);
		ron::Options::default().to_io_writer_pretty(config_ron, &config, PrettyConfig::new())?;

		return Ok(());
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[repr(u8)]
pub enum ThemeMode {
	Light = 0,
	Dark = 1,
}

pub enum ThemeModeError {
	InvalidValue,
}

impl TryFrom<u8> for ThemeMode {
	type Error = ThemeModeError;

	fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
		match value {
			0 => Ok(ThemeMode::Light),
			1 => Ok(ThemeMode::Dark),
			_ => Err(ThemeModeError::InvalidValue),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn sanitised_config_replaces_non_finite_thresholds_with_defaults() {
		let state = AppState {
			lumens_threshold: f32::NAN,
			seconds_threshold: f32::INFINITY,
			..AppState::default()
		}
		.sanitised();

		assert_eq!(state.lumens_threshold, DEFAULT_LUMENS_THRESHOLD);
		assert_eq!(state.seconds_threshold, DEFAULT_SECONDS_THRESHOLD);
	}

	#[test]
	fn sanitised_config_clamps_out_of_range_thresholds() {
		let state = AppState {
			lumens_threshold: -1.,
			seconds_threshold: 999.,
			..AppState::default()
		}
		.sanitised();

		assert_eq!(state.lumens_threshold, MIN_LUMENS_THRESHOLD);
		assert_eq!(state.seconds_threshold, MAX_SECONDS_THRESHOLD);
	}
}
