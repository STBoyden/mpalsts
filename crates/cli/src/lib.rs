use clap::{Arg, ArgAction, Command};
use dotenv::dotenv;
#[cfg(feature = "include_gui")]
use mpalsts_gui::run as _gui_run;
use thiserror::Error;
use tracing::trace;

use crate::runtime::{RuntimeMode, runtime};

mod runtime;

fn daemon_run() -> Result<(), CliError> {
	trace!("Running in daemon mode");

	runtime(Some(RuntimeMode::Daemon))?;
	return Ok(());
}

#[cfg(feature = "include_gui")]
fn gui_run() -> Result<(), CliError> {
	trace!("Running in GUI mode");

	_gui_run();
	return Ok(());
}

#[derive(Debug, Error)]
pub enum CliError {
	#[error("runtime error: {0}")]
	RuntimeError(#[from] runtime::RuntimeError),

	#[error("unexpected error: {0}")]
	UnexpectedError(#[from] anyhow::Error),
}

pub fn main() -> Result<(), CliError> {
	dotenv().ok();
	tracing_subscriber::fmt::try_init().map_err(|err| anyhow::anyhow!(err))?;

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
		.help(if cfg!(feature = "include_gui") {
			"Run as a daemon"
		} else {
			"Run as a daemon (required)"
		})
		.long_help(
			"Run as a daemon - required flag if the binary is compiled without the `include_gui` feature",
		);

	let cmd = Command::new("mpalsts").arg(daemon_arg);

	let matches = cmd.get_matches();

	let run_as_daemon = matches.get_flag("daemon");

	#[cfg(feature = "include_gui")]
	{
		if run_as_daemon {
			return daemon_run();
		} else {
			return gui_run();
		}
	}
	#[cfg(not(feature = "include_gui"))]
	{
		use std::process::exit;

		if run_as_daemon {
			return daemon_run();
		} else {
			use anyhow::anyhow;

			return Err(anyhow!("Somehow got to an unreachable code path"));
		}
	}
}
