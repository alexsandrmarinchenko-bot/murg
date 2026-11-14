# PlayerDock

**PlayerDock** — настольная панель управления воспроизведением для Linux. Цель проекта — единый интерфейс для разных музыкальных и медиаплееров, а не отдельный клиент одного сервиса.

> **Текущее состояние:** исходная интеграция с Murglar работает через MPRIS, но обнаружение и переключение между несколькими плеерами ещё предстоит реализовать. Поддержка остальных приложений описана в [плане развития](docs/ROADMAP.md); README не обещает того, чего в текущей сборке пока нет.

## Что уже работает

- Play/Pause, предыдущий и следующий трек для текущей интеграции Murglar.
- Перемотка и управление громкостью.
- Отображение названия трека, исполнителя и локальной обложки, если плеер отдаёт её через `file://`.
- Изменяемое по размеру окно и анимированные состояния наведения/нажатия у кнопок библиотеки и настроек.

Кнопки **Библиотека** и **Настройки** пока являются заглушками. Сейчас код использует фиксированное D-Bus имя Murglar; универсальное обнаружение MPRIS-плееров — первый этап переработки.

## Технологии

- Rust stable, edition 2024.
- Slint 1.18.1 — графический интерфейс.
- zbus 5 — подключение к пользовательской D-Bus-сессии и MPRIS.
- Cargo — зависимости и сборка.

## Сборка и запуск на Arch Linux

Нужны Rust stable, Cargo, системные библиотеки Slint и работающая текущая интеграция Murglar с включённым MPRIS.

При необходимости установи системные зависимости:

```bash
sudo pacman -S --needed base-devel pkgconf libx11 libxkbcommon wayland mesa fontconfig freetype2
```

Исходники, Cargo-кэш и результаты сборки размещаются на отдельном разделе `/mnt/Storage`:

```bash
git clone https://github.com/alexsandrmarinchenko-bot/murg.git /mnt/Storage/playerdock
cd /mnt/Storage/playerdock

mkdir -p /mnt/Storage/CargoHome/playerdock /mnt/Storage/CargoTarget/playerdock
CARGO_HOME=/mnt/Storage/CargoHome/playerdock \
CARGO_TARGET_DIR=/mnt/Storage/CargoTarget/playerdock \
cargo run --locked
```

Проверка сборки без запуска окна:

```bash
CARGO_HOME=/mnt/Storage/CargoHome/playerdock \
CARGO_TARGET_DIR=/mnt/Storage/CargoTarget/playerdock \
cargo check --locked
```

## Структура проекта сейчас

```text
.
├── .github/workflows/build.yml  # CI: проверка сборки
├── Frame/
│   ├── main.slint               # Рабочий интерфейс Slint
│   └── assets/                  # Используемые изображения и иконки
├── src/
│   ├── main.rs                  # Точка входа
│   ├── ui.rs                    # Связь Slint-интерфейса с текущей интеграцией
│   └── player.rs                # MPRIS-команды; пока D-Bus имя Murglar задано явно
├── docs/
│   ├── ROADMAP.md               # Архитектура и этапы универсализации
│   ├── figma-export.md          # Как обновлять интерфейс из Figma
│   └── figma-exports/           # Исторические экспорты для сравнения/восстановления
├── build.rs                     # Компиляция Slint
├── Cargo.toml
└── Cargo.lock
```

## Обновление

```bash
cd /mnt/Storage/playerdock
git pull --ff-only origin main

CARGO_HOME=/mnt/Storage/CargoHome/playerdock \
CARGO_TARGET_DIR=/mnt/Storage/CargoTarget/playerdock \
cargo run --locked
```

Экспорт из Figma и порядок обновления UI описаны в [docs/figma-export.md](docs/figma-export.md).

## Шрифт

Файл Inter не включён в репозиторий; текст использует доступный системный шрифт-замену. Поэтому рендеринг текста может отличаться от Figma. При добавлении разрешённого файла Inter его можно подключить импортом в `Frame/main.slint`.

## Название репозитория

Имя приложения и Cargo-пакета изменено на PlayerDock. Существующий URL GitHub пока остаётся `alexsandrmarinchenko-bot/murg`: переименование самого GitHub-репозитория требует отдельного действия в настройках GitHub.
