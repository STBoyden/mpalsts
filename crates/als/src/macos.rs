//! MacOS implementation of the ambient light sensor reader.
//!
//! Main functionality has been ported from
//! [DarkModeBuddy's](https://github.com/insidegui/DarkModeBuddy) Objective-C code, which is
//! liscensed under BSD 2-Clause.
use std::{
	cell::{RefCell, RefMut},
	ffi::c_void,
	fmt,
	ptr::NonNull,
};

use objc2::rc::Retained;
use objc2_io_kit::{IOHIDEventStruct, IOHIDServiceClient};
use thiserror::Error;

use crate::{LightSensor, SensorOutput, error::ALSError};

#[allow(
	non_upper_case_globals,
	reason = "constant name matches HID event field name and general Darwin constant naming convention"
)]
const kAmbientLightSensorEvent: i64 = 12;

#[link(name = "BezelServices", kind = "framework")]
unsafe extern "C" {
	fn ALCALSCopyALSServiceClient() -> *mut IOHIDServiceClient;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
	fn CFRelease(cf: *const c_void);
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

fn to_event_field<T>(field: T) -> i32
where
	T: TryInto<i32>,
	T::Error: fmt::Debug,
{
	return field.try_into().expect("cannot fit field value inside i32") << 16;
}

#[derive(Error, Debug)]
pub enum MacOSALSError {
	#[error("device incapable of providing ambient light sensor readings")]
	NoSensor,
}

type Result<T> = std::result::Result<T, MacOSALSError>;

struct CopiedHIDEvent {
	ptr: NonNull<IOHIDEventStruct>,
}

impl CopiedHIDEvent {
	fn new(ptr: *mut IOHIDEventStruct) -> Option<Self> {
		return NonNull::new(ptr).map(|ptr| Self { ptr });
	}

	fn as_ptr(&self) -> *mut IOHIDEventStruct {
		return self.ptr.as_ptr();
	}
}

impl Drop for CopiedHIDEvent {
	fn drop(&mut self) {
		unsafe { CFRelease(self.ptr.as_ptr() as *const c_void) };
	}
}

pub struct MacOSSensorReader {
	client: Option<Retained<IOHIDServiceClient>>,
	#[cfg(test)]
	pub(crate) _test: i32,
}

impl MacOSSensorReader {
	pub fn new() -> RefCell<Self> {
		return RefCell::new(Self {
			client: None,
			#[cfg(test)]
			_test: 0,
		});
	}

	fn copy_hid_event(&mut self) -> Option<CopiedHIDEvent> {
		if self.client.is_none() {
			self.client = unsafe { Retained::from_raw(ALCALSCopyALSServiceClient()) };
		}

		if let Some(ref mut client) = self.client {
			let client_ptr = unsafe {
				std::mem::transmute::<&IOHIDServiceClient, *mut IOHIDServiceClient>(client.as_ref())
			};

			let event_ptr =
				unsafe { IOHIDServiceClientCopyEvent(client_ptr, kAmbientLightSensorEvent, 0, 0) };
			return CopiedHIDEvent::new(event_ptr);
		}

		return None;
	}

	fn can_get_event(&mut self) -> bool {
		let event = self.copy_hid_event();
		return event.is_some();
	}

	fn take_reading(&mut self) -> Result<SensorOutput> {
		let Some(event) = self.copy_hid_event() else {
			return Err(MacOSALSError::NoSensor);
		};

		let value =
			unsafe { IOHIDEventGetFloatValue(event.as_ptr(), to_event_field(kAmbientLightSensorEvent)) };

		return Ok(value);
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
