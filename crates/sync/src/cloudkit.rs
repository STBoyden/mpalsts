#[cfg(cloudkit)]
mod _cloudkit {
	use mpalsts_shared::ThemeMode;
	use thiserror::Error;

	use crate::{Result, SyncProvider};

	pub struct CloudKitSyncProvider;

	#[derive(Debug, Error)]
	pub enum CloudKitSyncProviderError {}

	impl SyncProvider for CloudKitSyncProvider {
		async fn update_appearance(&self, theme: ThemeMode) -> Result<()> {
			return Ok(());
		}

		async fn get_appearance(&self) -> Result<ThemeMode> {
			return Ok(ThemeMode::Dark);
		}
	}
}

#[cfg(all(feature = "cloudkit", not(macos)))]
mod _cloudkit {
	compile_error!("CloudKit is only availble on macOS");
}

#[cfg(dummy)]
mod _cloudkit {}

#[allow(unused_imports)]
pub use _cloudkit::*;
