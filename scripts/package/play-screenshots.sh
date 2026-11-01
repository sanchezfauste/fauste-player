#!/usr/bin/env bash
# Takes the Google Play phone screenshots (packaging/android/play/<language>/
# images/phoneScreenshots) on an Android emulator, in every interface
# language:
#
#   scripts/package/play-screenshots.sh [--apk <file>] [--out <dir>] [--lang <locale>]...
#
# --out writes <dir>/<language>/images/phoneScreenshots/*.png instead of
# into the listing; --lang (a tag of crates/fp-app/locales, repeatable)
# takes only those languages. Each locale goes to the listing folder of
# the same tag, or of its language alone (ca-ES -> ca), which must exist.
#
# It installs the APK (default: the release APK of
# `scripts/package/android.sh release`), generates the stand-in songs and
# carts of the desktop screenshots (scripts/site/tones.sh), writes the demo
# state with examples/demo_session (players paused mid-track, so nothing
# moves between two runs) and puts it straight into the app's files folder,
# which needs root: an emulator image without the Play Store (google_apis,
# `adb root`). It replaces the emulator's Music folder with the demo songs
# and carts, and stops when there is audio elsewhere on the emulator (the
# Music screen would list it). Then it fixes the screen at 1080 × 2160
# (Play refuses a taller ratio) and 420 dpi, puts the status bar in demo
# mode, opens each screen with a tap on its top-bar button and saves the
# screenshots as 24-bit PNG (Play refuses an alpha channel) once the
# screen has stopped changing. The songs are analysed once; then the app
# starts again in each language (ui.language in its config.json).
#
# The taps are in pixels of that fixed screen: after a change to the phone
# top bar or the Music screen, check them (TABS and music_scene below).
# The app's own clock and a paused player's blinking buttons change all the
# time: packaging/android/play/screenshot-masks.txt lists them, for
# scripts/site/compare-screenshots.sh and for the wait for a still screen.
#
# Needs adb (on PATH or under ANDROID_HOME) and one running emulator
# (ANDROID_SERIAL picks one of several), cargo, python3, ffmpeg and
# ImageMagick. PLAY_SHOTS_ANALYSIS_TIMEOUT (seconds, default 180) bounds
# the wait for the analysis of the songs.
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
apk=$root/mobile/android/app/build/outputs/apk/release/app-release.apk
listing=$root/packaging/android/play
out=$listing
langs=()
while [[ $# -gt 0 ]]; do
    case $1 in
        --apk) apk=${2:?--apk needs a file}; shift 2 ;;
        --out) out=${2:?--out needs a folder}; shift 2 ;;
        --lang) langs+=("${2:?--lang needs a locale}"); shift 2 ;;
        *) echo "usage: play-screenshots.sh [--apk <file>] [--out <dir>] [--lang <locale>]..." >&2; exit 2 ;;
    esac
done

die() { echo "play-screenshots.sh: $*" >&2; exit 1; }

