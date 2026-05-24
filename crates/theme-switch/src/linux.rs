use std::{collections::BTreeSet, env, fs, path::PathBuf, process::Command};

use crate::ThemeSwitcher;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DesktopEnvironment {
	Gnome,
	Kde,
}

#[derive(Debug, Clone)]
pub struct LinuxThemeSwitcher {
	light_theme: Option<String>,
	dark_theme:  Option<String>,
}

impl LinuxThemeSwitcher {
	pub(crate) fn new() -> Self {
		let mut switcher = Self {
			light_theme: None,
			dark_theme:  None,
		};

		switcher.light_theme = switcher.get_current_light_theme();
		switcher.dark_theme = switcher.get_current_dark_theme();

		return switcher;
	}

	pub fn set_light_theme(&mut self, theme: Option<String>) {
		if theme.is_some() {
			self.light_theme = theme;
		}
	}

	pub fn set_dark_theme(&mut self, theme: Option<String>) {
		if theme.is_some() {
			self.dark_theme = theme;
		}
	}

	pub fn get_themes(&self) -> Vec<String> {
		return self.collect_directory_names(self.theme_directories());
	}

	pub fn get_kde_themes(&self) -> Vec<String> {
		let mut themes = BTreeSet::new();

		for theme_dir in self.kde_theme_directories() {
			Self::collect_directory_names_from(&theme_dir, &mut themes);
		}

		for color_scheme_dir in self.kde_color_scheme_directories() {
			Self::collect_file_stems_from(&color_scheme_dir, "colors", &mut themes);
		}

		return themes.into_iter().collect();
	}

	pub fn get_current_light_theme(&self) -> Option<String> {
		let current_theme = self.current_theme_name()?;
		let installed = self.installed_themes();

		return self
			.find_light_variant(&current_theme, &installed)
			.or(Some(current_theme));
	}

	pub fn get_current_dark_theme(&self) -> Option<String> {
		let current_theme = self.current_theme_name()?;
		let installed = self.installed_themes();

		return self
			.find_dark_variant(&current_theme, &installed)
			.or(Some(current_theme));
	}

	fn current_theme_name(&self) -> Option<String> {
		match Self::desktop_environment() {
			Some(DesktopEnvironment::Gnome) => {
				return self
					.gnome_current_theme()
					.or_else(|| self.kde_current_theme());
			}
			Some(DesktopEnvironment::Kde) => {
				return self
					.kde_current_theme()
					.or_else(|| self.gnome_current_theme());
			}
			None => {}
		}

		return self
			.gnome_current_theme()
			.or_else(|| self.kde_current_theme());
	}

	fn desktop_environment() -> Option<DesktopEnvironment> {
		let values = [
			env::var("XDG_CURRENT_DESKTOP").ok(),
			env::var("XDG_SESSION_DESKTOP").ok(),
			env::var("DESKTOP_SESSION").ok(),
		];

		for value in values.into_iter().flatten() {
			let value = value.to_lowercase();

			if value.contains("gnome") {
				return Some(DesktopEnvironment::Gnome);
			}

			if value.contains("kde") || value.contains("plasma") {
				return Some(DesktopEnvironment::Kde);
			}
		}

		return None;
	}

	fn gnome_current_theme(&self) -> Option<String> {
		return self
			.run_command([
				"gsettings",
				"get",
				"org.gnome.desktop.interface",
				"gtk-theme",
			])
			.map(|theme| theme.trim().trim_matches('\'').to_string())
			.filter(|theme| !theme.is_empty());
	}

	fn kde_current_theme(&self) -> Option<String> {
		return self
			.run_command([
				"kreadconfig6",
				"--file",
				"kdeglobals",
				"--group",
				"General",
				"--key",
				"ColorScheme",
			])
			.or_else(|| {
				return self.run_command([
					"kreadconfig5",
					"--file",
					"kdeglobals",
					"--group",
					"General",
					"--key",
					"ColorScheme",
				]);
			})
			.map(|theme| theme.trim().to_string())
			.filter(|theme| !theme.is_empty());
	}

	fn run_command<const N: usize>(&self, args: [&str; N]) -> Option<String> {
		let (program, command_args) = args.split_first()?;
		let output = Command::new(program).args(command_args).output().ok()?;
		if !output.status.success() {
			return None;
		}
		let stdout = String::from_utf8(output.stdout).ok()?;
		return Some(stdout);
	}

	fn run_command_status<const N: usize>(&self, args: [&str; N]) -> bool {
		let Some((program, command_args)) = args.split_first() else {
			return false;
		};

		return Command::new(program)
			.args(command_args)
			.status()
			.map(|status| status.success())
			.unwrap_or(false);
	}

	fn switch_to_theme(&self, theme: Option<&String>, dark: bool) {
		let Some(theme) = theme else {
			return;
		};

		match Self::desktop_environment() {
			Some(DesktopEnvironment::Gnome) => {
				if self.gnome_current_theme().as_deref() != Some(theme.as_str()) {
					self.switch_gnome_theme(theme, dark);
				}
			}
			Some(DesktopEnvironment::Kde) => {
				if self.kde_current_theme().as_deref() != Some(theme.as_str()) {
					self.switch_kde_theme(theme);
				}
			}
			None => {
				if self.gnome_current_theme().as_deref() != Some(theme.as_str()) {
					self.switch_gnome_theme(theme, dark);
				}

				if self.kde_current_theme().as_deref() != Some(theme.as_str()) {
					self.switch_kde_theme(theme);
				}
			}
		}
	}

