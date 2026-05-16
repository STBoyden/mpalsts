use thiserror::Error;

#[cfg(target_os = "linux")]
use crate::linux::LinuxALSError;
#[cfg(target_os = "macos")]
use crate::macos::MacOSALSError;

#[derive(Error, Debug)]
pub enum ALSError {
	#[cfg(target_os = "macos")]
	#[error("macos error: {0}")]
	Platform(#[from] MacOSALSError),

	#[cfg(target_os = "linux")]
	#[error("linux error: {0}")]
	Platform(#[from] LinuxALSError),
}
