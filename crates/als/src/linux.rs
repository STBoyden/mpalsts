use std::{
	cell::{RefCell, RefMut},
	fs::File,
	io::{BufRead, BufReader},
	path::PathBuf,
};

use glob::glob;
use thiserror::Error;

use crate::{LightSensor, SensorOutput, error::ALSError};

#[derive(Error, Debug)]
pub enum LinuxALSError {
	#[error("no illumination device")]
	NoSensor,

	#[error("failed to read sensor: {0}")]
	Io(#[from] std::io::Error),

	#[error("could not parse sensor reading")]
	ParseError(#[from] std::num::ParseIntError),
}

type Result<T> = std::result::Result<T, LinuxALSError>;

pub struct LinuxSensorReader {
	device_file:             RefCell<Option<PathBuf>>,
	pub(crate) backup_files: Vec<PathBuf>,
	#[cfg(test)]
	pub(crate) _test:        i32,
}

impl LinuxSensorReader {
	pub fn new() -> RefCell<Self> {
		let mut device_file: Option<PathBuf> = None;
		let mut backup_files: Vec<PathBuf> = Vec::new();

		for entry in glob("/sys/bus/iio/devices/iio:device*/in_illuminance_raw")
			.expect("Failed to read glob pattern")
			.flatten()
		{
			if device_file.is_some() {
				backup_files.push(entry);
			} else {
				device_file = Some(entry);
			}
		}

		return RefCell::new(Self {
			device_file: RefCell::new(device_file),
			backup_files,
			#[cfg(test)]
			_test: 0,
		});
	}

	pub fn get_current_device_file(&self) -> Option<PathBuf> {
		self.device_file.borrow().clone()
	}

	pub fn get_illumination_files(&self) -> Vec<String> {
		let mut files = self
			.device_file
			.borrow()
			.clone()
			.map(|p| vec![p.to_string_lossy().into_owned()])
			.unwrap_or_default();

		files.extend(
			self
				.backup_files
				.iter()
				.map(|p| p.to_string_lossy().into_owned())
				.collect::<Vec<_>>(),
		);

		return files;
	}

	pub fn set_device_file(&mut self, device_file: Option<PathBuf>) {
		let Some(device_file) = device_file else {
			return;
		};

		if matches!(
			self.device_file.borrow().clone(),
			Some(d) if d == device_file
		) {
			return;
		}

		let current = self.device_file.clone().into_inner();

		*self.device_file.borrow_mut() = Some(device_file);

		let Some(current) = current else {
			return;
		};

		self.backup_files.push(current);
	}

	fn take_reading(&mut self) -> Result<u32> {
		if let Some(ref device_file) = *self.device_file.borrow() {
			let fd = File::open(device_file).map_err(LinuxALSError::Io)?;
			let mut buf_reader = BufReader::new(fd);

			let mut line = String::new();
			buf_reader.read_line(&mut line).map_err(LinuxALSError::Io)?;

			let reading = line
				.trim()
				.parse::<u32>()
				.map_err(LinuxALSError::ParseError)?;

			return Ok(reading);
		} else {
			return Err(LinuxALSError::NoSensor);
		}
	}

	pub fn go_to_next_backup(&mut self) {
		if self.backup_files.is_empty() {
			return;
		}

		let current = self.device_file.borrow();
		let next = self.backup_files.first();

		if let (Some(current), Some(next)) = (current.clone(), next) {
			*self.device_file.borrow_mut() = Some(next.clone());
			self.backup_files.remove(0);
			self.backup_files.push(current);
		}
	}

	#[cfg(test)]
	pub(crate) fn test_mutable(&mut self) -> i32 {
		self._test += 1;
		return self._test;
	}
}

impl LightSensor for RefCell<LinuxSensorReader> {
	async fn read(&self) -> crate::Result<SensorOutput> {
		let reading = self
			.borrow_mut()
			.take_reading()
			.map_err(ALSError::Platform)?;
		return Ok(reading as SensorOutput);
	}

	fn has_sensor(&self) -> bool {
		return self.borrow().device_file.borrow().is_some();
	}

	fn mutate_concrete(&self, mutate: impl FnOnce(RefMut<'_, self::LinuxSensorReader>)) {
		let borrow = self.borrow_mut();
		mutate(borrow);
	}
}
