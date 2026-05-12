use futures::StreamExt;
use gpui::{prelude::*, *};
use gpui_component::{
    ActiveTheme, Root, Theme, ThemeMode, TitleBar,
    button::{Button, ButtonVariants},
    checkbox::Checkbox,
    h_flex,
    slider::{Slider, SliderEvent, SliderState},
    v_flex,
};
use system_theme::{SystemTheme, ThemeScheme};

pub struct App {
    enable_theme_switching: bool,
    lumens_slider_state: Entity<SliderState>,
    lumens_threshold: f32,
    lumens_slider_subscription: Subscription,
    seconds_slider_state: Entity<SliderState>,
    seconds_threshold: f32,
    seconds_slider_subscription: Subscription,
}

impl App {
    const DEFAULT_LUMENS_THRESHOLD: f32 = 100.0;
    const DEFAULT_SECONDS_THRESHOLD: f32 = 30.0;

    fn new(cx: &mut Context<Self>) -> Self {
        let lumens_slider_state = cx.new(|_| {
            SliderState::new()
                .default_value(Self::DEFAULT_LUMENS_THRESHOLD)
                .min(10.)
                .max(15_000.)
        });

        let lumens_slider_subscription =
            cx.subscribe(&lumens_slider_state, |this, _, event: &SliderEvent, cx| {
                if let SliderEvent::Change(value) = event {
                    this.lumens_threshold = value.start();
                    cx.notify();
                }
            });

        let seconds_slider_state = cx.new(|_| {
            SliderState::new()
                .default_value(Self::DEFAULT_SECONDS_THRESHOLD)
                .min(10.)
                .max(120.)
        });

        let seconds_slider_subscription =
            cx.subscribe(&seconds_slider_state, |this, _, event: &SliderEvent, cx| {
                if let SliderEvent::Change(value) = event {
                    this.seconds_threshold = value.start();
                    cx.notify();
                }
            });

        Self {
            enable_theme_switching: false,
            lumens_slider_state,
            lumens_threshold: Self::DEFAULT_LUMENS_THRESHOLD,
            lumens_slider_subscription,
            seconds_slider_state,
            seconds_threshold: Self::DEFAULT_SECONDS_THRESHOLD,
            seconds_slider_subscription,
        }
    }

    fn enable_theme_toggle(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        Checkbox::new("enable_theme_switching")
            .label("Enable theme switching")
            .checked(self.enable_theme_switching)
            .on_click(cx.listener(|view, checked, _, cx| {
                view.enable_theme_switching = *checked;
                cx.notify();
            }))
    }

    fn lumens_slider(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(
                h_flex()
                    .child(Slider::new(&self.lumens_slider_state))
                    .child(Button::new("reset_lumens").label("Reset").primary()),
            )
            .child(format!("Threshold: {}", self.lumens_threshold))
    }

    fn seconds_slider(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(Slider::new(&self.seconds_slider_state))
            .child(format!("Seconds: {}", self.seconds_threshold))
    }

    fn body(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .justify_center()
            .items_center()
            .gap_2()
            .child(self.enable_theme_toggle(cx))
            .child(self.lumens_slider(cx))
            .child(self.seconds_slider(cx))
    }
}

impl Render for App {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(TitleBar::new())
            .child(self.body(cx))
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(move |cx| {
            gpui_component::init(cx);

            Theme::sync_system_appearance(None, cx);
            Theme::sync_scrollbar_appearance(cx);

            let window_options = WindowOptions {
                titlebar: Some(TitleBar::title_bar_options()),
                window_bounds: Some(WindowBounds::centered(size(px(600.), px(600.)), cx)),
                window_decorations: Some(WindowDecorations::Client),
                is_resizable: false,
                is_movable: false,
                is_minimizable: false,
                ..Default::default()
            };

            cx.spawn(async move |cx| {
                cx.open_window(window_options, |window, cx| {
                    window.activate_window();
                    window.set_window_title("Multiplatform Ambient Light Sensor Theme Switcher");

                    // cx.new(|cx| {
                    //     cx.background_spawn(async move |cx| {
                    //         let system_theme =
                    //             SystemTheme::new().expect("Failed to create system theme");
                    //         let mut stream = system_theme.subscribe();

                    //         for update in stream.next().await {
                    //             use system_theme:ThemeScheme;

                    //             match system_theme.get_scheme().ok() {

                    //               Some(ThemeScheme::Dark) => {

                    //               }
                    //               Some(ThemeScheme::Light) => {}
                    //               _ => {}
                    //             };
                    //         }
                    //         }.into())
                    // });

                    let view = cx.new(App::new);

                    println!("Theme: {:?}", window.appearance());

                    Theme::sync_system_appearance(Some(window), cx);
                    Theme::sync_scrollbar_appearance(cx);

                    _ = window.observe_window_appearance(|window, cx| {
                        println!("Theme changed: {:?}", window.appearance());

                        Theme::sync_system_appearance(Some(window), cx);
                        Theme::sync_scrollbar_appearance(cx);
                    });

                    cx.new(|cx| Root::new(view, window, cx))
                })
                .expect("Failed to open window")
            })
            .detach();
        })
}
