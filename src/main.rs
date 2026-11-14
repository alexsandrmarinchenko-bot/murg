mod player;
mod ui;
mod updater;

slint::include_modules!();

fn main() {
    let arguments: Vec<_> = std::env::args_os().collect();
    if arguments
        .get(1)
        .is_some_and(|argument| argument.to_string_lossy() == "--apply-update")
    {
        let parent_pid = arguments
            .get(2)
            .and_then(|value| value.to_string_lossy().parse::<u32>().ok());
        let staged_path = arguments.get(3).map(std::path::PathBuf::from);

        let (Some(parent_pid), Some(staged_path)) = (parent_pid, staged_path) else {
            eprintln!("Некорректные аргументы помощника обновления PlayerDock");
            std::process::exit(2);
        };

        if let Err(error) = updater::apply_staged_update(parent_pid, &staged_path) {
            eprintln!("Не удалось применить обновление PlayerDock: {error}");
            std::process::exit(1);
        }
        return;
    }

    if let Err(error) = ui::run() {
        eprintln!("Не удалось запустить PlayerDock: {error}");
        std::process::exit(1);
    }
}
