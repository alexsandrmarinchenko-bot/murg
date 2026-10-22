use std::{collections::HashMap, env, error::Error};

use zbus::{
    blocking::{Connection, Proxy},
    zvariant::OwnedValue,
};

const MURGLAR_SERVICE: &str = "org.mpris.MediaPlayer2.com.badmanners.murglar";
const MPRIS_PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER_INTERFACE: &str = "org.mpris.MediaPlayer2.Player";

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "status".to_owned());

    let connection = Connection::session()?;
    let player = Proxy::new(
        &connection,
        MURGLAR_SERVICE,
        MPRIS_PATH,
        PLAYER_INTERFACE,
    )?;

    match command.as_str() {
        "status" => print_status(&player)?,
        "toggle" => {
            player.call_method("PlayPause", &())?;
            println!("Отправлена команда переключения воспроизведения.");
        }
        "play" => {
            player.call_method("Play", &())?;
            println!("Отправлена команда воспроизведения.");
        }
        "pause" => {
            player.call_method("Pause", &())?;
            println!("Отправлена команда паузы.");
        }
        "next" => {
            player.call_method("Next", &())?;
            println!("Отправлена команда следующего трека.");
        }
        "previous" => {
            player.call_method("Previous", &())?;
            println!("Отправлена команда предыдущего трека.");
        }
        "volume" => print_volume(&player)?,
        "set-volume" => {
            let raw_value = args
                .next()
                .ok_or("Укажи громкость от 0 до 100, например: set-volume 20")?;

            let percent: f64 = raw_value.parse()?;

            if !percent.is_finite() || !(0.0..=100.0).contains(&percent) {
                return Err("Громкость должна быть числом от 0 до 100".into());
            }

            player.set_property("Volume", &(percent / 100.0))?;
            println!("Запрошена громкость: {percent:.0}%");
        }
        _ => {
            eprintln!(
                "Команды: status, toggle, play, pause, next, previous, volume, set-volume 20"
            );
            return Err("Неизвестная команда".into());
        }
    }

    Ok(())
}

fn print_status(player: &Proxy<'_>) -> Result<(), Box<dyn Error>> {
    let status: String = player.get_property("PlaybackStatus")?;
    let metadata: HashMap<String, OwnedValue> = player.get_property("Metadata")?;

    let title = metadata
        .get("xesam:title")
        .and_then(|value| String::try_from(value.clone()).ok())
        .unwrap_or_else(|| "Неизвестный трек".to_owned());

    let artist = metadata
        .get("xesam:artist")
        .and_then(|value| {
            Vec::<String>::try_from(value.clone())
                .ok()
                .map(|artists| artists.join(", "))
        })
        .unwrap_or_else(|| "Неизвестный исполнитель".to_owned());

    println!("Murglar Controller");
    println!("------------------");
    println!("Состояние: {status}");
    println!("Трек:      {title}");
    println!("Исполнитель: {artist}");

    Ok(())
}

fn print_volume(player: &Proxy<'_>) -> Result<(), Box<dyn Error>> {
    let volume: f64 = player.get_property("Volume")?;
    println!("Громкость: {:.0}%", volume * 100.0);
    Ok(())
}
