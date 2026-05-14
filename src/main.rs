#![allow(clippy::needless_return)]
#![warn(clippy::implicit_return)]

use gpui::{prelude::*, *};
use gpui_component::{
	ActiveTheme, Root, Theme, TitleBar,
	button::{Button, ButtonVariants},
	checkbox::Checkbox,
	h_flex,
	slider::{Slider, SliderEvent, SliderState},
	v_flex,
};

pub struct App {
	enable_theme_switching: Entity<bool>,
	enable_autostart: Entity<bool>,
	lumens_slider_state: Entity<SliderState>,
	lumens_threshold: Entity<f32>,
	seconds_slider_state: Entity<SliderState>,
	seconds_threshold: Entity<f32>,
}

impl App {
	const DEFAULT_LUMENS_THRESHOLD: f32 = 100.0;
	const DEFAULT_SECONDS_THRESHOLD: f32 = 30.0;

	fn new(cx: &mut Context<Self>) -> Self {
		let lumens_slider_state = cx.new(|_| {
			return SliderState::new()
				.default_value(Self::DEFAULT_LUMENS_THRESHOLD)
				.min(10.)
				.max(2000.);
		});

		cx.subscribe(&lumens_slider_state, |this, _, event: &SliderEvent, cx| {
			match event {
				SliderEvent::Change(value) | SliderEvent::Release(value) => {
					this.lumens_threshold.update(cx, |this, cx| {
						*this = value.start();
						cx.notify();
					});
				}
			};
		})
		.detach();

		let seconds_slider_state = cx.new(|_| {
			return SliderState::new()
				.default_value(Self::DEFAULT_SECONDS_THRESHOLD)
				.min(10.)
				.max(120.);
		});

		cx.subscribe(
			&seconds_slider_state,
			|this, _, event: &SliderEvent, cx| match event {
				SliderEvent::Change(value) | SliderEvent::Release(value) => {
					this.seconds_threshold.update(cx, |this, cx| {
						*this = value.start();
						cx.notify();
					});
				}
			},
		)
		.detach();

		return Self {
			enable_theme_switching: cx.new(|_| return false),
			enable_autostart: cx.new(|_| return false),
			lumens_slider_state,
			lumens_threshold: cx.new(|_| return Self::DEFAULT_LUMENS_THRESHOLD),
			seconds_slider_state,
			seconds_threshold: cx.new(|_| return Self::DEFAULT_SECONDS_THRESHOLD),
		};
	}

	fn enable_theme_toggle(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		let enable_theme_switching = self.enable_theme_switching.read(cx);
		let enable_autostart = self.enable_autostart.read(cx);

		return v_flex()
			.gap_2()
			.child(
				Checkbox::new("autostart")
					.label("Start at login")
					.checked(*enable_autostart)
					.on_click(cx.listener(|view, checked, _, cx| {
						view.enable_autostart.update(cx, |this, cx| {
							*this = *checked;
							cx.notify();
						})
					})),
			)
			.child(
				Checkbox::new("enable_theme_switching")
					.label("Enable theme switching")
					.checked(*enable_theme_switching)
					.on_click(cx.listener(|view, checked, _, cx| {
						view.enable_theme_switching.update(cx, |this, cx| {
							*this = *checked;
							cx.notify();
						})
					})),
			);
	}

	fn lumens_slider(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		let lumens_threshold = self.lumens_threshold.read(cx);

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
							.on_click(cx.listener(|view, _, window, cx| {
								view.lumens_threshold.update(cx, |this, cx| {
									*this = Self::DEFAULT_LUMENS_THRESHOLD;
									cx.notify();
								});
								view.lumens_slider_state.update(cx, |this, cx| {
									this.set_value(Self::DEFAULT_LUMENS_THRESHOLD, window, cx);
									cx.notify();
								});
							})),
					),
			)
			.text_color(cx.theme().muted_foreground)
			.child(format!("{lumens_threshold}"));
	}

	fn seconds_slider(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		let seconds_threshold = self.seconds_threshold.read(cx);

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
							.on_click(cx.listener(|view, _, window, cx| {
								view.seconds_threshold.update(cx, |this, cx| {
									*this = Self::DEFAULT_SECONDS_THRESHOLD;
									cx.notify();
								});
								view.seconds_slider_state.update(cx, |this, cx| {
									this.set_value(Self::DEFAULT_SECONDS_THRESHOLD, window, cx);
								});
							})),
					),
			)
			.text_color(cx.theme().muted_foreground)
			.child(format!("{seconds_threshold}s",));
	}

	fn explainer_text(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
		let lumen_threshold = self.lumens_threshold.read(cx);
		let seconds_threshold = self.seconds_threshold.read(cx);

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
			.child(self.enable_theme_toggle(cx))
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

fn main() {
	gpui_platform::application()
		.with_assets(gpui_component_assets::Assets)
		.run(move |cx| {
			gpui_component::init(cx);

			Theme::sync_system_appearance(None, cx);
			Theme::sync_scrollbar_appearance(cx);
			override_colours(cx);

			let window_options = WindowOptions {
				titlebar: Some(TitleBar::title_bar_options()),
				window_bounds: Some(WindowBounds::centered(size(px(600.), px(400.)), cx)),
				window_decorations: Some(WindowDecorations::Client),
				is_resizable: false,
				is_movable: false,
				is_minimizable: false,
				..Default::default()
			};

			cx.spawn(async move |cx| {
				return cx
					.open_window(window_options, |window, cx| {
						window.activate_window();
						window.set_window_title("Multiplatform Ambient Light Sensor Theme Switcher");

						window
							.observe_window_appearance(|window, cx| {
								Theme::sync_system_appearance(Some(window), cx);
								Theme::sync_scrollbar_appearance(cx);
								override_colours(cx);
								cx.refresh_windows();
							})
							.detach();

						let view = cx.new(App::new);

						return cx.new(|cx| return Root::new(view, window, cx));
					})
					.expect("Failed to open window");
			})
			.detach();
		})
}
