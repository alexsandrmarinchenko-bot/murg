mod player;
mod ui;

slint::include_modules!();

fn main() {
    if let Err(error) = ui::run() {
        eprintln!("Не удалось запустить Murglar Controller: {error}");
        std::process::exit(1);
    }
}
