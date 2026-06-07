use std::{
	io, process, thread,
	time::{Duration, Instant},
};

use fork::{Fork, fork};
use futures::executor::block_on;
use mpalsts_sensors::LightSensor;
use mpalsts_shared::{AppState, ThemeMode};
use mpalsts_theme_switch::ThemeSwitcher;
use thiserror::Error;
use tracing::{Level, info, span, trace, warn};

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

	#[error("no ambient light sensor found")]
	NoSensor,
}

pub fn runtime(mode: Option<RuntimeMode>, state: AppState) -> Result<(), RuntimeError> {
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

				span.in_scope(|| runtime(Some(RuntimeMode::Interactive), state))
			}
			Err(err) => Err(RuntimeError::ForkError(err)),
		};
	};

	// Should be unreachable by the parent process if `mode` is `Daemon`.

	trace!("Interactive mode");

	let sensor = mpalsts_sensors::get_platform_reader();

	#[cfg(target_os = "linux")]
	{
		use std::path::PathBuf;

		sensor.mutate_concrete(|mut concrete| {
			concrete.set_device_file(state.linux.sensor.as_ref().map(PathBuf::from));
		});
	}

	if !sensor.has_sensor() {
		return Err(RuntimeError::NoSensor);
	}

	let mut theme_switcher = mpalsts_theme_switch::get();

	#[cfg(target_os = "linux")]
	{
		theme_switcher.set_light_theme(state.linux.light_theme.clone());
		theme_switcher.set_dark_theme(state.linux.dark_theme.clone());
	}

	let lumens_threshold = f64::from(state.lumens_threshold);
	let seconds_threshold = Duration::from_secs_f32(state.seconds_threshold);
	let poll_rate = Duration::from_secs(1);
	let mut current_theme_mode: Option<ThemeMode> = None;
	let mut pending_theme_mode: Option<(ThemeMode, Instant)> = None;

	info!("Starting runtime");
	info!(
		lumens_threshold = lumens_threshold,
		seconds_threshold = format!("{seconds_threshold:?}"),
	);

	loop {
		let lumens = match block_on(sensor.read()) {
			Ok(lumens) => lumens,
			Err(error) => {
				warn!("failed to read ambient light sensor: {error}");
				thread::sleep(poll_rate);
				continue;
			}
		};

		let desired_theme_mode = if lumens > lumens_threshold {
			ThemeMode::Light
		} else {
			ThemeMode::Dark
		};

		trace!(
			lumens = lumens,
			lumens_threshold = lumens_threshold,
			seconds_threshold = format!("{seconds_threshold:?}"),
			desired_theme_mode = format!("{desired_theme_mode:?}"),
		);

		if current_theme_mode.is_none() {
			switch_theme(&theme_switcher, desired_theme_mode);
			current_theme_mode = Some(desired_theme_mode);
			pending_theme_mode = None;
			thread::sleep(poll_rate);
			continue;
		}

		if current_theme_mode == Some(desired_theme_mode) {
			pending_theme_mode = None;
			thread::sleep(poll_rate);
			continue;
		}

		match pending_theme_mode {
			Some((pending_mode, pending_since)) if pending_mode == desired_theme_mode => {
				if pending_since.elapsed() >= seconds_threshold {
					switch_theme(&theme_switcher, desired_theme_mode);
					current_theme_mode = Some(desired_theme_mode);
					pending_theme_mode = None;
				}
			}
			_ => {
				pending_theme_mode = Some((desired_theme_mode, Instant::now()));
			}
		}

		thread::sleep(poll_rate);
	}
}

fn switch_theme(theme_switcher: &impl ThemeSwitcher, mode: ThemeMode) {
	info!("Switching to {mode:?} theme");

	match mode {
		ThemeMode::Light => theme_switcher.to_light(),
		ThemeMode::Dark => theme_switcher.to_dark(),
	}
}
