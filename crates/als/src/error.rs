use thiserror::Error;

#[cfg(target_os = "macos")]
use crate::macos::MacOSALSError;

#[derive(Error, Debug)]
pub enum ALSError {
	#[cfg(target_os = "macos")]
	#[error("macos error: {0}")]
	Platform(#[from] MacOSALSError),
}
