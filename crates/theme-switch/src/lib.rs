#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
pub use linux::LinuxThemeSwitcher as PlatformThemeSwitcher;
#[cfg(target_os = "macos")]
pub use macos::MacosThemeSwitcher as PlatformThemeSwitcher;

pub trait ThemeSwitcher {
	fn to_light(&self);
	fn to_dark(&self);
}

pub fn get() -> PlatformThemeSwitcher {
	return PlatformThemeSwitcher::new();
}
