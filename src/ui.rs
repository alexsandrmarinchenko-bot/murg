use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
};

use gtk::prelude::*;
use relm4::gtk;

use crate::player;

const DESIGN_CSS: &str = r#"
window.figma-main {
    background-color: #202020;
    background-image: none;
    color: #c3c3c3;
}

.design-canvas {
    background-color: #202020;
    background-image: none;
    color: #c3c3c3;
}

label.figma-track-title {
    color: #c3c3c3;
    font-family: "Inter", "Noto Sans", sans-serif;
    font-size: 20px;
    font-weight: 400;
    letter-spacing: 0.2px;
    padding: 0;
    margin: 0;
}

label.figma-artist {
    color: #ffffff;
    font-family: "Inter", "Noto Sans", sans-serif;
    font-size: 15px;
    font-weight: 400;
    letter-spacing: 0.15px;
    padding: 0;
    margin: 0;
}

box.figma-pause-mask {
    background-color: #3c3b3b;
    background-image: none;
    border: none;
    border-radius: 0;
    padding: 0;
    margin: 0;
}

box.figma-pause-bar {
    background-color: #000000;
    background-image: none;
    border: none;
    border-radius: 0;
    padding: 0;
    margin: 0;
}

picture.figma-cover {
    background-color: #d9d9d9;
    background-image: none;
    border: none;
    border-radius: 10px;
    padding: 0;
    margin: 0;
}

button.figma-control {
    background-image: none;
    background-color: transparent;
    color: transparent;
    border: none;
    border-radius: 0;
    box-shadow: none;
    padding: 0;
    margin: 0;
    min-width: 0;
    min-height: 0;
}

button.figma-control:hover,
button.figma-control:active,
button.figma-control:checked {
    background-image: none;
    background-color: transparent;
    border: none;
    box-shadow: none;
}

button.figma-control > picture,
button.figma-control > overlay,
button.figma-control picture,
button.figma-control image {
    padding: 0;
    margin: 0;
    border: none;
    box-shadow: none;
}

button.figma-playing {
    background-image: none;
    background-color: #3c3b3b;
    color: #c3c3c3;
    border: 1px solid #646464;
    border-radius: 8px;
    box-shadow: none;
    padding: 0;
    margin: 0;
    min-width: 43px;
    min-height: 41px;
}

button.figma-playing:hover,
button.figma-playing:active {
    background-color: #3c3b3b;
    border-color: #646464;
}

scale.figma-seek,
scale.figma-volume {
    background-image: none;
    background-color: transparent;
    padding: 0;
    margin: 0;
    min-height: 0;
}

scale.figma-seek trough {
    min-height: 5px;
    background-color: #d9d9d9;
    background-image: none;
    border: none;
    border-radius: 0;
    box-shadow: none;
}

scale.figma-seek highlight {
    min-height: 5px;
    background-color: #d9d9d9;
    background-image: none;
    border: none;
    border-radius: 0;
    box-shadow: none;
}

scale.figma-seek slider {
    min-width: 23px;
    min-height: 23px;
    background-color: #d9d9d9;
    background-image: none;
    border: none;
    border-radius: 4px;
    box-shadow: none;
}

scale.figma-seek slider:hover {
    background-color: #d9d9d9;
}

scale.figma-volume {
    min-height: 22px;
    padding: 0;
    margin: 0;
}

scale.figma-volume trough {
    min-height: 5px;
    background-color: transparent;
    background-image: @VOLUME_TRACK_URL@;
    background-repeat: no-repeat;
    background-position: center;
    border: none;
    border-radius: 0;
    box-shadow: none;
}

scale.figma-volume highlight {
    min-height: 5px;
    background-color: #d9d9d9;
    background-image: none;
    border: none;
    border-radius: 0;
    box-shadow: none;
}

