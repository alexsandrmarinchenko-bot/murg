use std::collections::HashMap;

use zbus::{
    blocking::{fdo::DBusProxy, Connection, Proxy},
    zvariant::{OwnedObjectPath, OwnedValue},
};

const MPRIS_PREFIX: &str = "org.mpris.MediaPlayer2.";
const PATH: &str = "/org/mpris/MediaPlayer2";
const ROOT_INTERFACE: &str = "org.mpris.MediaPlayer2";
const PLAYER_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";
const MICROSECONDS: f64 = 1_000_000.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerDescriptor {
    pub service_name: String,
    pub display_name: String,
}

pub struct PlayerInfo {
    pub title: String,
    pub artist: String,
    pub status: String,
    pub volume: f64,
    pub position: f64,
    pub duration: f64,
    pub track_id: Option<String>,
    pub art_url: Option<String>,
}

fn connect() -> Result<Connection, String> {
    Connection::session().map_err(|error| format!("D-Bus session недоступна: {error}"))
}

/// Finds every currently-owned MPRIS service on the user's session bus.
pub fn list_players() -> Result<Vec<PlayerDescriptor>, String> {
    let connection = connect()?;
    let bus = DBusProxy::new(&connection).map_err(|error| error.to_string())?;
    let names = bus.list_names().map_err(|error| error.to_string())?;

    let mut players = Vec::new();

    for name in names {
        let service_name = name.to_string();
        if !service_name.starts_with(MPRIS_PREFIX) {
            continue;
        }

        let display_name = read_identity(&connection, &service_name)
            .unwrap_or_else(|| display_name_from_service(&service_name));

        players.push(PlayerDescriptor {
            service_name,
            display_name,
        });
    }

    players.sort_by(|left, right| {
        left.display_name
            .to_lowercase()
            .cmp(&right.display_name.to_lowercase())
            .then_with(|| left.service_name.cmp(&right.service_name))
    });
    players.dedup_by(|left, right| left.service_name == right.service_name);

    Ok(players)
}

fn read_identity(connection: &Connection, service_name: &str) -> Option<String> {
    let root = Proxy::new(connection, service_name, PATH, ROOT_INTERFACE).ok()?;
    let identity: String = root.get_property("Identity").ok()?;
    let identity = identity.trim();
    (!identity.is_empty()).then(|| identity.to_owned())
}

fn display_name_from_service(service_name: &str) -> String {
    service_name
        .strip_prefix(MPRIS_PREFIX)
        .unwrap_or(service_name)
        .rsplit('.')
        .next()
        .unwrap_or(service_name)
        .replace('-', " ")
        .replace('_', " ")
}

fn player_proxy<'a>(
    connection: &'a Connection,
    service_name: &'a str,
) -> Result<Proxy<'a>, String> {
    if !service_name.starts_with(MPRIS_PREFIX) {
        return Err("Некорректное имя MPRIS-сервиса".to_owned());
    }

    Proxy::new(connection, service_name, PATH, PLAYER_INTERFACE)
        .map_err(|error| error.to_string())
}

pub fn call_player(service_name: &str, method: &str) -> Result<(), String> {
    let allowed_methods = ["PlayPause", "Play", "Pause", "Stop", "Next", "Previous"];
    if !allowed_methods.contains(&method) {
        return Err("Неподдерживаемая команда управления".to_owned());
    }

    let connection = connect()?;
    let player = player_proxy(&connection, service_name)?;

    player
        .call_method(method, &())
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub fn set_volume(service_name: &str, volume: f64) -> Result<(), String> {
    if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
        return Err("Громкость должна быть от 0 до 1".to_owned());
    }

    let connection = connect()?;
    let player = player_proxy(&connection, service_name)?;

    player
        .set_property("Volume", &volume)
        .map_err(|error| error.to_string())
}

pub fn set_position(
    service_name: &str,
    track_id: &str,
    seconds: f64,
) -> Result<(), String> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err("Некорректная позиция трека".to_owned());
    }

    let track_id = OwnedObjectPath::try_from(track_id.to_owned())
        .map_err(|error| error.to_string())?;
    let position = (seconds * MICROSECONDS) as i64;

    let connection = connect()?;
    let player = player_proxy(&connection, service_name)?;

    player
        .call_method("SetPosition", &(track_id, position))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub fn read_status(service_name: &str) -> Result<PlayerInfo, String> {
    let connection = connect()?;
    let player = player_proxy(&connection, service_name)?;

    let status: String = player
        .get_property("PlaybackStatus")
        .map_err(|error| error.to_string())?;

    let metadata: HashMap<String, OwnedValue> = player
        .get_property("Metadata")
        .map_err(|error| error.to_string())?;

    let title = metadata
        .get("xesam:title")
        .and_then(|value| String::try_from(value.clone()).ok())
        .unwrap_or_else(|| "Нет данных о треке".to_owned());

    let artist = metadata
        .get("xesam:artist")
        .and_then(|value| {
            Vec::<String>::try_from(value.clone())
                .ok()
                .map(|artists| artists.join(", "))
        })
        .unwrap_or_default();

    let duration = metadata
        .get("mpris:length")
        .and_then(|value| i64::try_from(value.clone()).ok())
        .unwrap_or(0)
        .max(0) as f64
        / MICROSECONDS;

    let track_id = metadata
        .get("mpris:trackid")
        .and_then(|value| OwnedObjectPath::try_from(value.clone()).ok())
        .map(|path| path.to_string());

    let art_url = metadata
        .get("mpris:artUrl")
        .and_then(|value| String::try_from(value.clone()).ok());

    // Some MPRIS implementations omit optional properties. Missing values
    // must not make the whole player disappear from the controller.
    let position: i64 = player.get_property("Position").unwrap_or(0);
    let volume: f64 = player.get_property("Volume").unwrap_or(0.0);

    Ok(PlayerInfo {
        title,
        artist,
        status,
        volume: if volume.is_finite() {
            volume.clamp(0.0, 1.0)
        } else {
            0.0
        },
        position: (position.max(0) as f64 / MICROSECONDS).min(duration),
        duration,
        track_id,
        art_url,
    })
}
