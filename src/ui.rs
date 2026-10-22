use gtk::prelude::*;
use relm4::{gtk, prelude::*};

use crate::player;

#[derive(Debug)]
enum Msg {
    OpenSettings,
    Refresh,
    Toggle,
    Previous,
    Next,
    SetVolume(f64),
    Seek(f64),
}

struct App {
    title: String,
    artist: String,
    status: String,
    volume: f64,
    position: f64,
    duration: f64,
    track_id: Option<String>,
    art_path: Option<String>,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            set_title: Some("Murglar Controller"),
            set_default_width: 1200,
            set_default_height: 800,
            set_size_request: (720, 520),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,
                set_margin_top: 14,
                set_margin_bottom: 30,
                set_margin_start: 10,
                set_margin_end: 10,
                set_hexpand: true,
                set_vexpand: true,
                add_css_class: "main-surface",

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_halign: gtk::Align::Start,

                    gtk::Button {
                        set_icon_name: "view-list-symbolic",
                        set_tooltip_text: Some("Библиотека"),
                        set_size_request: (31, 31),
                        add_css_class: "library-button",
                    },
                },

                gtk::Box {
                    set_vexpand: true,
                    set_hexpand: true,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_start: 20,
                    set_margin_end: 60,
                    add_css_class: "bottom-player",

                    gtk::Label {
                        #[watch]
                        set_label: &format!("{} — {}", model.title, model.artist),
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_max_width_chars: 48,
                        add_css_class: "dock-track-title",
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 22,
                        set_valign: gtk::Align::Center,

                        gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,
                            set_vscrollbar_policy: gtk::PolicyType::Never,
                            set_min_content_width: 101,
                            set_min_content_height: 96,
                            set_propagate_natural_width: false,
                            set_propagate_natural_height: false,
                            set_has_frame: false,
                            set_size_request: (101, 96),
                            set_halign: gtk::Align::Start,
                            set_valign: gtk::Align::Start,
                            set_margin_start: 14,
                            add_css_class: "dock-cover-viewport",

                            gtk::Picture {
                                #[watch]
                                set_filename: model.art_path.as_deref(),
                                set_can_shrink: true,
                                set_size_request: (101, 96),
                                add_css_class: "dock-cover",
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,
                            set_hexpand: true,
                            set_valign: gtk::Align::End,

                            gtk::Scale {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_digits: 0,
                                set_draw_value: false,
                                set_hexpand: true,
                                #[watch]
                                set_tooltip_text: Some(&format!(
                                    "{} / {}",
                                    format_time(model.position),
                                    format_time(model.duration)
                                )),
                                add_css_class: "dock-seek-scale",

                                #[watch]
                                #[block_signal(seek_handler)]
                                set_range: (0.0, model.duration.max(1.0)),

                                #[watch]
                                #[block_signal(seek_handler)]
                                set_value: model.position,

                                connect_value_changed[sender = sender.clone()] => move |scale| {
                                    sender.input(Msg::Seek(scale.value()));
                                } @seek_handler,
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 12,
                                set_valign: gtk::Align::Center,
                                add_css_class: "dock-controls",

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 4,
                                    set_valign: gtk::Align::Center,
                                    set_width_request: 94,
                                    add_css_class: "dock-volume",

                                    gtk::Scale {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_digits: 0,
                                        set_draw_value: false,
                                        set_hexpand: true,
                                        set_size_request: (68, 22),
                                        set_range: (0.0, 100.0),
                                        set_tooltip_text: Some("Громкость"),
                                        add_css_class: "dock-volume-scale",

                                        #[watch]
                                        #[block_signal(volume_handler)]
                                        set_value: model.volume * 100.0,

                                        connect_value_changed[sender = sender.clone()] => move |scale| {
                                            sender.input(Msg::SetVolume(scale.value()));
                                        } @volume_handler,
                                    },

                                    gtk::Image {
                                        set_icon_name: Some("audio-volume-high-symbolic"),
                                        set_pixel_size: 22,
                                        set_tooltip_text: Some("Громкость"),
                                    },
                                },

                                gtk::Box {
                                    set_size_request: (276, 1),
                                },

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 14,
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    add_css_class: "dock-transport",

                                    gtk::Button {
                                        set_icon_name: "media-skip-backward-symbolic",
                                        set_tooltip_text: Some("Предыдущий трек"),
                                        set_size_request: (31, 31),
                                        add_css_class: "dock-skip-button",
                                        connect_clicked => Msg::Previous,
                                    },

                                    gtk::Button {
                                        #[watch]
                                        set_icon_name: if model.status == "Playing" {
                                            "media-playback-pause-symbolic"
                                        } else {
                                            "media-playback-start-symbolic"
                                        },
                                        set_tooltip_text: Some(if model.status == "Playing" {
                                            "Пауза"
                                        } else {
                                            "Воспроизвести"
                                        }),
                                        set_size_request: (43, 41),
                                        add_css_class: "dock-play-button",
                                        connect_clicked => Msg::Toggle,
                                    },

                                    gtk::Button {
                                        set_icon_name: "media-skip-forward-symbolic",
                                        set_tooltip_text: Some("Следующий трек"),
                                        set_size_request: (31, 31),
                                        add_css_class: "dock-skip-button",
                                        connect_clicked => Msg::Next,
                                    },
                                },

                                gtk::Box {
                                    set_hexpand: true,
                                },

                                gtk::Button {
                                    set_icon_name: "emblem-system-symbolic",
                                    set_tooltip_text: Some("Настройки"),
                                    set_size_request: (38, 38),
                                    add_css_class: "dock-settings-button",
                                    connect_clicked => Msg::OpenSettings,
                                },
                            },
                        },
                    },
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.status,
                    #[watch]
                    set_visible: !matches!(model.status.as_str(), "Playing" | "Paused" | "Stopped"),
                    set_halign: gtk::Align::Start,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_max_width_chars: 100,
                    set_margin_start: 20,
                    set_margin_top: 8,
                    add_css_class: "dock-status",
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = App {
            title: "Загрузка…".to_owned(),
            artist: String::new(),
            status: "Подключение к Murglar…".to_owned(),
            volume: 0.08,
            position: 0.0,
            duration: 1.0,
            track_id: None,
            art_path: None,
        };

        refresh_model(&mut model);

        let timer_sender = sender.clone();
        gtk::glib::timeout_add_seconds_local(1, move || {
            timer_sender.input(Msg::Refresh);
            gtk::glib::ControlFlow::Continue
        });

        install_css();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Msg::OpenSettings => {
                let window = crate::settings::build_window();
                window.present();
            }

            Msg::Refresh => refresh_model(self),

            Msg::Toggle => run_action(self, "PlayPause"),
            Msg::Previous => run_action(self, "Previous"),
            Msg::Next => run_action(self, "Next"),

            Msg::SetVolume(percent) => {
                if !percent.is_finite() || !(0.0..=100.0).contains(&percent) {
                    return;
                }

                match player::set_volume(percent / 100.0) {
                    Ok(()) => self.volume = percent / 100.0,
                    Err(error) => self.status = format!("Ошибка громкости: {error}"),
                }
            }

            Msg::Seek(seconds) => {
                if !seconds.is_finite() || seconds < 0.0 {
                    return;
                }

                let Some(track_id) = self.track_id.as_deref() else {
                    return;
                };

                match player::set_position(track_id, seconds.min(self.duration)) {
                    Ok(()) => self.position = seconds.min(self.duration),
                    Err(error) => self.status = format!("Ошибка перемотки: {error}"),
                }
            }
        }
    }
}