scale.figma-volume slider {
    min-width: 23px;
    min-height: 23px;
    background-color: transparent;
    background-image: @VOLUME_THUMB_URL@;
    background-repeat: no-repeat;
    background-position: center;
    border: none;
    border-radius: 0;
    box-shadow: none;
}

label.figma-status {
    color: #c3c3c3;
    font-family: "Inter", "Noto Sans", sans-serif;
    font-size: 10px;
    padding: 0;
    margin: 0;
}
"#;

#[derive(Clone)]
struct PlayerSnapshot {
    title: String,
    artist: String,
    status: String,
    volume: f64,
    position: f64,
    duration: f64,
    track_id: Option<String>,
    art_path: Option<String>,
}

impl Default for PlayerSnapshot {
    fn default() -> Self {
        Self {
            title: "название трека".to_owned(),
            artist: "исполнитель".to_owned(),
            status: "Stopped".to_owned(),
            volume: 0.08,
            position: 0.0,
            duration: 1.0,
            track_id: None,
            art_path: None,
        }
    }
}

#[derive(Clone)]
struct UiWidgets {
    title: gtk::Label,
    artist: gtk::Label,
    cover: gtk::Picture,
    seek: gtk::Scale,
    volume: gtk::Scale,
    play: gtk::Button,
    status: gtk::Label,
    updating_seek: Rc<Cell<bool>>,
    updating_volume: Rc<Cell<bool>>,
    pause_overlay: gtk::Fixed,
}

fn project_path(relative: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // Prefer the newest Figma export in Frame-1 when both exports are present.
    // Fall back to Frame for older local checkouts.
    let preferred = if let Some(rest) = relative.strip_prefix("Frame/") {
        Some(root.join("Frame-1").join(rest))
    } else if let Some(rest) = relative.strip_prefix("Frame-1/") {
        Some(root.join("Frame").join(rest))
    } else {
        None
    };

    if let Some(path) = preferred.filter(|path| path.is_file()) {
        return path;
    }

    let requested = root.join(relative);
    if requested.is_file() {
        return requested;
    }

    requested
}

fn picture_from_project_file(relative: &str, width: i32, height: i32) -> gtk::Picture {
    let picture = gtk::Picture::new();
    if let Some(path) = project_path(relative).to_str() {
        picture.set_filename(Some(path));
    }
    picture.set_size_request(width, height);
    picture.set_can_shrink(true);
    picture.set_keep_aspect_ratio(false);
    picture
}

fn install_design_css() {
    let provider = gtk::CssProvider::new();
    let volume_track = project_path("Frame/assets/image-49194f4ab8df0950.png");
    let volume_thumb = project_path("Frame/assets/image-a71d83b2d743a058.png");
    let volume_track_url = format!("url(\"file://{}\")", volume_track.display());
    let volume_thumb_url = format!("url(\"file://{}\")", volume_thumb.display());
    let css = DESIGN_CSS
        .replace("@VOLUME_TRACK_URL@", &volume_track_url)
        .replace("@VOLUME_THUMB_URL@", &volume_thumb_url);
    provider.load_from_data(&css);

    if let Some(display) = gtk::gdk::Display::default() {
        // Slightly above APPLICATION priority so the exported Figma layout remains
        // faithful even if a theme is selected in the existing settings window.
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION + 1,
        );
        DESIGN_PROVIDER.with(|slot| *slot.borrow_mut() = Some(provider));
    }
}

thread_local! {
    static DESIGN_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
    static THEME_PROVIDER: RefCell<Option<gtk::CssProvider>> = const { RefCell::new(None) };
}

fn format_time(seconds: f64) -> String {
    let safe_seconds = if seconds.is_finite() {
        seconds.max(0.0) as u64
    } else {
        0
    };
    format!("{:02}:{:02}", safe_seconds / 60, safe_seconds % 60)
}

fn apply_play_icon(ui: &UiWidgets, is_playing: bool) {
    ui.pause_overlay.set_visible(is_playing);
}

