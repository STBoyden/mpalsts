use std::{
	fs::{self, File},
	io::BufWriter,
};

use anyhow::anyhow;
use directories::ProjectDirs;
use gpui::{prelude::*, *};
use gpui_component::{
	ActiveTheme, Root, Theme, TitleBar,
	button::{Button, ButtonVariants},
	checkbox::Checkbox,
	h_flex,
	separator::Separator,
	slider::{Slider, SliderEvent, SliderState},
	v_flex,
};
use log::{error, trace};
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};

const MIN_LUMENS_THRESHOLD: f32 = 10.;
const DEFAULT_LUMENS_THRESHOLD: f32 = 100.;
const MAX_LUMENS_THRESHOLD: f32 = 2000.;

const MIN_SECONDS_THRESHOLD: f32 = 10.;
const DEFAULT_SECONDS_THRESHOLD: f32 = 30.;
const MAX_SECONDS_THRESHOLD: f32 = 120.;

#[derive(Debug, Serialize, Deserialize)]
pub struct AppState {
	enable_theme_switching: bool,
	enable_autostart:       bool,
	lumens_threshold:       f32,
	seconds_threshold:      f32,
}

impl Default for AppState {
	fn default() -> Self {
		return Self {
			enable_theme_switching: false,
			enable_autostart:       false,
			lumens_threshold:       DEFAULT_LUMENS_THRESHOLD,
			seconds_threshold:      DEFAULT_SECONDS_THRESHOLD,
		};
	}
}

impl AppState {
	pub(crate) fn read_config() -> anyhow::Result<Self> {
		let project_dir = ProjectDirs::from("com", "stboyden", "mpalsts")
			.ok_or(anyhow!("failed to resolve project directory"))?;

		let config_dir = project_dir.config_dir();

		fs::create_dir_all(config_dir)?;

		let config_ron = config_dir.join("config.ron");

		let exists = fs::exists(&config_ron);
		if exists.is_err() || !exists? {
			return Ok(Self::default());
		}

		let config_ron = File::open(&config_ron)?;
		let config: AppState = ron::de::from_reader(config_ron)?;

		return Ok(config);
	}

	pub(crate) fn save_config(&self) -> anyhow::Result<()> {
		let project_dir = ProjectDirs::from("com", "stboyden", "mpalsts")
			.ok_or(anyhow!("failed to resolve project directory"))?;

		let config_dir = project_dir.config_dir();
		fs::create_dir_all(config_dir)?;

		let config_ron = config_dir.join("config.ron");
		let config_ron = BufWriter::new(File::create(&config_ron)?);
		ron::Options::default().to_io_writer_pretty(config_ron, self, PrettyConfig::new())?;

		return Ok(());
	}
}

#[derive(Debug)]
struct App {
	state:                Entity<AppState>,
	lumens_slider_state:  Entity<SliderState>,
	seconds_slider_state: Entity<SliderState>,
}

impl App {
	fn new(state: Entity<AppState>, cx: &mut Context<Self>) -> Self {
		let lumens_slider_state = cx.new(|cx| {
			return SliderState::new()
				.default_value(state.read(cx).lumens_threshold)
				.min(MIN_LUMENS_THRESHOLD)
				.max(MAX_LUMENS_THRESHOLD);
		});

		cx.subscribe(&lumens_slider_state, |this, _, event: &SliderEvent, cx| {
			if let SliderEvent::Change(value) = event {
				this.state.update(cx, |this, cx| {
					this.lumens_threshold = value.start();
					cx.notify();
				});
			};
		})
		.detach();

		let seconds_slider_state = cx.new(|cx| {
			return SliderState::new()
				.default_value(state.read(cx).seconds_threshold)
				.min(MIN_SECONDS_THRESHOLD)
				.max(MAX_SECONDS_THRESHOLD);
		});

		cx.subscribe(&seconds_slider_state, |this, _, event: &SliderEvent, cx| {
			if let SliderEvent::Change(value) = event {
				this.state.update(cx, |this, cx| {
					this.seconds_threshold = value.start();
					cx.notify();
				});
			}
		})
		.detach();

		return Self {
			state,
			lumens_slider_state,
			seconds_slider_state,
		};
	}

	fn toggles(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		let Self { state, .. } = self;

		let enable_theme_switching = state.read(cx).enable_theme_switching;
		let enable_autostart = state.read(cx).enable_autostart;

		return v_flex()
			.gap_2()
			.child(
				Checkbox::new("autostart")
					.cursor_pointer()
					.label("Start at login")
					.checked(enable_autostart)
					.on_click(cx.listener(|App { state, .. }, checked, _, cx| {
						state.update(cx, |this, cx| {
							this.enable_autostart = *checked;
							cx.notify();
						})
					})),
			)
			.child(
				Checkbox::new("enable_theme_switching")
					.cursor_pointer()
					.label("Enable theme switching")
					.checked(enable_theme_switching)
					.on_click(cx.listener(|App { state, .. }, checked, _, cx| {
						state.update(cx, |this, cx| {
							this.enable_theme_switching = *checked;
							cx.notify();
						});
					})),
			);
	}

