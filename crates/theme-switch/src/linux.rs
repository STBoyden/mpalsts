use std::{collections::BTreeSet, env, fs, path::PathBuf, process::Command};

use crate::ThemeSwitcher;

#[derive(Debug, Clone)]
pub struct LinuxThemeSwitcher {
	light_theme: Option<String>,
	dark_theme:  Option<String>,
}

impl LinuxThemeSwitcher {
	pub(crate) fn new() -> Self {
		return Self {
			light_theme: None,
			dark_theme:  None,
		};
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
			.or_else(|| Some(current_theme));
	}

	pub fn get_current_dark_theme(&self) -> Option<String> {
		let current_theme = self.current_theme_name()?;
		let installed = self.installed_themes();

		return self
			.find_dark_variant(&current_theme, &installed)
			.or_else(|| Some(current_theme));
	}

	fn current_theme_name(&self) -> Option<String> {
		if let Some(theme) = self.gnome_current_theme() {
			return Some(theme);
		}

		if let Some(theme) = self.kde_current_theme() {
			return Some(theme);
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
		let stdout = String::from_utf8(output.stdout).ok()?;
		return Some(stdout);
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
	fn to_light(&self) {}

	fn to_dark(&self) {}
}

impl Default for LinuxThemeSwitcher {
	fn default() -> Self {
		return Self::new();
	}
}