fn refresh_player(model: &Rc<RefCell<PlayerSnapshot>>, ui: &UiWidgets) {
    let info = match player::read_status() {
        Ok(info) => info,
        Err(_) => return,
    };

    let art_path = info
        .art_url
        .as_deref()
        .and_then(|url| url.strip_prefix("file://"))
        .map(str::to_owned);

    let art_changed;
    {
        let mut current = model.borrow_mut();
        art_changed = current.art_path != art_path;
        current.title = info.title;
        current.artist = info.artist;
        current.status = info.status;
        current.volume = if info.volume.is_finite() {
            info.volume.clamp(0.0, 1.0)
        } else {
            0.0
        };
        current.position = info.position.max(0.0);
        current.duration = info.duration.max(1.0);
        current.track_id = info.track_id;
        current.art_path = art_path;
    }

    let current = model.borrow();
    ui.title.set_text(&current.title);
    ui.artist.set_text(&current.artist);
    if art_changed {
        ui.cover.set_filename(current.art_path.as_deref());
    }

    ui.updating_seek.set(true);
    ui.seek.set_range(0.0, current.duration.max(1.0));
    ui.seek.set_value(current.position.min(current.duration));
    ui.updating_seek.set(false);

    ui.updating_volume.set(true);
    ui.volume.set_value(current.volume * 100.0);
    ui.updating_volume.set(false);

    ui.seek.set_tooltip_text(Some(&format!(
        "{} / {}",
        format_time(current.position),
        format_time(current.duration)
    )));
    ui.status.set_visible(false);
    apply_play_icon(ui, current.status == "Playing");
}

fn show_error(ui: &UiWidgets, message: &str) {
    ui.status.set_text(message);
    ui.status.set_visible(true);
}

fn run_player_action(method: &str, model: &Rc<RefCell<PlayerSnapshot>>, ui: &UiWidgets) {
    match player::call_player(method) {
        Ok(()) => refresh_player(model, ui),
        Err(error) => show_error(ui, &format!("Ошибка управления: {error}")),
    }
}