fn refresh_model(model: &mut App) {
    match player::read_status() {
        Ok(info) => {
            model.title = info.title;
            model.artist = info.artist;
            model.status = info.status;
            model.volume = info.volume.clamp(0.0, 1.0);
            model.position = info.position;
            model.duration = info.duration.max(1.0);
            model.track_id = info.track_id;
            model.art_path = info
                .art_url
                .as_deref()
                .and_then(|url| url.strip_prefix("file://"))
                .map(str::to_owned);
        }
        Err(error) => {
            model.status = format!("Ошибка подключения: {error}");
        }
    }
}

fn run_action(model: &mut App, method: &str) {
    match player::call_player(method) {
        Ok(()) => refresh_model(model),
        Err(error) => model.status = format!("Ошибка управления: {error}"),
    }
}

thread_local! {
    static THEME_PROVIDER: std::cell::RefCell<Option<gtk::CssProvider>> =
        const { std::cell::RefCell::new(None) };
}

fn format_time(seconds: f64) -> String {
    let total = if seconds.is_finite() {
        seconds.max(0.0) as u64
    } else {
        0
    };

    format!("{:02}:{:02}", total / 60, total % 60)
}

fn theme_config_path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".config"))
        })?;
    Some(base.join("murglar-controller").join("theme"))
}

fn save_theme(theme: &str) {
    let Some(path) = theme_config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent).and_then(|_| std::fs::write(&path, theme)) {
            eprintln!("Не удалось сохранить тему: {e}");
        }
    }
}

fn load_saved_theme() -> String {
    theme_config_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| {
            matches!(
                s.as_str(),
                "glass" | "matte" | "cyberpunk" | "midnight" | "forest" | "amber"
            )
        })
        .unwrap_or_else(|| "glass".to_owned())
}

pub fn apply_theme(theme: &str) {
    let file = match theme {
        "matte" => "matte.css",
        "cyberpunk" => "cyberpunk.css",
        "midnight" => "midnight.css",
        "forest" => "forest.css",
        "amber" => "amber.css",
        _ => "dark.css",
    };

    // CSS подключается динамически, чтобы смена темы работала без перезапуска.
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("themes")
        .join(file);
    let provider = gtk::CssProvider::new();
    if let Ok(css) = std::fs::read_to_string(path) {
        provider.load_from_data(&css);
    } else {
        eprintln!("Не удалось загрузить файл темы: {file}");
        return;
    }

    if let Some(display) = gtk::gdk::Display::default() {
        THEME_PROVIDER.with(|slot| {
            if let Some(old) = slot.borrow_mut().take() {
                gtk::style_context_remove_provider_for_display(&display, &old);
            }
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
            *slot.borrow_mut() = Some(provider);
        });
        save_theme(theme);
    }
}

fn install_css() {
    let theme = load_saved_theme();
    apply_theme(&theme);
}

pub fn run() {
    RelmApp::new("ru.murglar.controller").run::<App>(());
}
