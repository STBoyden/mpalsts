use std::{
	fs::{self, File},
	io::{self, BufWriter},
};

use directories::ProjectDirs;
use ron::{de::SpannedError, ser::PrettyConfig};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Debug, Error)]
pub enum AppStateError {
	#[error("failed to config directory")]
	NoConfigDir,

	#[error("file read or write error: {0}")]
	IO(#[from] io::Error),

	#[error("failed to read config: {0}")]
	ReadConfig(#[from] SpannedError),

	#[error("failed to serialize config: {0}")]
	SerializeConfig(#[from] ron::Error),
}

pub type Result<T> = std::result::Result<T, AppStateError>;

impl AppState {
	pub fn read_config() -> Result<Self> {
		let project_dir =
			ProjectDirs::from("com", "stboyden", "mpalsts").ok_or(AppStateError::NoConfigDir)?;

		let config_dir = project_dir.config_dir();

		fs::create_dir_all(config_dir)?;

		let config_ron = config_dir.join("config.ron");

		let exists = fs::exists(&config_ron);
		if exists.is_err() || !exists? {
			return Ok(Self::default());
		}

		let config_ron = File::open(&config_ron)?;
		let config: AppState = ron::de::from_reader(config_ron)?;

		return Ok(config);
	}

	pub fn save_config(&self) -> Result<()> {
		let project_dir =
			ProjectDirs::from("com", "stboyden", "mpalsts").ok_or(AppStateError::NoConfigDir)?;

		let config_dir = project_dir.config_dir();
		fs::create_dir_all(config_dir)?;

		let config_ron = config_dir.join("config.ron");
		let config_ron = BufWriter::new(File::create(&config_ron)?);
		ron::Options::default().to_io_writer_pretty(config_ron, self, PrettyConfig::new())?;

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