fn build_main_window(application: &gtk::Application) -> gtk::ApplicationWindow {
    apply_theme(&load_saved_theme());
    install_design_css();

    let window = gtk::ApplicationWindow::new(application);
    window.set_title(Some("Murglar Controller"));
    window.set_default_size(1280, 720);
    window.set_size_request(720, 520);
    window.add_css_class("figma-main");

    // Use a fixed coordinate canvas like the 1280x720 Figma artboard.
    // GtkBox spacing/margins can be compressed and shift unrelated elements,
    // so each top-level widget is anchored directly to the canvas.
    let root = gtk::Fixed::new();
    root.set_hexpand(true);
    root.set_vexpand(true);
    root.set_size_request(1280, 720);
    root.add_css_class("design-canvas");
    window.set_child(Some(&root));

    // Library control: exact Figma position x=25, y=28.
    let header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    header.set_size_request(31, 31);
    header.set_halign(gtk::Align::Start);
    header.set_valign(gtk::Align::Start);

    let library = gtk::Button::new();
    library.add_css_class("figma-control");
    library.set_size_request(31, 31);
    library.set_tooltip_text(Some("Библиотека"));
    library.set_focusable(false);

    let library_layers = gtk::Fixed::new();
    library_layers.set_size_request(31, 31);
    library_layers.put(
        &picture_from_project_file("Frame/assets/image-d81a3bd1d543f942.png", 31, 31),
        0.0,
        0.0,
    );
    library_layers.put(
        &picture_from_project_file("Frame/assets/image-99c1e7fe16e3ea5d.png", 22, 18),
        4.47,
        6.30,
    );
    library.set_child(Some(&library_layers));
    header.append(&library);
    root.put(&header, 25.0, 28.0);

    // Dock begins at y=548; its children align with the Figma coordinates:
    // title y=548, artist y=574, cover y=598, seek y=622.
    let dock = gtk::Box::new(gtk::Orientation::Vertical, 0);
    dock.set_size_request(1229, 146);
    dock.set_hexpand(true);
    dock.add_css_class("player-dock");

    let title = gtk::Label::new(Some("название трека"));
    title.set_halign(gtk::Align::Start);
    title.set_valign(gtk::Align::Start);
    title.set_xalign(0.0);
    title.set_yalign(0.0);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title.set_wrap(false);
    title.set_size_request(162, 23);
    title.add_css_class("figma-track-title");
    title.set_margin_bottom(3);
    dock.append(&title);

    let artist = gtk::Label::new(Some("исполнитель"));
    artist.set_halign(gtk::Align::Start);
    artist.set_valign(gtk::Align::Start);
    artist.set_xalign(0.0);
    artist.set_yalign(0.0);
    artist.set_ellipsize(gtk::pango::EllipsizeMode::End);
    artist.set_wrap(false);
    artist.set_size_request(154, 16);
    artist.add_css_class("figma-artist");
    artist.set_margin_bottom(8);
    dock.append(&artist);

    // Cover and controls occupy one adaptive row. Only internal icon artwork
    // uses fixed local coordinates; no widget is positioned on a fixed canvas.
    let main_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    main_row.set_hexpand(true);
    main_row.set_size_request(-1, 96);
    main_row.set_valign(gtk::Align::Start);

    let cover = gtk::Picture::new();
    cover.set_size_request(96, 96);
    cover.set_can_shrink(true);
    cover.set_keep_aspect_ratio(false);
    cover.add_css_class("figma-cover");
    main_row.append(&cover);

    let controls = gtk::Box::new(gtk::Orientation::Vertical, 0);
    controls.set_hexpand(true);
    controls.set_size_request(120, 96);
    controls.set_margin_start(27);
    controls.add_css_class("figma-controls");

    let seek_top_gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
    seek_top_gap.set_size_request(1, 24);
    controls.append(&seek_top_gap);

    let seek = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 1.0);
    seek.set_draw_value(false);
    seek.set_digits(0);
    seek.set_hexpand(true);
    seek.set_size_request(1, 24);
    seek.set_tooltip_text(Some("Перемотка"));
    seek.add_css_class("figma-seek");
    controls.append(&seek);

    // A 48px action row intentionally allows transport buttons to sit 8px
    // lower than the volume slider, matching the Figma artwork.
    let action_row = gtk::Overlay::new();
    action_row.set_hexpand(true);
    action_row.set_size_request(1, 48);

    let action_base = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    action_base.set_hexpand(true);
    action_base.set_size_request(1, 48);

    let volume_group = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    volume_group.set_size_request(112, 38);
    volume_group.set_halign(gtk::Align::Start);
    volume_group.set_valign(gtk::Align::Start);
    volume_group.add_css_class("figma-volume-group");

    let volume = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume.set_draw_value(false);
    volume.set_digits(0);
    volume.set_value(8.0);
    volume.set_size_request(68, 38);
    volume.set_tooltip_text(Some("Громкость"));
    volume.add_css_class("figma-volume");
    volume_group.append(&volume);

    let volume_icon = picture_from_project_file(
        "Frame/assets/image-a7524bd04a5d9478.png",
        38,
        38,
    );
    volume_group.append(&volume_icon);
    action_base.append(&volume_group);

    let action_spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    action_spacer.set_hexpand(true);
    action_base.append(&action_spacer);

    let settings = gtk::Button::new();
    settings.add_css_class("figma-control");
    settings.set_size_request(31, 31);
    settings.set_tooltip_text(Some("Настройки"));
    settings.set_focusable(false);
    settings.set_margin_top(17);
    settings.set_valign(gtk::Align::Start);
    settings.set_child(Some(&picture_from_project_file(
        "Frame/assets/image-d81a3bd1d543f942.png",
        31,
        31,
    )));
    action_base.append(&settings);
    action_row.set_child(Some(&action_base));

    // Center the transport group on the window's visual center. Since the seek
    // area begins at x=167, a 40px end margin shifts the group's center 20px left
    // from the seek area's center, matching the exported 1200px composition.
    let transport = gtk::Box::new(gtk::Orientation::Horizontal, 15);
    transport.set_halign(gtk::Align::Center);
    transport.set_valign(gtk::Align::Start);
    transport.set_margin_top(7);
    transport.set_margin_end(122);
    transport.add_css_class("figma-transport");

    let previous = gtk::Button::new();
    previous.add_css_class("figma-control");
    previous.set_size_request(31, 31);
    previous.set_tooltip_text(Some("Предыдущий трек"));
    previous.set_focusable(false);
    previous.set_margin_top(4);
    previous.set_child(Some(&picture_from_project_file(
        "Frame/assets/image-ad9b5a6d06da7dec.png",
        31,
        31,
    )));
    transport.append(&previous);

    let play = gtk::Button::new();
    play.add_css_class("figma-control");
    play.set_size_request(43, 41);
    play.set_tooltip_text(Some("Воспроизвести / пауза"));
    play.set_focusable(false);
    let play_layers = gtk::Overlay::new();
    play_layers.set_size_request(43, 41);
    play_layers.set_child(Some(&picture_from_project_file(
        "Frame/assets/image-9e45c6515b00fe78.png",
        43,
        41,
    )));

    // Keep the exported play button background in both states. When playing,
    // cover its triangle and draw pause bars inside the same button frame.
    let pause_overlay = gtk::Fixed::new();
    pause_overlay.set_size_request(43, 41);

    let pause_mask = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    pause_mask.set_size_request(18, 23);
    pause_mask.add_css_class("figma-pause-mask");
    pause_overlay.put(&pause_mask, 13.0, 8.0);

    let pause_bar_left = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    pause_bar_left.set_size_request(4, 15);
    pause_bar_left.add_css_class("figma-pause-bar");
    pause_overlay.put(&pause_bar_left, 16.0, 13.0);

    let pause_bar_right = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    pause_bar_right.set_size_request(4, 15);
    pause_bar_right.add_css_class("figma-pause-bar");
    pause_overlay.put(&pause_bar_right, 24.0, 13.0);

    pause_overlay.set_visible(false);
    play_layers.add_overlay(&pause_overlay);
    play.set_child(Some(&play_layers));
    transport.append(&play);

    let next = gtk::Button::new();
    next.add_css_class("figma-control");
    next.set_size_request(31, 31);
    next.set_tooltip_text(Some("Следующий трек"));
    next.set_focusable(false);
    next.set_margin_top(4);
    next.set_child(Some(&picture_from_project_file(
        "Frame/assets/image-8c4553e8b0d341b7.png",
        31,
        31,
    )));
    transport.append(&next);

    action_row.add_overlay(&transport);
    controls.append(&action_row);
    main_row.append(&controls);
    dock.append(&main_row);
    root.put(&dock, 25.0, 548.0);

    let status = gtk::Label::new(None);
    status.set_halign(gtk::Align::Start);
    status.set_valign(gtk::Align::Start);
    status.set_xalign(0.0);
    status.set_yalign(0.0);
    status.set_ellipsize(gtk::pango::EllipsizeMode::End);
    status.set_size_request(900, 16);
    status.add_css_class("figma-status");
    status.set_visible(false);
    // Display diagnostic messages above the track title when an action fails.
    root.put(&status, 44.0, 527.0);

    let snapshot = Rc::new(RefCell::new(PlayerSnapshot::default()));
    let ui = UiWidgets {
        title,
        artist,
        cover,
        seek,
        volume,
        play,
        status,
        updating_seek: Rc::new(Cell::new(false)),
        updating_volume: Rc::new(Cell::new(false)),
        pause_overlay,
    };

    {
        let model = Rc::clone(&snapshot);
        let ui_for_callback = ui.clone();
        let play_button = ui.play.clone();
        play_button.connect_clicked(move |_| {
            run_player_action("PlayPause", &model, &ui_for_callback)
        });
    }
    {
        let model = Rc::clone(&snapshot);
        let ui = ui.clone();
        previous.connect_clicked(move |_| run_player_action("Previous", &model, &ui));
    }
    {
        let model = Rc::clone(&snapshot);
        let ui = ui.clone();
        next.connect_clicked(move |_| run_player_action("Next", &model, &ui));
    }
    {
        let model = Rc::clone(&snapshot);
        let ui_for_callback = ui.clone();
        let volume_control = ui.volume.clone();
        volume_control.connect_value_changed(move |scale| {
            if ui_for_callback.updating_volume.get() {
                return;
            }
            let percent = scale.value();
            if !percent.is_finite() || !(0.0..=100.0).contains(&percent) {
                return;
            }
            match player::set_volume(percent / 100.0) {
                Ok(()) => model.borrow_mut().volume = percent / 100.0,
                Err(error) => show_error(&ui_for_callback, &format!("Ошибка громкости: {error}")),
            }
        });
    }
    {
        let model = Rc::clone(&snapshot);
        let ui_for_callback = ui.clone();
        let seek_control = ui.seek.clone();
        seek_control.connect_value_changed(move |scale| {
            if ui_for_callback.updating_seek.get() {
                return;
            }
            let seconds = scale.value();
            if !seconds.is_finite() || seconds < 0.0 {
                return;
            }
            let (track_id, duration) = {
                let current = model.borrow();
                (current.track_id.clone(), current.duration)
            };
            let Some(track_id) = track_id else {
                return;
            };
            let seconds = seconds.min(duration.max(0.0));
            match player::set_position(&track_id, seconds) {
                Ok(()) => model.borrow_mut().position = seconds,
                Err(error) => show_error(&ui_for_callback, &format!("Ошибка перемотки: {error}")),
            }
        });
    }
    {
        let app = application.clone();
        settings.connect_clicked(move |_| {
            let window = crate::settings::build_window();
            window.set_application(Some(&app));
            window.present();
        });
    }

    refresh_player(&snapshot, &ui);
    let refresh_model = Rc::clone(&snapshot);
    let refresh_ui = ui.clone();
    gtk::glib::timeout_add_seconds_local(1, move || {
        refresh_player(&refresh_model, &refresh_ui);
        gtk::glib::ControlFlow::Continue
    });

    window
}

