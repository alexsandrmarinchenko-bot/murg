use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    thread,
    time::Duration,
};

use slint::{ComponentHandle, Image, SharedString, Timer, TimerMode};
use url::Url;

use crate::{
    player::{self, PlayerDescriptor},
    updater,
    Demo,
};

#[derive(Default)]
struct PlayerSnapshot {
    track_id: Option<String>,
    duration: f64,
    art_url: Option<String>,
}

#[derive(Default)]
struct PlayerSelection {
    available: Vec<PlayerDescriptor>,
    selected_service: Option<String>,
}

fn set_status(window: &slint::Weak<Demo>, message: impl Into<SharedString>) {
    if let Some(window) = window.upgrade() {
        window.set_status_message(message.into());
    }
}

fn art_image_from_url(value: &str) -> Option<Image> {
    // MPRIS may publish remote artwork. Until bounded remote caching is added,
    // only local file:// artwork is loaded.
    let parsed = Url::parse(value).ok()?;
    if parsed.scheme() != "file" {
        return None;
    }
    let path = parsed.to_file_path().ok()?;
    Image::load_from_path(&path).ok()
}

fn selected_service(selection: &Rc<RefCell<PlayerSelection>>) -> Option<String> {
    selection.borrow().selected_service.clone()
}

fn refresh_player(
    window: &slint::Weak<Demo>,
    selection: &Rc<RefCell<PlayerSelection>>,
    snapshot: &Rc<RefCell<PlayerSnapshot>>,
    seek_dragging: &Cell<bool>,
    volume_dragging: &Cell<bool>,
) {
    let Some(window) = window.upgrade() else {
        return;
    };

    let Some(service_name) = selected_service(selection) else {
        window.set_track_title("Запусти медиаплеер");
        window.set_artist_name("Ожидание MPRIS-плееров");
        window.set_is_playing(false);
        window.set_position_seconds(0.0);
        window.set_duration_seconds(1.0);
        window.set_volume_percent(0.0);
        window.set_cover_art(Image::default());
        window.set_status_message("Не найдено активных MPRIS-плееров.");
        let mut current = snapshot.borrow_mut();
        current.track_id = None;
        current.duration = 1.0;
        current.art_url = None;
        return;
    };

    let info = match player::read_status(&service_name) {
        Ok(info) => info,
        Err(error) => {
            window.set_status_message(
                format!("Не удалось прочитать состояние плеера: {error}").into(),
            );
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

fn refresh_discovery(
    window: &slint::Weak<Demo>,
    selection: &Rc<RefCell<PlayerSelection>>,
    snapshot: &Rc<RefCell<PlayerSnapshot>>,
    seek_dragging: &Cell<bool>,
    volume_dragging: &Cell<bool>,
) {
    let players = match player::list_players() {
        Ok(players) => players,
        Err(error) => {
            set_status(window, format!("Не удалось найти MPRIS-плееры: {error}"));
            return;
        }
    };

    let (selected_name, player_count) = {
        let mut current = selection.borrow_mut();
        let selected_is_available = current.selected_service.as_ref().is_some_and(|selected| {
            players
                .iter()
                .any(|player| &player.service_name == selected)
        });

        if !selected_is_available {
            current.selected_service = players.first().map(|player| player.service_name.clone());
        }

        current.available = players;

        let selected_name = current
            .available
            .iter()
            .find(|player| Some(&player.service_name) == current.selected_service.as_ref())
            .map(|player| player.display_name.clone())
            .unwrap_or_else(|| "Нет активного плеера".to_owned());

        (selected_name, current.available.len())
    };

    if let Some(window) = window.upgrade() {
        window.set_player_name(SharedString::from(selected_name));
        window.set_player_count(player_count as i32);
    }

    refresh_player(
        window,
        selection,
        snapshot,
        seek_dragging,
        volume_dragging,
    );
}

fn switch_player(
    window: &slint::Weak<Demo>,
    selection: &Rc<RefCell<PlayerSelection>>,
    snapshot: &Rc<RefCell<PlayerSnapshot>>,
    seek_dragging: &Cell<bool>,
    volume_dragging: &Cell<bool>,
) {
    {
        let mut current = selection.borrow_mut();
        if current.available.len() < 2 {
            return;
        }

        let current_index = current
            .available
            .iter()
            .position(|player| Some(&player.service_name) == current.selected_service.as_ref())
            .unwrap_or(0);
        let next_index = (current_index + 1) % current.available.len();
        current.selected_service =
            Some(current.available[next_index].service_name.clone());

        if let Some(window) = window.upgrade() {
            window.set_player_name(SharedString::from(
                current.available[next_index].display_name.clone(),
            ));
        }
    }

    refresh_player(
        window,
        selection,
        snapshot,
        seek_dragging,
        volume_dragging,
    );
}

fn run_player_action(
    method: &str,
    window: &slint::Weak<Demo>,
    selection: &Rc<RefCell<PlayerSelection>>,
    snapshot: &Rc<RefCell<PlayerSnapshot>>,
    seek_dragging: &Cell<bool>,
    volume_dragging: &Cell<bool>,
) {
    let Some(service_name) = selected_service(selection) else {
        set_status(window, "Нет активного плеера для управления.");
        return;
    };

    if let Err(error) = player::call_player(&service_name, method) {
        set_status(window, format!("Ошибка управления плеером: {error}"));
        return;
    }

    refresh_player(
        window,
        selection,
        snapshot,
        seek_dragging,
        volume_dragging,
    );
}

fn check_for_updates(window: &slint::Weak<Demo>) {
    let window = window.clone();

    thread::spawn(move || match updater::check_for_update() {
        Ok(Some(update)) => {
            let version = update.version;
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(window) = window.upgrade() {
                    window.set_update_version(SharedString::from(version));
                    window.set_update_available(true);
                    window.set_update_message(SharedString::default());
                }
            });
        }
        Ok(None) => {
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(window) = window.upgrade() {
                    window.set_update_available(false);
                    window.set_update_version(SharedString::default());
                }
            });
        }
        Err(error) => {
            eprintln!("Проверка обновлений PlayerDock не удалась: {error}");
        }
    });
}

