#[cfg(feature = "include_gui")]
use mpalsts_gui::run as gui_run;

pub fn main() {
	#[cfg(feature = "include_gui")]
	gui_run();
}