	fn lumens_slider(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		let Self { state, .. } = self;

		let lumens_threshold = state.read(cx).lumens_threshold;

		return div()
			.w_full()
			.gap_2()
			.child("Ambient light threshold:")
			.child(
				h_flex()
					.w_full()
					.gap_4()
					.child(
						Slider::new(&self.lumens_slider_state)
							.bg(cx.theme().accent)
							.cursor_pointer(),
					)
					.child(
						Button::new("reset_lumens")
							.label("Reset")
							.primary()
							.cursor_pointer()
							.on_click(cx.listener(
								|App {
								   state,
								   lumens_slider_state,
								   ..
								 },
								 _,
								 window,
								 cx| {
									state.update(cx, |this, cx| {
										this.lumens_threshold = DEFAULT_LUMENS_THRESHOLD;
										cx.notify();
									});

									lumens_slider_state.update(cx, |this, cx| {
										this.set_value(DEFAULT_LUMENS_THRESHOLD, window, cx);
										cx.notify();
									});
								},
							)),
					),
			)
			.text_color(cx.theme().muted_foreground)
			.child(format!("{lumens_threshold}"));
	}

	fn seconds_slider(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		let Self { state, .. } = self;

		let seconds_threshold = state.read(cx).seconds_threshold;

		return div()
			.w_full()
			.gap_2()
			.child("Delay time:")
			.child(
				h_flex()
					.w_full()
					.gap_4()
					.child(
						Slider::new(&self.seconds_slider_state)
							.bg(cx.theme().accent)
							.cursor_pointer(),
					)
					.child(
						Button::new("reset_seconds")
							.label("Reset")
							.primary()
							.cursor_pointer()
							.on_click(cx.listener(
								|App {
								   state,
								   seconds_slider_state,
								   ..
								 },
								 _,
								 window,
								 cx| {
									state.update(cx, |this, cx| {
										this.seconds_threshold = DEFAULT_SECONDS_THRESHOLD;
										cx.notify();
									});

									seconds_slider_state.update(cx, |this, cx| {
										this.set_value(DEFAULT_SECONDS_THRESHOLD, window, cx);
									});
								},
							)),
					),
			)
			.text_color(cx.theme().muted_foreground)
			.child(format!("{seconds_threshold}s",));
	}

	fn explainer_text(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		let Self { state, .. } = self;

		let lumen_threshold = state.read(cx).lumens_threshold;
		let seconds_threshold = state.read(cx).seconds_threshold;

		return div()
			.w_full()
			.justify_center()
			.text_align(TextAlign::Center)
			.child(format!("When the ambient light drops below {lumen_threshold} lumens for at least {seconds_threshold} seconds, the theme will switch to dark mode."));
	}

	fn body(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		return v_flex()
			.size_full()
			.p_4()
			.justify_center()
			.items_center()
			.content_center()
			.gap_4()
			.child(self.toggles(cx))
			.child(Separator::horizontal().w_full())
			.child(self.lumens_slider(cx))
			.child(self.seconds_slider(cx))
			.child(self.explainer_text(cx));
	}
}

impl Render for App {
	fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
		return v_flex()
			.size_full()
			.child(
				TitleBar::new().child(
					h_flex()
						.w_full()
						.justify_center()
						.child("Multiplatform Ambient Light Sensor Theme Switcher"),
				),
			)
			.child(self.body(cx));
	}
}

fn override_colours(cx: &mut gpui::App) {
	let theme = Theme::global_mut(cx);
	theme.accent = theme.blue;
}

fn update_theme(cx: &mut gpui::App, window: Option<&mut Window>) {
	Theme::sync_system_appearance(window, cx);
	Theme::sync_scrollbar_appearance(cx);
	override_colours(cx);
}

fn main() {
	env_logger::init();

	gpui_platform::application()
		.with_assets(gpui_component_assets::Assets)
		.run(move |cx| {
			trace!("Initialising GPUI component assets...");
			gpui_component::init(cx);

			update_theme(cx, None);
			trace!("Initialising window theme, {:#?}", cx.window_appearance());

			let window_options = WindowOptions {
				titlebar: Some(TitleBar::title_bar_options()),
				window_bounds: Some(WindowBounds::centered(size(px(600.), px(400.)), cx)),
				window_decorations: Some(WindowDecorations::Client),
				is_resizable: false,
				is_movable: false,
				is_minimizable: false,
				..Default::default()
			};

			let state = cx.new(|_| {
				return AppState::read_config().unwrap_or_default();
			});

			cx.spawn(async move |cx| {
				return cx
					.open_window(window_options, |window, cx| {
						window.activate_window();
						window.set_window_title("Multiplatform Ambient Light Sensor Theme Switcher");

						window
							.observe_window_appearance(|window, cx| {
								trace!("Window appearance changed, {:#?}", window.appearance());

								update_theme(cx, Some(window));
								cx.refresh_windows();
							})
							.detach();

						cx.observe(&state, |view, cx| {
							if let Err(e) = view.read(cx).save_config() {
								error!("Failed to save config: {e}");
							}
						})
						.detach();

						let view = cx.new(|cx| return App::new(state, cx));

						return cx.new(|cx| return Root::new(view, window, cx));
					})
					.expect("Failed to open window");
			})
			.detach();
		})
}
