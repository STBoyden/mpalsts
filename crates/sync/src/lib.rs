use std::time::{SystemTime, UNIX_EPOCH};

use mpalsts_shared::ThemeMode;
use serde::{Deserialize, Serialize};

mod cloudkit;

#[derive(Serialize, Deserialize)]
pub(crate) struct SyncObject {
	theme: ThemeMode,
	updated_time_ms: u128,
}

impl SyncObject {
	pub(crate) fn new(theme: ThemeMode) -> Self {
		return Self {
			theme,
			updated_time_ms: SystemTime::now()
				.duration_since(UNIX_EPOCH)
				.expect("typically time moves forward")
				.as_millis(),
		};
	}
}

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
	#[cfg(cloudkit)]
	#[error("cloudkit error: {0}")]
	CloudKitSyncProviderError(#[from] cloudkit::CloudKitSyncProviderError),
}

pub(crate) type Result<T> = std::result::Result<T, SyncError>;

pub trait SyncProvider {
	fn update_appearance(&self, theme: ThemeMode) -> impl Future<Output = Result<()>>;
	fn get_appearance(&self) -> impl Future<Output = Result<ThemeMode>>;
}

#[cfg(cloudkit)]
pub fn cloudkit() -> cloudkit::CloudKitSyncProvider {
	return cloudkit::CloudKitSyncProvider;
}