fn theme_config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    Some(base.join("murglar-controller").join("theme"))
}

fn save_theme(theme: &str) {
    let Some(path) = theme_config_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        if let Err(error) = std::fs::create_dir_all(parent).and_then(|_| std::fs::write(&path, theme)) {
            eprintln!("Не удалось сохранить тему: {error}");
        }
    }
}

fn load_saved_theme() -> String {
    theme_config_path()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|value| value.trim().to_owned())
        .filter(|value| {
            matches!(
                value.as_str(),
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("themes").join(file);
    let provider = gtk::CssProvider::new();
    match std::fs::read_to_string(path) {
        Ok(css) => provider.load_from_data(&css),
        Err(error) => {
            eprintln!("Не удалось загрузить файл темы {file}: {error}");
            return;
        }
    }

    if let Some(display) = gtk::gdk::Display::default() {
        THEME_PROVIDER.with(|slot| {
            if let Some(previous) = slot.borrow_mut().take() {
                gtk::style_context_remove_provider_for_display(&display, &previous);
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

pub fn run() {
    let application = gtk::Application::new(
        Some("ru.murglar.controller"),
        gtk::gio::ApplicationFlags::FLAGS_NONE,
    );
    application.connect_activate(|application| {
        let window = build_main_window(application);
        window.present();
    });
    let _ = application.run();
}
