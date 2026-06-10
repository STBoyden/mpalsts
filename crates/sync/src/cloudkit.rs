#[cfg(cloudkit)]
mod _cloudkit {
	use std::{
		ffi::{c_char, c_uchar, c_void},
		sync::mpsc::{self, RecvError},
	};

	use mpalsts_shared::ThemeMode;
	use thiserror::Error;
	use tracing::error;

	use crate::{Result, SyncError, SyncProvider};

	mod ffi {
		use std::ffi::{c_uchar, c_void};

		pub const COULD_NOT_UPDATE_APPEARANCE: i8 = -1;
		pub const INVALID_THEME_MODE: i8 = -2;

		type SaveAppearanceCallback = extern "C" fn(*mut c_void, i8);
		type GetAppearanceCallback = extern "C" fn(*mut c_void, u8);

		#[link(name = "MPALSTSCloudKitSync")]
		unsafe extern "C" {
			pub fn mpalsts_cloudkit_save_appearance(
				theme_mode_raw: c_uchar,
				context: *mut c_void,
				callback: SaveAppearanceCallback,
			);

			pub fn mpalsts_cloudkit_get_appearance(context: *mut c_void, callback: GetAppearanceCallback);
		}
	}

	fn mpalsts_cloudkit_save_appearance(
		theme_mode: ThemeMode,
	) -> std::result::Result<(), CloudKitSyncProviderError> {
		let (tx, rx) = mpsc::channel::<i8>();

		let context = Box::into_raw(Box::new(tx)) as *mut c_void;

		extern "C" fn save_callback(context: *mut c_void, code: c_char) {
			if context.is_null() {
				error!(name: "save_callback", "context is null");
				return;
			}

			unsafe {
				let sender: Box<mpsc::Sender<i8>> = Box::from_raw(context as *mut mpsc::Sender<i8>);

				error!(name: "save_callback", "sending code: {code}");
				let _ = sender.send(code);
			}
		}

		unsafe {
			ffi::mpalsts_cloudkit_save_appearance(theme_mode as u8, context, save_callback);
		}

		return save_result_from_code(
			rx.recv()
				.map_err(CloudKitSyncProviderError::FFIBoundaryReceiveError)?,
		);
	}

	fn save_result_from_code(code: i8) -> std::result::Result<(), CloudKitSyncProviderError> {
		return match code {
			0 => Ok(()),

			ffi::INVALID_THEME_MODE => Err(CloudKitSyncProviderError::InvalidThemeMode),
			ffi::COULD_NOT_UPDATE_APPEARANCE => Err(CloudKitSyncProviderError::CouldNotUpdateAppearance),

			x => Err(CloudKitSyncProviderError::UnknownError(x)),
		};
	}

	pub fn mpalsts_cloudkit_get_appearance()
	-> std::result::Result<ThemeMode, CloudKitSyncProviderError> {
		let (tx, rx) = mpsc::channel::<u8>();

		let context = Box::into_raw(Box::new(tx)) as *mut c_void;

		extern "C" fn save_callback(context: *mut c_void, code: c_uchar) {
			if context.is_null() {
				error!(name: "save_callback", "context is null");
				return;
			}

			unsafe {
				let sender: Box<mpsc::Sender<u8>> = Box::from_raw(context as *mut mpsc::Sender<u8>);
				let _ = sender.send(code);
			}
		}

		unsafe {
			ffi::mpalsts_cloudkit_get_appearance(context, save_callback);
		}

		return appearance_result_from_code(
			rx.recv()
				.map_err(CloudKitSyncProviderError::FFIBoundaryReceiveError)?,
		);
	}

	fn appearance_result_from_code(
		code: u8,
	) -> std::result::Result<ThemeMode, CloudKitSyncProviderError> {
		return ThemeMode::try_from(code).map_err(|_| CloudKitSyncProviderError::InvalidThemeMode);
	}

	pub struct CloudKitSyncProvider;

	#[derive(Debug, Error)]
	pub enum CloudKitSyncProviderError {
		#[error("could not update appearance")]
		CouldNotUpdateAppearance,

		#[error("invalid theme mode")]
		InvalidThemeMode,

		#[error("ffi boundary receive error: {0}")]
		FFIBoundaryReceiveError(#[from] RecvError),

		#[error("unknown error: got error code {0}")]
		UnknownError(i8),
	}

	impl SyncProvider for CloudKitSyncProvider {
		async fn update_appearance(&self, theme: ThemeMode) -> Result<()> {
			return mpalsts_cloudkit_save_appearance(theme).map_err(SyncError::CloudKitSyncProviderError);
		}

		async fn get_appearance(&self) -> Result<ThemeMode> {
			return mpalsts_cloudkit_get_appearance().map_err(SyncError::CloudKitSyncProviderError);
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		#[test]
		fn save_result_from_code_maps_success() {
			assert!(save_result_from_code(0).is_ok());
		}

		#[test]
		fn save_result_from_code_maps_known_errors() {
			assert!(matches!(
				save_result_from_code(ffi::COULD_NOT_UPDATE_APPEARANCE),
				Err(CloudKitSyncProviderError::CouldNotUpdateAppearance)
			));
			assert!(matches!(
				save_result_from_code(ffi::INVALID_THEME_MODE),
				Err(CloudKitSyncProviderError::InvalidThemeMode)
			));
		}

		#[test]
		fn save_result_from_code_preserves_unknown_error_code() {
			assert!(matches!(
				save_result_from_code(-42),
				Err(CloudKitSyncProviderError::UnknownError(-42))
			));
		}

		#[test]
		fn appearance_result_from_code_maps_theme_modes() {
			assert!(matches!(
				appearance_result_from_code(0),
				Ok(ThemeMode::Light)
			));
			assert!(matches!(
				appearance_result_from_code(1),
				Ok(ThemeMode::Dark)
			));
		}

		#[test]
		fn appearance_result_from_code_rejects_invalid_theme_mode() {
			assert!(matches!(
				appearance_result_from_code(2),
				Err(CloudKitSyncProviderError::InvalidThemeMode)
			));
		}

		#[test]
		fn live_cloudkit_pushes_dark_appearance_and_reads_it_back() {
			if std::env::var("MPALSTS_CLOUDKIT_LIVE_TESTS").as_deref() != Ok("1") {
				return;
			}

			let provider = CloudKitSyncProvider;

			futures::executor::block_on(async {
				provider.update_appearance(ThemeMode::Dark).await.unwrap();

				let appearance = provider.get_appearance().await.unwrap();
				assert_eq!(appearance, ThemeMode::Dark);
			});
		}
	}
}

#[cfg(not(cloudkit))]
mod _cloudkit {}

#[allow(unused_imports)]
pub use _cloudkit::*;
