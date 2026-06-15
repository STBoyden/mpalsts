use std::{
	borrow::Cow,
	error::Error,
	fs::{self, File, OpenOptions, TryLockError},
	io::{self, BufWriter, Read, Write},
	path::{Path, PathBuf},
	process,
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
	thread,
	time::{Duration, Instant},
};

use fork::{Fork, fork};
use futures::executor::block_on;
use mpalsts_sensors::LightSensor;
use mpalsts_shared::{AppState, PROJECT_DIR, ThemeMode};
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
pub enum ForkError {
	#[error("error from fork() syscall: {0}")]
	SyscallError(#[from] io::Error),

	#[error("daemon already running with pid {pid}")]
	ExistingDaemon { pid: i64 },
}

#[derive(Debug, Error)]
pub enum PIDFileError {
	#[error("could not find path for PID file: {reason}")]
	PathError { reason: Cow<'static, str> },

	#[error("pid format is corrupted, should be integer, instead is {corrupted_pid}: {error}")]
	CorruptedPID {
		corrupted_pid: String,
		error: Box<dyn Error>,
	},

	#[error("i/o error: {0}")]
	IOError(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum RuntimeError {
	#[error("could not fork process: {0}")]
	ForkError(ForkError),

	#[error("pid file error: {0}")]
	PIDFileError(PIDFileError),

	#[error("no ambient light sensor found")]
	NoSensor,
}

#[derive(Debug, Error)]
enum AutoDeletingFileError {
	#[error("path is not a file")]
	NotAFile { path: Box<Path> },

	#[error("i/o error: {0}")]
	IOError(#[from] io::Error),

	#[error("could not get lock to file: {0}")]
	FailedToObtainLock(TryLockError),
}

struct AutoDeletingFile {
	file: Option<File>,
	file_path: Arc<PathBuf>,
	is_in_drop: Arc<AtomicBool>,
}

impl AutoDeletingFile {
	fn new<P: Into<PathBuf>>(
		file_path: P,
		wait_for_lock: bool,
	) -> Result<Self, AutoDeletingFileError> {
		let file_path = file_path.into();
		if !file_path.is_file() {
			return Err(AutoDeletingFileError::NotAFile {
				path: file_path.into_boxed_path(),
			});
		}

		let file = OpenOptions::new()
			.create(true)
			.append(true)
			.read(true)
			.open(&file_path)
			.map_err(AutoDeletingFileError::IOError)?;

		if wait_for_lock {
			file.lock().map_err(AutoDeletingFileError::IOError)?;
		} else {
			file
				.try_lock()
				.map_err(AutoDeletingFileError::FailedToObtainLock)?;
		}

		let file_path = Arc::new(file_path);
		let is_in_drop = Arc::new(AtomicBool::new(false));

		{
			let file_path = file_path.clone();
			let is_in_drop = is_in_drop.clone();

			thread::spawn(move || {
				let file_path = file_path.clone();
				let file_path = file_path.as_path();

				let contents = fs::read_to_string(file_path).expect("could not get contents of PID file");

				while !is_in_drop.load(Ordering::Acquire) {
					if !file_path.exists() {
						warn!("PIDfile got deleted somehow! recreating...");

						_ = File::create_new(file_path)
							.and_then(|mut file| file.write_fmt(format_args!("{contents}")))
							.inspect_err(|err| warn!("got error while trying to recreate pid file: {err}"));
					}

					thread::sleep(Duration::from_millis(10));
				}
			});
		}

		return Ok(Self {
			file_path: file_path.clone(),
			file: Some(file),
			is_in_drop,
		});
	}
}

impl Drop for AutoDeletingFile {
	fn drop(&mut self) {
		let AutoDeletingFile {
			file: Some(file),
			file_path,
			is_in_drop,
		} = self
		else {
			return;
		};

		is_in_drop.store(true, Ordering::Release);

		_ = fs::remove_file(file_path.as_path()).inspect_err(|err| {
			tracing::error!(
				"could not delete file at path {}: {err}",
				file_path.display()
			)
		});

		_ = file.unlock();
	}
}

fn daemon_mode(state: AppState) -> Result<Option<AutoDeletingFile>, RuntimeError> {
	let config_dir = PROJECT_DIR
		.as_ref()
		.ok_or(RuntimeError::PIDFileError(PIDFileError::PathError {
			reason: "could not determine project directory".into(),
		}))?
		.config_dir();

	let pid_file_path = config_dir.join("daemon.pid");

	if pid_file_path.exists()
		&& let Ok(mut file) = File::open(&pid_file_path)
	{
		let mut buffer = String::new();
		file
			.read_to_string(&mut buffer)
			.map_err(|err| RuntimeError::PIDFileError(PIDFileError::IOError(err)))?;

		let pid = buffer.parse::<i64>().map_err(|err| {
			RuntimeError::PIDFileError(PIDFileError::CorruptedPID {
				corrupted_pid: buffer,
				error: err.into(),
			})
		})?;

		return Err(RuntimeError::ForkError(ForkError::ExistingDaemon { pid }));
	}

	if !config_dir.exists() {
		fs::create_dir_all(config_dir)
			.map_err(|err| RuntimeError::PIDFileError(PIDFileError::IOError(err)))?;
	}

	let pid_file = File::create(&pid_file_path)
		.map_err(|err| RuntimeError::PIDFileError(PIDFileError::IOError(err)))?;

	trace!("Created new PIDfile at {}", pid_file_path.display());

	return match fork() {
		Ok(Fork::Parent(child_pid)) => {
			info!("Daemon process has been started. Child PID: {child_pid}");
			Ok(None)
		}

		Ok(Fork::Child) => {
			let child_pid = process::id();

			let span = span!(
				Level::INFO,
				"child",
				pid = process::id(),
				pid_file = pid_file_path.display().to_string()
			);
			let _enter = span.enter();

			pid_file
				.lock()
				.map_err(|err| RuntimeError::PIDFileError(PIDFileError::IOError(err)))?;

			let mut writer = BufWriter::new(&pid_file);

			writer
				.write_fmt(format_args!("{child_pid}"))
				.map_err(|err| RuntimeError::PIDFileError(PIDFileError::IOError(err)))?;

			trace!("Wrote PID to pidfile: {child_pid}");

			drop(writer);

			_ = pid_file.unlock();

			span.in_scope(|| {
				let pid_file =
					AutoDeletingFile::new(&pid_file_path, true).expect("could not obtain pid file!");

				runtime(Some(RuntimeMode::Interactive), state).expect("runtime failed!");

				return Ok(Some(pid_file));
			})
		}

		Err(err) => Err(RuntimeError::ForkError(ForkError::SyscallError(err))),
	};
}

#[allow(unused_mut)]
pub fn runtime(mode: Option<RuntimeMode>, state: AppState) -> Result<(), RuntimeError> {
	let mode = mode.unwrap_or_default();

	let RuntimeMode::Interactive = mode else {
		return daemon_mode(state).map(|_| ());
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
