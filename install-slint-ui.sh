#!/usr/bin/env bash
set -Eeuo pipefail

PROJECT_DIR="${MURGLAR_PROJECT_DIR:-/mnt/Storage/murglar-controller}"
BACKUP_ROOT="/mnt/Storage/murglar-controller-backups"
RAW_URL="https://raw.githubusercontent.com/alexsandrmarinchenko-bot/murg/fix-slint-export-1280x720/src/ui.rs"
TARGET_FILE="$PROJECT_DIR/src/ui.rs"
TMP_FILE=""

cleanup() {
    if [[ -n "${TMP_FILE:-}" ]]; then
        rm -f -- "$TMP_FILE"
    fi
}
trap cleanup EXIT

die() {
    printf 'Ошибка: %s\n' "$*" >&2
    exit 1
}

[[ -d "$PROJECT_DIR" ]] || die "Не найден проект: $PROJECT_DIR"
[[ -f "$PROJECT_DIR/Cargo.toml" ]] || die "Не найден Cargo.toml"
[[ -f "$TARGET_FILE" ]] || die "Не найден src/ui.rs"
[[ -f "$PROJECT_DIR/Frame/main.slint" ]] || die "Не найден Frame/main.slint. Убедись, что последний Figma-экспорт уже синхронизирован."
for asset in \
    image-d81a3bd1d543f942.png \
    image-99c1e7fe16e3ea5d.png \
    image-a7524bd04a5d9478.png \
    image-9e45c6515b00fe78.png \
    image-ad9b5a6d06da7dec.png \
    image-8c4553e8b0d341b7.png \
    image-cfb10b8456555fd5.png
do
    [[ -s "$PROJECT_DIR/Frame/assets/$asset" ]] || die "Не найден ресурс Frame/assets/$asset"
done
command -v curl >/dev/null 2>&1 || die "Установи curl."
command -v cargo >/dev/null 2>&1 || die "Не найден cargo."

mkdir -p -- "$BACKUP_ROOT"
TIMESTAMP="$(date +%Y%m%d-%H%M%S)"
BACKUP_FILE="$BACKUP_ROOT/ui.rs.$TIMESTAMP.bak"
TMP_FILE="$(mktemp "$PROJECT_DIR/src/.ui.rs.download.XXXXXX")"

printf 'Скачиваю интерфейс, подогнанный под Frame/main.slint...\n'
curl --fail --location --silent --show-error --retry 3 --connect-timeout 15 \
    "$RAW_URL" --output "$TMP_FILE"
grep -Fq 'set_default_size(1280, 720)' "$TMP_FILE" || die "Проверка версии не пройдена."
grep -Fq '"Frame/assets/' "$TMP_FILE" || die "В коде нет путей к новым ассетам."
if grep -Fq 'Frame-1/assets/' "$TMP_FILE"; then
    die "В скачанном коде обнаружены устаревшие пути Frame-1."
fi
if grep -Fq 'image-1cd20f420cbc147e.png' "$TMP_FILE"; then
    die "Код ссылается на отсутствующий бегунок."
fi

cp -a -- "$TARGET_FILE" "$BACKUP_FILE"
chmod --reference="$TARGET_FILE" "$TMP_FILE" 2>/dev/null || true
mv -f -- "$TMP_FILE" "$TARGET_FILE"
TMP_FILE=""

export CARGO_HOME="${CARGO_HOME:-/mnt/Storage/Cargo}"
mkdir -p -- "$CARGO_HOME"
printf 'Резервная копия: %s\n' "$BACKUP_FILE"
printf 'Проверяю сборку (cargo check --locked)...\n'

if (cd -- "$PROJECT_DIR" && cargo check --locked); then
    printf '\nГотово: интерфейс обновлён под макет 1280×720, сборка успешна.\n'
    printf 'Запуск: cd "%s" && CARGO_HOME="%s" cargo run --locked\n' "$PROJECT_DIR" "$CARGO_HOME"
    printf 'Резервная копия: %s\n' "$BACKUP_FILE"
else
    printf '\nСборка не прошла. Восстанавливаю прежний src/ui.rs...\n' >&2
    cp -a -- "$BACKUP_FILE" "$TARGET_FILE"
    printf 'Исходный файл восстановлен. Проверь ошибку cargo выше.\n' >&2
    exit 1
fi