	fn switch_gnome_theme(&self, theme: &str, dark: bool) {
		let color_scheme = if dark { "prefer-dark" } else { "prefer-light" };

		_ = self.run_command_status([
			"gsettings",
			"set",
			"org.gnome.desktop.interface",
			"gtk-theme",
			theme,
		]);
		_ = self.run_command_status([
			"gsettings",
			"set",
			"org.gnome.desktop.interface",
			"color-scheme",
			color_scheme,
		]);
	}

	fn switch_kde_theme(&self, theme: &str) {
		let theme = match theme {
			"Breeze-Dark" => "BreezeDark",
			"Breeze" => "BreezeLight",
			_ => theme,
		};

		let theme = theme.replace("-", "");
		if self.run_command_status(["plasma-apply-colorscheme", &theme]) {
			return;
		}

		if self.run_command_status(["lookandfeeltool", "-a", &theme]) {
			return;
		}

		_ = self.run_command_status([
			"kwriteconfig6",
			"--file",
			"kdeglobals",
			"--group",
			"General",
			"--key",
			"ColorScheme",
			&theme,
		]) || self.run_command_status([
			"kwriteconfig5",
			"--file",
			"kdeglobals",
			"--group",
			"General",
			"--key",
			"ColorScheme",
			&theme,
		]);
	}

	fn installed_themes(&self) -> BTreeSet<String> {
		let mut themes = BTreeSet::new();

		for theme in self.get_themes() {
			themes.insert(theme);
		}

		for theme in self.get_kde_themes() {
			themes.insert(theme);
		}

		return themes;
	}

	fn find_light_variant(&self, theme: &str, installed: &BTreeSet<String>) -> Option<String> {
		return self.find_variant(theme, &["-dark", "_dark", " dark", "Dark"], installed);
	}

	fn find_dark_variant(&self, theme: &str, installed: &BTreeSet<String>) -> Option<String> {
		return self
			.find_variant(theme, &["-light", "_light", " light", "Light"], installed)
			.or_else(|| self.find_variant(theme, &["", " "], installed));
	}

	fn find_variant(
		&self,
		theme: &str,
		suffixes: &[&str],
		installed: &BTreeSet<String>,
	) -> Option<String> {
		for suffix in suffixes {
			if let Some(candidate) = theme.strip_suffix(suffix)
				&& installed.contains(candidate)
			{
				return Some(candidate.to_string());
			}
		}

		for suffix in suffixes {
			let candidate = format!("{theme}{suffix}");
			if installed.contains(&candidate) {
				return Some(candidate);
			}
		}

		return None;
	}

	fn collect_directory_names(&self, directories: Vec<PathBuf>) -> Vec<String> {
		let mut themes = BTreeSet::new();

		for theme_dir in directories {
			Self::collect_directory_names_from(&theme_dir, &mut themes);
		}

		return themes.into_iter().collect();
	}

	fn collect_directory_names_from(directory: &PathBuf, themes: &mut BTreeSet<String>) {
		let Ok(entries) = fs::read_dir(directory) else {
			return;
		};

		for entry in entries.flatten() {
			let Ok(file_type) = entry.file_type() else {
				continue;
			};

			if !file_type.is_dir() {
				continue;
			}

			if let Some(theme_name) = entry.file_name().to_str() {
				themes.insert(theme_name.to_string());
			}
		}
	}

	fn collect_file_stems_from(directory: &PathBuf, extension: &str, themes: &mut BTreeSet<String>) {
		let Ok(entries) = fs::read_dir(directory) else {
			return;
		};

		for entry in entries.flatten() {
			let Ok(file_type) = entry.file_type() else {
				continue;
			};

			if !file_type.is_file() {
				continue;
			}

			let path = entry.path();
			let Some(file_extension) = path.extension().and_then(|value| value.to_str()) else {
				continue;
			};

			if file_extension != extension {
				continue;
			}

			if let Some(theme_name) = path.file_stem().and_then(|value| value.to_str()) {
				themes.insert(theme_name.to_string());
			}
		}
	}

	fn theme_directories(&self) -> Vec<PathBuf> {
		let mut directories = vec![
			PathBuf::from("/usr/share/themes"),
			PathBuf::from("/usr/local/share/themes"),
		];

		if let Some(home) = env::var_os("HOME") {
			let home = PathBuf::from(home);
			directories.push(home.join(".themes"));
			directories.push(home.join(".local/share/themes"));
		}

		return directories;
	}

	fn kde_theme_directories(&self) -> Vec<PathBuf> {
		let mut directories = vec![
			PathBuf::from("/usr/share/plasma/desktoptheme"),
			PathBuf::from("/usr/local/share/plasma/desktoptheme"),
		];

		if let Some(home) = env::var_os("HOME") {
			let home = PathBuf::from(home);
			directories.push(home.join(".local/share/plasma/desktoptheme"));
		}

		return directories;
	}

	fn kde_color_scheme_directories(&self) -> Vec<PathBuf> {
		let mut directories = vec![
			PathBuf::from("/usr/share/color-schemes"),
			PathBuf::from("/usr/local/share/color-schemes"),
		];

		if let Some(home) = env::var_os("HOME") {
			let home = PathBuf::from(home);
			directories.push(home.join(".local/share/color-schemes"));
		}

		return directories;
	}
}

impl ThemeSwitcher for LinuxThemeSwitcher {
	fn to_light(&self) {
		self.switch_to_theme(self.light_theme.as_ref(), false);
	}

	fn to_dark(&self) {
		self.switch_to_theme(self.dark_theme.as_ref(), true);
	}
}

impl Default for LinuxThemeSwitcher {
	fn default() -> Self {
		return Self::new();
	}
}
