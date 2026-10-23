use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use slint::{ComponentHandle, Image, SharedString, Timer, TimerMode};
use url::Url;

use crate::player;

use crate::Demo;

#[derive(Default)]
struct PlayerSnapshot {
    track_id: Option<String>,
    duration: f64,
    art_url: Option<String>,
}

fn set_status(window: &slint::Weak<Demo>, message: impl Into<SharedString>) {
    if let Some(window) = window.upgrade() {
        window.set_status_message(message.into());
    }
}

fn art_image_from_url(value: &str) -> Option<Image> {
    // Murglar normally publishes a local file:// art URL through MPRIS.
    // Remote URLs are deliberately not fetched implicitly.
    let parsed = Url::parse(value).ok()?;
    if parsed.scheme() != "file" {
        return None;
    }
    let path = parsed.to_file_path().ok()?;
    Image::load_from_path(&path).ok()
}

fn refresh_player(
    window: &slint::Weak<Demo>,
    snapshot: &Rc<RefCell<PlayerSnapshot>>,
    seek_dragging: &Cell<bool>,
    volume_dragging: &Cell<bool>,
) {
    let Some(window) = window.upgrade() else {
        return;
    };

    let info = match player::read_status() {
        Ok(info) => info,
        Err(error) => {
            window.set_status_message(format!("Нет связи с Murglar через MPRIS: {error}").into());
            return;
        }
    };

    let safe_duration = if info.duration.is_finite() {
        info.duration.max(1.0)
    } else {
        1.0
    };
    let safe_position = if info.position.is_finite() {
        info.position.clamp(0.0, safe_duration)
    } else {
        0.0
    };
    let safe_volume = if info.volume.is_finite() {
        info.volume.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let art_changed = {
        let mut current = snapshot.borrow_mut();
        current.track_id = info.track_id;
        current.duration = safe_duration;
        let changed = current.art_url != info.art_url;
        current.art_url = info.art_url.clone();
        changed
    };

    window.set_track_title(SharedString::from(info.title));
    window.set_artist_name(SharedString::from(if info.artist.trim().is_empty() {
        "Неизвестный исполнитель".to_owned()
    } else {
        info.artist
    }));
    window.set_duration_seconds(safe_duration as f32);
    window.set_is_playing(info.status == "Playing");

    if !seek_dragging.get() {
        window.set_position_seconds(safe_position as f32);
    }
    if !volume_dragging.get() {
        window.set_volume_percent((safe_volume * 100.0) as f32);
    }

    if art_changed {
        if let Some(image) = info.art_url.as_deref().and_then(art_image_from_url) {
            window.set_cover_art(image);
        } else {
            window.set_cover_art(Image::default());
        }
    }

    window.set_status_message(SharedString::default());
}

fn run_player_action(
    method: &str,
    window: &slint::Weak<Demo>,
    snapshot: &Rc<RefCell<PlayerSnapshot>>,
    seek_dragging: &Cell<bool>,
    volume_dragging: &Cell<bool>,
) {
    if let Err(error) = player::call_player(method) {
        set_status(window, format!("Ошибка управления плеером: {error}"));
        return;
    }
    refresh_player(window, snapshot, seek_dragging, volume_dragging);
}

pub fn run() -> Result<(), slint::PlatformError> {
    let window = Demo::new()?;
    let window_weak = window.as_weak();
    let snapshot = Rc::new(RefCell::new(PlayerSnapshot::default()));
    let seek_dragging = Rc::new(Cell::new(false));
    let volume_dragging = Rc::new(Cell::new(false));

    {
        let weak = window_weak.clone();
        window.on_library_clicked(move || {
            set_status(
                &weak,
                "Библиотека пока не подключена к источнику данных Murglar.",
            );
        });
    }

    {
        let weak = window_weak.clone();
        window.on_settings_clicked(move || {
            set_status(&weak, "Настройки пока не подключены.");
        });
    }

    {
        let weak = window_weak.clone();
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        window.on_previous_clicked(move || {
            run_player_action("Previous", &weak, &state, &seeking, &adjusting_volume);
        });
    }

    {
        let weak = window_weak.clone();
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        window.on_play_clicked(move || {
            run_player_action("PlayPause", &weak, &state, &seeking, &adjusting_volume);
        });
    }

    {
        let weak = window_weak.clone();
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        window.on_next_clicked(move || {
            run_player_action("Next", &weak, &state, &seeking, &adjusting_volume);
        });
    }

    {
        let seeking = Rc::clone(&seek_dragging);
        window.on_seek_drag_started(move || seeking.set(true));
    }
    {
        let seeking = Rc::clone(&seek_dragging);
        let weak = window_weak.clone();
        window.on_seek_drag_cancelled(move || {
            seeking.set(false);
            if let Some(window) = weak.upgrade() {
                let message = window.get_status_message();
                if message.is_empty() {
                    window.set_status_message(SharedString::default());
                }
            }
        });
    }
    {
        let weak = window_weak.clone();
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        window.on_seek_committed(move |seconds: f32| {
            let (track_id, duration) = {
                let current = state.borrow();
                (current.track_id.clone(), current.duration)
            };
            let result = if let Some(track_id) = track_id {
                let target = (seconds as f64).clamp(0.0, duration.max(0.0));
                player::set_position(&track_id, target)
            } else {
                Err("Murglar не сообщил идентификатор текущего трека".to_owned())
            };
            match result {
                Ok(()) => {
                    seeking.set(false);
                    set_status(&weak, SharedString::default());
                    if let Some(window) = weak.upgrade() {
                        window.set_position_seconds(seconds.max(0.0));
                    }
                }
                Err(error) => {
                    seeking.set(false);
                    set_status(&weak, format!("Ошибка перемотки: {error}"));
                }
            }
        });
    }

    {
        let adjusting = Rc::clone(&volume_dragging);
        window.on_volume_drag_started(move || adjusting.set(true));
    }
    {
        let adjusting = Rc::clone(&volume_dragging);
        window.on_volume_drag_finished(move || adjusting.set(false));
    }
    {
        let weak = window_weak.clone();
        let adjusting = Rc::clone(&volume_dragging);
        window.on_volume_changed(move |percent: f32| {
            if !percent.is_finite() {
                set_status(&weak, "Некорректное значение громкости.");
                return;
            }
            let normalized = (percent as f64 / 100.0).clamp(0.0, 1.0);
            match player::set_volume(normalized) {
                Ok(()) => set_status(&weak, SharedString::default()),
                Err(error) => set_status(&weak, format!("Ошибка громкости: {error}")),
            }
            adjusting.set(false);
        });
    }

<<<<<<< HEAD
=======

>>>>>>> 4524bb6 (23 окт. 2026 г., 22:18)
    refresh_player(
        &window_weak,
        &snapshot,
        &seek_dragging,
        &volume_dragging,
    );

    // Keep the polling timer alive until the main event loop exits.
    let timer = Timer::default();
    {
        let weak = window_weak.clone();
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting = Rc::clone(&volume_dragging);
        timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            refresh_player(&weak, &state, &seeking, &adjusting);
        });
    }

    window.run()
}
