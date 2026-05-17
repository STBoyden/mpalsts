#![allow(clippy::transmute_ptr_to_ref, clippy::useless_transmute)]

mod error;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
mod tests;

use std::{
	cell::{RefCell, RefMut},
	time::Duration,
};

use async_stream::stream;
use futures::Stream;
use futures_time::task::sleep;

use crate::error::ALSError;
#[cfg(target_os = "linux")]
use crate::linux::LinuxSensorReader as PlatformSensorReader;
#[cfg(target_os = "macos")]
use crate::macos::MacOSSensorReader as PlatformSensorReader;

pub(crate) type Result<T> = std::result::Result<T, ALSError>;

pub type ConcreteSensor = RefCell<PlatformSensorReader>;

pub type SensorOutput = f64;
pub trait LightSensor {
	/// Read the sensor and return the current light level.
	async fn read(&self) -> Result<SensorOutput>;

	/// Returns an infinite stream that polls the sensor at the specified duration.
	fn stream(&self, poll_rate: Duration) -> impl Stream<Item = Result<SensorOutput>> {
		return stream! {
			loop {
				yield self.read().await;
				sleep(poll_rate.into()).await;
			}
		};
	}

	/// Returns whether the platform has a sensor available.
	fn has_sensor(&self) -> bool;

	/// Mutate the concrete sensor reader.
	fn mutate_concrete(&self, mutate: impl FnOnce(RefMut<'_, PlatformSensorReader>));
}

/// Returns whether the platform has a sensor available.
pub fn is_sensor_available() -> bool {
	return PlatformSensorReader::new().has_sensor();
}

/// Returns this platform's sensor reader, if one is available.
///
/// ## Errors
///
/// - (MacOS only) [`MacOSALSError`]: When the sensor reader is not available. This can occur if
///   your MacOS-based system does not have a sensor. Most modern MacBooks, and iMacs will have a
///   sensor.
/// - (Linux only) [`LinuxALSError`]: When the sensor reader is not available. This can occur if
///   your Linux-based system does not have a sensor.
pub fn get_platform_reader() -> ConcreteSensor {
	return PlatformSensorReader::new();
}