pub fn run() -> Result<(), slint::PlatformError> {
    let window = Demo::new()?;
    let window_weak = window.as_weak();
    let snapshot = Rc::new(RefCell::new(PlayerSnapshot::default()));
    let selection = Rc::new(RefCell::new(PlayerSelection::default()));
    let seek_dragging = Rc::new(Cell::new(false));
    let volume_dragging = Rc::new(Cell::new(false));

    {
        let weak = window_weak.clone();
        window.on_library_clicked(move || {
            set_status(&weak, "Библиотека пока не подключена.");
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
        let players = Rc::clone(&selection);
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        window.on_player_switch_clicked(move || {
            switch_player(&weak, &players, &state, &seeking, &adjusting_volume);
        });
    }

    {
        let weak = window_weak.clone();
        let players = Rc::clone(&selection);
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        window.on_previous_clicked(move || {
            run_player_action("Previous", &weak, &players, &state, &seeking, &adjusting_volume);
        });
    }

    {
        let weak = window_weak.clone();
        let players = Rc::clone(&selection);
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        window.on_play_clicked(move || {
            run_player_action("PlayPause", &weak, &players, &state, &seeking, &adjusting_volume);
        });
    }

    {
        let weak = window_weak.clone();
        let players = Rc::clone(&selection);
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        window.on_next_clicked(move || {
            run_player_action("Next", &weak, &players, &state, &seeking, &adjusting_volume);
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
                if window.get_status_message().is_empty() {
                    window.set_status_message(SharedString::default());
                }
            }
        });
    }
    {
        let weak = window_weak.clone();
        let players = Rc::clone(&selection);
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        window.on_seek_committed(move |seconds: f32| {
            let (service_name, track_id, duration) = {
                let current = players.borrow();
                let service_name = current.selected_service.clone();
                let track_id = state.borrow().track_id.clone();
                let duration = state.borrow().duration;
                (service_name, track_id, duration)
            };

            let result = match (service_name, track_id) {
                (Some(service_name), Some(track_id)) => {
                    let target = (seconds as f64).clamp(0.0, duration.max(0.0));
                    player::set_position(&service_name, &track_id, target)
                }
                (None, _) => Err("Нет активного плеера для перемотки".to_owned()),
                (_, None) => Err("Плеер не сообщил идентификатор текущего трека".to_owned()),
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

            refresh_player(
                &weak,
                &players,
                &state,
                &seeking,
                &adjusting_volume,
            );
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
        let players = Rc::clone(&selection);
        let adjusting = Rc::clone(&volume_dragging);
        window.on_volume_changed(move |percent: f32| {
            if !percent.is_finite() {
                set_status(&weak, "Некорректное значение громкости.");
                return;
            }

            let Some(service_name) = selected_service(&players) else {
                set_status(&weak, "Нет активного плеера для изменения громкости.");
                adjusting.set(false);
                return;
            };

            let normalized = (percent as f64 / 100.0).clamp(0.0, 1.0);
            match player::set_volume(&service_name, normalized) {
                Ok(()) => set_status(&weak, SharedString::default()),
                Err(error) => set_status(&weak, format!("Ошибка громкости: {error}")),
            }
            adjusting.set(false);
        });
    }

    {
        let weak = window_weak.clone();
        window.on_update_clicked(move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            if window.get_update_busy() {
                return;
            }

            window.set_update_busy(true);
            window.set_update_message("Скачивание и проверка обновления...");

            let weak = weak.clone();
            thread::spawn(move || match updater::download_and_launch_update() {
                Ok(version) => {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(window) = weak.upgrade() {
                            window.set_update_message(
                                format!("Установка {version}. Перезапуск...").into(),
                            );
                        }
                        slint::quit_event_loop();
                    });
                }
                Err(error) => {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(window) = weak.upgrade() {
                            window.set_update_busy(false);
                            window.set_update_message(
                                format!("Ошибка обновления: {error}").into(),
                            );
                        }
                    });
                }
            });
        });
    }

    refresh_discovery(
        &window_weak,
        &selection,
        &snapshot,
        &seek_dragging,
        &volume_dragging,
    );

    // Playback state is refreshed every second; player discovery is less frequent.
    let player_timer = Timer::default();
    {
        let weak = window_weak.clone();
        let players = Rc::clone(&selection);
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        player_timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            refresh_player(&weak, &players, &state, &seeking, &adjusting_volume);
        });
    }

    let discovery_timer = Timer::default();
    {
        let weak = window_weak.clone();
        let players = Rc::clone(&selection);
        let state = Rc::clone(&snapshot);
        let seeking = Rc::clone(&seek_dragging);
        let adjusting_volume = Rc::clone(&volume_dragging);
        discovery_timer.start(TimerMode::Repeated, Duration::from_secs(5), move || {
            refresh_discovery(&weak, &players, &state, &seeking, &adjusting_volume);
        });
    }

    check_for_updates(&window_weak);
    let update_timer = Timer::default();
    {
        let weak = window_weak.clone();
        update_timer.start(
            TimerMode::Repeated,
            Duration::from_secs(6 * 60 * 60),
            move || check_for_updates(&weak),
        );
    }

    window.run()
}
