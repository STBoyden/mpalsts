use std::cell::{RefCell, RefMut};

use objc2::rc::Retained;
use objc2_io_kit::{IOHIDEventStruct, IOHIDServiceClient};
use thiserror::Error;

use crate::{LightSensor, SensorOutput, error::ALSError};

const K_AMBIENT_LIGHT_SENSOR_EVENT: i64 = 12;

#[link(name = "BezelServices", kind = "framework")]
unsafe extern "C" {
	fn ALCALSCopyALSServiceClient() -> *mut IOHIDServiceClient;
}

unsafe extern "C" {
	fn IOHIDServiceClientCopyEvent(
		client: *mut IOHIDServiceClient,
		key: i64,
		a: i32,
		b: i64,
	) -> *mut IOHIDEventStruct;
	fn IOHIDEventGetFloatValue(r#ref: *mut IOHIDEventStruct, field: i32) -> f64;
}

fn to_event_field(field: i32) -> i32 {
	return field << 16;
}

#[derive(Error, Debug)]
pub enum MacOSALSError {
	#[error("device incapable of providing ambient light sensor readings")]
	NoSensor,
}

type Result<T> = std::result::Result<T, MacOSALSError>;

pub struct MacOSSensorReader {
	client:           Option<Retained<IOHIDServiceClient>>,
	#[cfg(test)]
	pub(crate) _test: i32,
}

impl MacOSSensorReader {
	pub fn new() -> RefCell<Self> {
		return RefCell::new(Self {
			client:             None,
			#[cfg(test)]
			_test:              0,
		});
	}

	fn copy_hid_event(&mut self) -> Option<Box<IOHIDEventStruct>> {
		if self.client.is_none() {
			self.client = unsafe { Retained::from_raw(ALCALSCopyALSServiceClient()) };
		}

		if let Some(ref mut client) = self.client {
			let client_ptr = unsafe {
				std::mem::transmute::<&IOHIDServiceClient, *mut IOHIDServiceClient>(client.as_ref())
			};

			let event_ptr =
				unsafe { IOHIDServiceClientCopyEvent(client_ptr, K_AMBIENT_LIGHT_SENSOR_EVENT, 0, 0) };
			let event = unsafe { Box::from_raw(event_ptr) };

			return Some(event);
		}

		return None;
	}

	fn can_get_event(&mut self) -> bool {
		let event = self.copy_hid_event();
		return event.is_some();
	}

	fn take_reading(&mut self) -> Result<SensorOutput> {
		if !self.can_get_event() {
			return Err(MacOSALSError::NoSensor);
		}

		let event = self.copy_hid_event();

		if let Some(mut event) = event {
			let value = unsafe {
				IOHIDEventGetFloatValue(
					event.as_mut() as *mut _,
					to_event_field(K_AMBIENT_LIGHT_SENSOR_EVENT as i32),
				)
			};

			return Ok(value);
		}

		return Ok(0.);
	}

	#[cfg(test)]
	pub(crate) fn test_mutable(&mut self) -> i32 {
		self._test += 1;
		return self._test;
	}
}

impl LightSensor for RefCell<MacOSSensorReader> {
	async fn read(&self) -> super::Result<SensorOutput> {
		return self.borrow_mut().take_reading().map_err(ALSError::Platform);
	}

	fn has_sensor(&self) -> bool {
		return self.borrow_mut().can_get_event();
	}

	fn mutate_concrete(&self, mutate: impl FnOnce(RefMut<'_, MacOSSensorReader>)) {
		let borrow = self.borrow_mut();
		mutate(borrow);
	}
}
