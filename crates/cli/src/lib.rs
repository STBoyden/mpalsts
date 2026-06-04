use clap::{Arg, ArgAction, Command};
use log::trace;
#[cfg(feature = "include_gui")]
use mpalsts_gui::run as gui_run;

fn daemon_run() {
	trace!("Running in daemon mode");
}

pub fn main() {
	let daemon_arg = Arg::new("daemon")
		.short('d')
		.long("daemon")
		.action(ArgAction::SetTrue)
		.default_value(if cfg!(feature = "include_gui") {
			"false"
		} else {
			"true"
		})
		.required(!cfg!(feature = "include_gui"))
		.help("Run as a daemon")
		.long_help(
			"Run as a daemon - required flag if the binary is compiled without the `include_gui` feature",
		);

	let cmd = Command::new("mpalsts").arg(daemon_arg);

	let matches = cmd.get_matches();

	let run_as_daemon = matches.get_flag("daemon");

	#[cfg(feature = "include_gui")]
	{
		if run_as_daemon {
			daemon_run();
		} else {
			gui_run();
		}
	}
	#[cfg(not(feature = "include_gui"))]
	{
		use std::process::exit;

		if run_as_daemon {
			daemon_run();
		} else {
			exit(1);
		}
	}
}
