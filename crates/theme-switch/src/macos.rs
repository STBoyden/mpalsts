use osakit::{Language, Script};

use crate::ThemeSwitcher;

#[derive(Debug)]
pub struct MacosThemeSwitcher {
	dark_mode_script:  Script,
	light_mode_script: Script,
}

impl MacosThemeSwitcher {
	pub(crate) fn new() -> Self {
		let mut dark_mode_script = Script::new_from_source(
			Language::AppleScript,
			r#"tell application "System Events" to tell appearance preferences to set dark mode to true"#,
		);
		let mut light_mode_script = Script::new_from_source(
			Language::AppleScript,
			r#"tell application "System Events" to tell appearance preferences to set dark mode to false"#,
		);

		dark_mode_script
			.compile()
			.expect("could not compile dark mode script");
		light_mode_script
			.compile()
			.expect("could not compile light mode script");

		return Self {
			dark_mode_script,
			light_mode_script,
		};
	}
}

impl Default for MacosThemeSwitcher {
	fn default() -> Self {
		return Self::new();
	}
}

impl ThemeSwitcher for MacosThemeSwitcher {
	fn to_light(&self) {
		_ = self.light_mode_script.execute();
	}

	fn to_dark(&self) {
		_ = self.dark_mode_script.execute();
	}
}
