use std::{io, process};

use fork::{Fork, fork};
use thiserror::Error;
use tracing::{Level, info, span, trace};

#[derive(Default)]
pub enum RuntimeMode {
	#[default]
	Interactive,
	Daemon,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
	#[error("could not fork process: {0}")]
	ForkError(#[from] io::Error),
}

pub fn runtime(mode: Option<RuntimeMode>) -> Result<(), RuntimeError> {
	let mode = mode.unwrap_or_default();

	let RuntimeMode::Interactive = mode else {
		return match fork() {
			Ok(Fork::Parent(child_pid)) => {
				info!("Daemon process has been started. Child PID: {child_pid}");

				Ok(())
			}
			Ok(Fork::Child) => {
				let span = span!(Level::INFO, "child", pid = process::id());
				let _enter = span.enter();

				span.in_scope(|| runtime(Some(RuntimeMode::Interactive)))
			}
			Err(err) => Err(RuntimeError::ForkError(err)),
		};
	};

	// Should be unreachable by the parent process if `mode` is `Daemon`.

	trace!("Interactive mode");

	return Ok(());
}
