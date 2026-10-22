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
            set_default_width: 780,
            set_default_height: 480,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 0,
                set_margin_top: 22,
                set_margin_bottom: 18,
                set_margin_start: 24,
                set_margin_end: 24,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,
                    set_margin_bottom: 20,
                    add_css_class: "topbar",

                    gtk::Label {
                        set_label: "♫",
                        add_css_class: "brand-mark",
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 1,

                        gtk::Label {
                            set_label: "MUSIC PLAYER",
                            set_halign: gtk::Align::Start,
                            add_css_class: "eyebrow",
                        },

                        gtk::Label {
                            set_label: "Murglar Controller",
                            set_halign: gtk::Align::Start,
                            add_css_class: "brand-title",
                        },
                    },

                    gtk::Box {
                        set_hexpand: true,
                    },

                    gtk::Button {
                        set_icon_name: "emblem-system-symbolic",
                        set_tooltip_text: Some("Настройки"),
                        set_size_request: (38, 38),
                        add_css_class: "settings-open-button",
                        connect_clicked => Msg::OpenSettings,
                    },
                },

                gtk::Separator {
                    add_css_class: "separator",
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 28,
                    set_margin_top: 24,
                    set_margin_bottom: 22,
                    set_vexpand: true,
                    add_css_class: "workspace",

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 14,
                        set_width_request: 270,
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,
                        add_css_class: "cover-column",

                        gtk::Picture {
                            #[watch]
                            set_filename: model.art_path.as_deref(),
                            set_can_shrink: true,
                            set_size_request: (250, 250),
                            set_halign: gtk::Align::Fill,
                            add_css_class: "album-cover",
                        },

                        gtk::Label {
                            set_label: "NOW PLAYING",
                            set_halign: gtk::Align::Start,
                            add_css_class: "eyebrow",
                        },

                        gtk::Label {
                            #[watch]
                            set_label: &model.title,
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            set_xalign: 0.0,
                            set_max_width_chars: 30,
                            add_css_class: "track-title",
                        },

                        gtk::Label {
                            #[watch]
                            set_label: &model.artist,
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            set_xalign: 0.0,
                            set_max_width_chars: 36,
                            add_css_class: "artist",
                        },
                    },

                    gtk::Separator {
                        set_orientation: gtk::Orientation::Vertical,
                        add_css_class: "separator",
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 16,
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,
                        add_css_class: "player-column",

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 3,
                                set_hexpand: true,

                                gtk::Label {
                                    set_label: "PLAYBACK",
                                    set_halign: gtk::Align::Start,
                                    add_css_class: "eyebrow",
                                },

                                gtk::Label {
                                    set_label: "Сейчас играет",
                                    set_halign: gtk::Align::Start,
                                    add_css_class: "section-title",
                                },
                            },

                            gtk::Label {
                                #[watch]
                                set_label: &model.status,
                                set_wrap: true,
                                set_max_width_chars: 24,
                                set_halign: gtk::Align::End,
                                add_css_class: "status-label",
                            },
                        },

                        gtk::Scale {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_digits: 0,
                            set_draw_value: false,
                            set_hexpand: true,
                            add_css_class: "seek-scale",

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

                            gtk::Label {
                                #[watch]
                                set_label: &format_time(model.position),
                                set_halign: gtk::Align::Start,
                                add_css_class: "time-label",
                            },

                            gtk::Box {
                                set_hexpand: true,
                            },

                            gtk::Label {
                                #[watch]
                                set_label: &format_time(model.duration),
                                set_halign: gtk::Align::End,
                                add_css_class: "time-label",
                            },
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            set_halign: gtk::Align::Center,
                            set_margin_top: 8,
                            set_margin_bottom: 8,
                            add_css_class: "transport-row",

                            gtk::Button {
                                set_icon_name: "media-skip-backward-symbolic",
                                set_tooltip_text: Some("Предыдущий трек"),
                                add_css_class: "transport-button",
                                connect_clicked => Msg::Previous,
                            },

                            gtk::Button {
                                #[watch]
                                set_icon_name: if model.status == "Playing" {
                                    "media-playback-pause-symbolic"
                                } else {
                                    "media-playback-start-symbolic"
                                },
                                set_tooltip_text: Some("Воспроизвести / пауза"),
                                add_css_class: "play-button",
                                connect_clicked => Msg::Toggle,
                            },

                            gtk::Button {
                                set_icon_name: "media-skip-forward-symbolic",
                                set_tooltip_text: Some("Следующий трек"),
                                add_css_class: "transport-button",
                                connect_clicked => Msg::Next,
                            },
                        },

                        gtk::Separator {
                            add_css_class: "separator",
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            set_margin_top: 8,
                            add_css_class: "volume-row",

                            gtk::Image {
                                set_icon_name: Some("audio-volume-high-symbolic"),
                                set_pixel_size: 17,
                                set_tooltip_text: Some("Громкость"),
                            },

                            gtk::Scale {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_digits: 0,
                                set_draw_value: false,
                                set_hexpand: true,
                                set_range: (0.0, 100.0),
                                add_css_class: "volume-scale",

                                #[watch]
                                #[block_signal(volume_handler)]
                                set_value: model.volume * 100.0,

                                connect_value_changed[sender = sender.clone()] => move |scale| {
                                    sender.input(Msg::SetVolume(scale.value()));
                                } @volume_handler,
                            },

                            gtk::Label {
                                #[watch]
                                set_label: &format!("{:.0}%", model.volume * 100.0),
                                set_width_chars: 4,
                                set_halign: gtk::Align::End,
                                add_css_class: "time-label",
                            },
                        },
                    },
                },

                gtk::Separator {
                    add_css_class: "separator",
                },

                gtk::Label {
                    #[watch]
                    set_label: &model.status,
                    set_halign: gtk::Align::Start,
                    set_wrap: true,
                    set_margin_top: 12,
                    add_css_class: "footer-status",
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

fn format_time(seconds: f64) -> String {
    let total = if seconds.is_finite() {
        seconds.max(0.0) as u64
    } else {
        0
    };

    format!("{:02}:{:02}", total / 60, total % 60)
}

thread_local! {
    static THEME_PROVIDER: std::cell::RefCell<Option<gtk::CssProvider>> =
        const { std::cell::RefCell::new(None) };
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
