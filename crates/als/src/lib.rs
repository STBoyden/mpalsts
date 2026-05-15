#![allow(clippy::transmute_ptr_to_ref, clippy::useless_transmute)]

mod error;
#[cfg(target_os = "macos")]
mod macos;
mod tests;

// #[cfg(target_os = "macos")]
// embed_entitlements!(r#"../resources/als.entitlement"#);

use std::time::Duration;

use async_stream::stream;
use futures::Stream;
use futures_time::task::sleep;

use crate::error::ALSError;
#[cfg(target_os = "macos")]
use crate::macos::MacOSSensorReader;

pub(crate) type Result<T> = std::result::Result<T, ALSError>;
pub type SensorOutput = f64;
pub trait LightSensor {
	async fn read(&mut self) -> Result<SensorOutput>;

	/// Returns an infinite stream that polls the sensor at the specified duration.
	fn stream(&mut self, poll_rate: Duration) -> impl Stream<Item = Result<SensorOutput>> {
		return stream! {
			loop {
				yield self.read().await;
				sleep(poll_rate.into()).await;
			}
		};
	}

	fn has_sensor(&self) -> bool;
}

/// Returns whether the platform has a sensor available.
pub fn is_sensor_available() -> bool {
	#[cfg(target_os = "macos")]
	return macos::MacOSSensorReader::new().has_sensor();
}

/// Returns this platform's sensor reader, if one is available.
///
/// ## Errors
///
/// - (MacOS only) [`MacOSALSError`]: When the sensor reader is not available. This can occur if
///   your MacOS-based system does not have a sensor. Most modern MacBooks, and iMacs will have a
///   sensor.
pub async fn get_platform_reader() -> Result<impl LightSensor> {
	#[cfg(target_os = "macos")]
	return Ok(MacOSSensorReader::new());
}