# Every interface language by default.
if [[ ${#langs[@]} -eq 0 ]]; then
    for d in "$root"/crates/fp-app/locales/*/; do langs+=("$(basename "$d")"); done
fi
# The listing folder of a locale: its tag, else its language alone.
listing_of() {
    if [[ -d $listing/$1 ]]; then
        echo "$1"
    elif [[ -d $listing/${1%%-*} ]]; then
        echo "${1%%-*}"
    else
        return 1
    fi
}
for lang in "${langs[@]}"; do
    [[ -f $root/crates/fp-app/locales/$lang/main.ftl ]] \
        || die "unknown locale $lang (see crates/fp-app/locales)"
    listing_of "$lang" >/dev/null || die "no Google Play listing for $lang in $listing"
done

if ! command -v adb >/dev/null; then
    sdk=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}
    PATH=$PATH:$sdk/platform-tools
fi
for tool in adb cargo python3 ffmpeg convert identify; do
    command -v "$tool" >/dev/null || die "$tool is needed"
done
[[ -f $apk ]] || die "no APK at $apk (scripts/package/android.sh release builds it)"
[[ $(adb get-state 2>/dev/null) == device ]] \
    || die "no device, or several (set ANDROID_SERIAL): start an emulator first"

package=com.sanchezfauste.FaustePlayer
activity=$package/com.sanchezfauste.fausteplayer.MainActivity
# Where the songs and carts go on the device.
device_root=/storage/emulated/0/Music
device_music=$device_root/Demo
device_carts=$device_root/Carts
# The screen the taps and the masks are measured on.
WIDTH=1080
HEIGHT=2160
DENSITY=420
# The top-bar buttons (live, carts, playlists, music), in pixels.
TABS_Y=187
declare -A TABS_X=([live]=88 [carts]=225 [playlists]=362 [music]=500)
# What changes all the time on each screen, left out when waiting for it
# to settle.
masks=$root/packaging/android/play/screenshot-masks.txt

# adb shell, without the carriage returns of its output.
shell() { adb shell "$@" | tr -d '\r'; }

# Waits up to $1 seconds for the command after it to succeed.
wait_for() {
    local secs=$1; shift
    for _ in $(seq "$secs"); do
        "$@" && return 0
        sleep 1
    done
    return 1
}

is_root() { [[ $(adb shell id -u 2>/dev/null | tr -d '\r') == 0 ]]; }

# adb root restarts adbd: wait until the new one answers as root.
adb root >/dev/null 2>&1 || true
adb wait-for-device
wait_for 30 is_root \
    || die "adb has no root here: use an emulator image without the Play Store (google_apis)"
[[ $(shell getprop ro.boot.qemu; shell getprop ro.kernel.qemu) == *1* ]] \
    || die "not an emulator: this script replaces the device's Music folder"

# The songs, the carts and the demo state (the paths rewritten to where
# they are on the device).
work=$root/target/play-screenshots
home=$work/home
mkdir -p "$work"
(cd "$root" && cargo build --release -q -p fp-app --example demo_session)
"$root/scripts/site/tones.sh" "$work/music" "$work/carts" >&2
rm -rf "$home"
FAUSTE_HOME=$home "$root/target/release/examples/demo_session" "$work/music" "$work/carts" >&2
python3 - "$home/data" "$work/music" "$device_music" "$work/carts" "$device_carts" <<'EOF'
import pathlib, sys
data, *pairs = sys.argv[1:]
for f in pathlib.Path(data).glob("*.json"):
    s = f.read_text()
    for host, device in zip(pairs[::2], pairs[1::2]):
        s = s.replace(host + "/", device + "/")
    f.write_text(s)
EOF

echo "Installing $apk" >&2
adb install -r "$apk" >/dev/null
shell pm clear "$package" >/dev/null
for permission in READ_MEDIA_AUDIO POST_NOTIFICATIONS; do
    shell pm grant "$package" "android.permission.$permission"
done
shell rm -rf "$device_root"
shell mkdir -p "$device_music" "$device_carts"
adb push "$work/music/." "$device_music/" >/dev/null
adb push "$work/carts/." "$device_carts/" >/dev/null
shell content call --method scan_volume --uri content://media --arg external_primary >/dev/null
rows=$(shell content query --uri content://media/external/audio/media --projection relative_path)
grep -q 'relative_path=Music/Demo/' <<<"$rows" || die "the media scan did not find the songs"
elsewhere=$(grep '^Row' <<<"$rows" | grep -v 'relative_path=Music/' || true)
[[ -z $elsewhere ]] || die "audio outside the Music folder; delete it first:
$elsewhere"

files=/data/data/$package/files
uid=$(shell stat -c %u "/data/data/$package")
shell mkdir -p "$files"
adb push "$home/." "$files/" >/dev/null
shell chown -R "$uid:$uid" "$files"
shell restorecon -R "$files" >/dev/null

# A fixed screen and status bar; put back as they were on exit.
rotation=$(shell settings get system accelerometer_rotation)
user_rotation=$(shell settings get system user_rotation)
demo_allowed=$(shell settings get global sysui_demo_allowed)
tmp=$(mktemp -d)
put_back() {
    local namespace=$1 key=$2 value=$3
    if [[ $value == null ]]; then
        shell settings delete "$namespace" "$key" >/dev/null || true
    else
        shell settings put "$namespace" "$key" "$value" || true
    fi
}
restore() {
    rm -rf "$tmp"
    shell am broadcast -a com.android.systemui.demo -e command exit >/dev/null || true
    shell wm size reset || true
    shell wm density reset || true
    put_back system accelerometer_rotation "$rotation"
    put_back system user_rotation "$user_rotation"
    put_back global sysui_demo_allowed "$demo_allowed"
}
trap restore EXIT
shell wm size "${WIDTH}x${HEIGHT}"
shell wm density "$DENSITY"
shell settings put system accelerometer_rotation 0
shell settings put system user_rotation 0
shell settings put global sysui_demo_allowed 1
demo() { shell am broadcast -a com.android.systemui.demo -e command "$@" >/dev/null; }
demo enter
demo clock -e hhmm 0900
demo battery -e level 100 -e plugged false
demo network -e wifi show -e level 4 -e fully true -e mobile show -e level 4 -e datatype none
demo notifications -e visible false

# True when the app's window has the focus. A slow emulator (a CI runner)
# may show another app's "not responding" dialog over it: stopping that app
# closes the dialog.
in_front() {
    local focus
    focus=$(shell dumpsys window | grep mCurrentFocus)
    if [[ $focus =~ Application\ Not\ Responding:\ ([A-Za-z0-9._]+) && ${BASH_REMATCH[1]} != "$package" ]]; then
        echo "Closing the not-responding dialog of ${BASH_REMATCH[1]}" >&2
        shell am force-stop "${BASH_REMATCH[1]}"
        return 1
    fi
    [[ $focus == *"$activity"* ]]
}
# What is on the screen and what the app logged, for a start that failed.
start_report() {
    {
        shell dumpsys window | grep -E 'mCurrentFocus|mFocusedApp' || true
        shell pidof "$package" || echo "$package is not running"
        shell logcat -d -t 300 | grep -E -i 'fauste|AndroidRuntime|libc|DEBUG|ActivityManager|ActivityTaskManager' | tail -80 || true
    } >&2
}
# Starts the app and waits until its window has the focus (a cold
# emulator on a CI runner can take a while).
start_app() {
    shell am start -n "$activity" >/dev/null
    wait_for 90 in_front || { start_report; die "the app did not come to the front"; }
}

# The first start analyses the songs (waveforms, durations, loudness) and
# saves them; the second shows them from the start.
library_ready() {
    # The database and its write-ahead log, copied to the host (the log
    # first, so no change is lost between the two copies): the saved rows may
    # still be in the log. SQLite rebuilds the shared-memory file from it.
    local dir=$tmp/library suffix
    rm -rf "$dir"
    mkdir -p "$dir"
    for suffix in -wal ""; do
        adb exec-out "cat '$files/data/fauste.db$suffix' 2>/dev/null" > "$dir/fauste.db$suffix" || true
    done
    python3 - "$dir/fauste.db" <<'PY'
import json, sqlite3, sys
try:
    db = sqlite3.connect(sys.argv[1], timeout=5)
    docs = [json.loads(r[0]) for r in db.execute("SELECT doc FROM tracks")]
    db.close()
except Exception:
    sys.exit(1)
sys.exit(0 if docs and all(t.get("analyzed") and t.get("loudness") for t in docs) else 1)
PY
}
start_app
timeout=${PLAY_SHOTS_ANALYSIS_TIMEOUT:-180}
wait_for "$timeout" library_ready || die "the songs were not analysed after $timeout s"

# Whether the status bar (the bottom of the screen, over the navigation
# bar) shows an alert: its text has the alert colour (theme::ON_AIR_TEXT).
alert_shown() {
    local red
    red=$(convert "$1" -alpha off -crop "${WIDTH}x80+0+$((HEIGHT - 130))" +repage -fuzz 15% \
        -fill black +opaque '#ff645f' -fill white -opaque '#ff645f' \
        -format '%[fx:int(mean*w*h)]' info:)
    [[ $red -gt 0 ]]
}

# Saves the screen to $1 once two captures in a row are the same (the
# image's masks left out) with the app in front, so neither a half-drawn
# screen nor a dialog over it is kept: within 30 s, or 90 s while a
# dropout alert shows in the status bar (the emulator, without audio,
# stutters now and then, and the alert lasts 5 s after the last dropout).
# On failure the last screen is kept as <name>.failed.png.
settled_capture() {
    local file=$1 previous="" now size img rect x y w h draw=() why
    local start=$SECONDS limit=30
    why="nothing was captured"
    rm -f "$tmp/shot.png"
    while read -r img rect _; do
        # shellcheck disable=SC2053 # the image is a pattern
        [[ $file == $img ]] || continue
        IFS=, read -r x y w h <<<"$rect"
        draw+=(-draw "rectangle $x,$y $((x + w - 1)),$((y + h - 1))")
    done < <(grep -v '^[[:space:]]*#' "$masks")
    while ((SECONDS - start < limit)); do
        if ! in_front; then
            why="the app was not in front"
            previous=""
            sleep 0.5
            continue
        fi
        adb exec-out screencap -p > "$tmp/shot.png"
        if alert_shown "$tmp/shot.png"; then
            why="the status bar showed an alert"
            limit=90
            previous=""
            sleep 0.5
            continue
        fi
        # The pixels only: a PNG written by ImageMagick carries its time.
        now=$(convert "$tmp/shot.png" -fill black "${draw[@]}" RGB:- | md5sum)
        if [[ $now == "$previous" ]]; then
            size=$(identify -format '%wx%h' "$tmp/shot.png")
            [[ $size == "${WIDTH}x${HEIGHT}" ]] || die "the screen is $size, not ${WIDTH}x${HEIGHT}"
            convert "$tmp/shot.png" -alpha off "PNG24:$file"
            return 0
        fi
        why="the screen kept changing"
        previous=$now
        sleep 0.5
    done
    [[ ! -s $tmp/shot.png ]] || cp "$tmp/shot.png" "${file%.png}.failed.png"
    die "no settled screen after $limit s, last because $why ($file)"
}

tap() { shell input tap "$1" "$2"; sleep "${3:-1}"; }

# The Music screen: the Demo folder, three songs chosen.
music_scene() {
    tap 300 840 1.5 # Demo: the root lists the folders of Music, Carts first
    for y in 866 1017 1320; do
        tap 80 "$y" 0.7
    done
}

# shoot <dir> <name> <tab> [scene]
shoot() {
    tap "${TABS_X[$3]}" "$TABS_Y" 1
    [[ -z ${4:-} ]] || "$4"
    settled_capture "$1/$2.png"
    echo "$1/$2.png"
}
# Starts the app again in the interface language $1: the host's demo
# config.json, with that language, replaces the device's (anything the app
# wrote to it is lost; nothing it writes matters to the screenshots).
start_in() {
    shell am force-stop "$package"
    python3 - "$home/config/config.json" "$1" <<'EOF2'
import json, sys
path, lang = sys.argv[1:]
with open(path) as f:
    doc = json.load(f)
doc["config"].setdefault("ui", {})["language"] = lang
with open(path, "w") as f:
    json.dump(doc, f, indent=2)
EOF2
    adb push "$home/config/config.json" "$files/config/config.json" >/dev/null
    shell chown "$uid:$uid" "$files/config/config.json"
    shell restorecon "$files/config/config.json" >/dev/null
    start_app
}
for lang in "${langs[@]}"; do
    dir=$out/$(listing_of "$lang")/images/phoneScreenshots
    mkdir -p "$dir"
    echo "Screenshots in $lang" >&2
    start_in "$lang"
    shoot "$dir" 1-live live
    shoot "$dir" 2-carts carts
    shoot "$dir" 3-playlists playlists
    shoot "$dir" 4-music music music_scene
done
