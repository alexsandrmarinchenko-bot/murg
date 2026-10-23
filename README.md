# Murglar Controller

Небольшой настольный пульт для **Murglar** на Linux. Интерфейс сделан на Slint, управление плеером — через MPRIS по пользовательской D-Bus-сессии.

## Возможности

- Play/Pause, предыдущий и следующий трек.
- Перемотка и управление громкостью.
- Отображение названия трека, исполнителя и локальной обложки, если Murglar отдаёт её через `file://`.
- Изменяемое по размеру окно и анимированные состояния наведения/нажатия у кнопок библиотеки и настроек.

Кнопки **Библиотека** и **Настройки** пока являются заглушками: их основные экраны ещё не подключены.

## Сборка и запуск

Нужны Rust stable (edition 2024), Cargo, системные библиотеки Slint и работающий Murglar с включённым MPRIS.

На Arch Linux необходимые системные пакеты Slint можно установить командой ниже, если они ещё не установлены:

```bash
sudo pacman -S --needed base-devel pkgconf libx11 libxkbcommon wayland mesa fontconfig freetype2
```

Клонирование проекта и запуск. Пример хранит исходники, кэш Cargo и результаты сборки на отдельном разделе `/mnt/Storage`:

```bash
git clone https://github.com/alexsandrmarinchenko-bot/murg.git /mnt/Storage/murg
cd /mnt/Storage/murg

mkdir -p /mnt/Storage/CargoHome/murg /mnt/Storage/CargoTarget/murg
CARGO_HOME=/mnt/Storage/CargoHome/murg \
CARGO_TARGET_DIR=/mnt/Storage/CargoTarget/murg \
cargo run --locked
```

Проверить сборку без запуска окна:

```bash
CARGO_HOME=/mnt/Storage/CargoHome/murg \
CARGO_TARGET_DIR=/mnt/Storage/CargoTarget/murg \
cargo check --locked
```

## Структура проекта

```text
.
├── .github/workflows/build.yml  # CI: проверка сборки
├── Frame/
│   ├── main.slint               # Интерфейс и состояния кнопок
│   └── assets/                  # Изображения и SVG-иконки
├── src/
│   ├── main.rs                  # Точка входа
│   ├── ui.rs                    # Связь Slint-интерфейса с плеером
│   └── player.rs                # MPRIS-команды и чтение метаданных
├── docs/
│   ├── figma-export.md          # Как обновлять интерфейс из Figma
│   └── figma-exports/           # Архивные экспорты, не участвуют в сборке
├── build.rs                     # Компиляция Slint
├── Cargo.toml
└── Cargo.lock
```

## Обновление

```bash
cd /mnt/Storage/murg
git pull --ff-only origin main
CARGO_HOME=/mnt/Storage/CargoHome/murg \
CARGO_TARGET_DIR=/mnt/Storage/CargoTarget/murg \
cargo run --locked
```

Экспорт из Figma и порядок обновления UI описаны в [docs/figma-export.md](docs/figma-export.md).

## Шрифт

Файл Inter не включён в репозиторий; текст использует доступный шрифт-замену. Поэтому рендеринг текста может отличаться от Figma. При добавлении разрешённого файла Inter можно подключить его импортом в `Frame/main.slint`.
