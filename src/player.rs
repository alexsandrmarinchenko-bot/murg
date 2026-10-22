use std::collections::HashMap;

use zbus::{
    blocking::{Connection, Proxy},
    zvariant::{OwnedObjectPath, OwnedValue},
};

const SERVICE: &str = "org.mpris.MediaPlayer2.com.badmanners.murglar";
const PATH: &str = "/org/mpris/MediaPlayer2";
const INTERFACE: &str = "org.mpris.MediaPlayer2.Player";
const MICROSECONDS: f64 = 1_000_000.0;

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
    Connection::session().map_err(|e| e.to_string())
}

pub fn call_player(method: &str) -> Result<(), String> {
    let connection = connect()?;
    let player = Proxy::new(&connection, SERVICE, PATH, INTERFACE)
        .map_err(|e| e.to_string())?;

    player
        .call_method(method, &())
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn set_volume(volume: f64) -> Result<(), String> {
    if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
        return Err("Громкость должна быть от 0 до 1".to_owned());
    }

    let connection = connect()?;
    let player = Proxy::new(&connection, SERVICE, PATH, INTERFACE)
        .map_err(|e| e.to_string())?;

    player
        .set_property("Volume", &volume)
        .map_err(|e| e.to_string())
}

pub fn set_position(track_id: &str, seconds: f64) -> Result<(), String> {
    if !seconds.is_finite() || seconds < 0.0 {
        return Err("Некорректная позиция трека".to_owned());
    }

    let track_id = OwnedObjectPath::try_from(track_id.to_owned())
        .map_err(|e| e.to_string())?;

    let position = (seconds * MICROSECONDS) as i64;

    let connection = connect()?;
    let player = Proxy::new(&connection, SERVICE, PATH, INTERFACE)
        .map_err(|e| e.to_string())?;

    player
        .call_method("SetPosition", &(track_id, position))
        .map(|_| ())
        .map_err(|e| e.to_string())
}

pub fn read_status() -> Result<PlayerInfo, String> {
    let connection = connect()?;
    let player = Proxy::new(&connection, SERVICE, PATH, INTERFACE)
        .map_err(|e| e.to_string())?;

    let status: String = player
        .get_property("PlaybackStatus")
        .map_err(|e| e.to_string())?;

    let metadata: HashMap<String, OwnedValue> = player
        .get_property("Metadata")
        .map_err(|e| e.to_string())?;

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

    let position: i64 = player
        .get_property("Position")
        .map_err(|e| e.to_string())?;

    let volume: f64 = player
        .get_property("Volume")
        .map_err(|e| e.to_string())?;

    Ok(PlayerInfo {
        title,
        artist,
        status,
        volume,
        position: (position.max(0) as f64 / MICROSECONDS).min(duration),
        duration,
        track_id,
        art_url,
    })
}
