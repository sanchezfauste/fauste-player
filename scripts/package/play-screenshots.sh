#!/usr/bin/env bash
# Takes the Google Play phone screenshots (packaging/android/play/en-US/
# images/phoneScreenshots) on an Android emulator:
#
#   scripts/package/play-screenshots.sh [--apk <file>] [--out <dir>]
#
# It installs the APK (default: the release APK of
# `scripts/package/android.sh release`), generates the stand-in songs and
# carts of the desktop screenshots (scripts/site/tones.sh), writes the demo
# state with examples/demo_session (players paused mid-track, so nothing
# moves between two runs) and puts it straight into the app's files folder,
# which needs root: an emulator image without the Play Store (google_apis,
# `adb root`). It replaces the emulator's Music folder with the demo songs
# and carts, and stops when there is audio elsewhere on the emulator (the
# Music screen would list it). Then it fixes the screen at 1080 × 2160 (Play refuses a
# taller ratio) and 420 dpi, puts the status bar in demo mode, opens each
# screen with a tap on its top-bar button and saves the screenshots as
# 24-bit PNG (Play refuses an alpha channel).
#
# The taps are in pixels of that fixed screen: after a change to the phone
# top bar or the Music screen, check them (TABS and music_scene below). The app's own clock changes on every
# run; packaging/android/play/screenshot-masks.txt hides it from
# scripts/site/compare-screenshots.sh.
#
# Needs adb (on PATH or under ANDROID_HOME), one running device, cargo,
# python3, ffmpeg and ImageMagick.
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
apk=$root/mobile/android/app/build/outputs/apk/release/app-release.apk
out=$root/packaging/android/play/en-US/images/phoneScreenshots
while [[ $# -gt 0 ]]; do
    case $1 in
        --apk) apk=${2:?--apk needs a file}; shift 2 ;;
        --out) out=${2:?--out needs a folder}; shift 2 ;;
        *) echo "usage: play-screenshots.sh [--apk <file>] [--out <dir>]" >&2; exit 2 ;;
    esac
done

die() { echo "play-screenshots.sh: $*" >&2; exit 1; }

if ! command -v adb >/dev/null; then
    sdk=${ANDROID_HOME:-${ANDROID_SDK_ROOT:-$HOME/Android/Sdk}}
    PATH=$PATH:$sdk/platform-tools
fi
for tool in adb cargo python3 ffmpeg convert; do
    command -v "$tool" >/dev/null || die "$tool is needed"
done
[[ -f $apk ]] || die "no APK at $apk (scripts/package/android.sh release builds it)"
[[ $(adb get-state 2>/dev/null) == device ]] || die "no device: start an emulator first"

package=com.sanchezfauste.FaustePlayer
activity=$package/com.sanchezfauste.fausteplayer.MainActivity
# Where the songs and carts go on the device.
device_root=/storage/emulated/0/Music
device_music=$device_root/Demo
device_carts=$device_root/Carts
# The top-bar buttons (live, carts, playlists, music) on a 1080 × 2160,
# 420 dpi screen, in pixels.
TABS_Y=187
declare -A TABS_X=([live]=88 [carts]=225 [playlists]=362 [music]=500)

shell() { adb shell "$@"; }

# Waits up to $1 seconds for the command after it to succeed.
wait_for() {
    local secs=$1; shift
    for _ in $(seq "$secs"); do
        "$@" && return 0
        sleep 1
    done
    return 1
}

adb root >/dev/null 2>&1 || true
wait_for 20 adb get-state >/dev/null 2>&1 || die "the device did not come back after adb root"
[[ $(shell getprop ro.boot.qemu; shell getprop ro.kernel.qemu) == *1* ]] \
    || die "not an emulator: this script replaces the device's Music folder"
[[ $(shell id -u | tr -d '\r') == 0 ]] \
    || die "adb has no root here: use an emulator image without the Play Store (google_apis)"

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
elsewhere=$(shell content query --uri content://media/external/audio/media --projection relative_path \
    | grep '^Row' | grep -v 'relative_path=Music/' || true)
[[ -z $elsewhere ]] || die "audio outside the Music folder; delete it first:
$elsewhere"

files=/data/data/$package/files
uid=$(shell stat -c %u "/data/data/$package" | tr -d '\r')
shell mkdir -p "$files"
adb push "$home/." "$files/" >/dev/null
shell chown -R "$uid:$uid" "$files"
shell restorecon -R "$files"

# A fixed screen and status bar; put back as they were on exit.
restore() {
    shell wm size reset || true
    shell wm density reset || true
    shell am broadcast -a com.android.systemui.demo -e command exit >/dev/null || true
}
trap restore EXIT
shell wm size 1080x2160
shell wm density 420
shell settings put system accelerometer_rotation 0
shell settings put system user_rotation 0
shell settings put global sysui_demo_allowed 1
demo() { shell am broadcast -a com.android.systemui.demo -e command "$@" >/dev/null; }
demo enter
demo clock -e hhmm 0900
demo battery -e level 100 -e plugged false
demo network -e wifi show -e level 4 -e mobile show -e level 4 -e datatype none
demo notifications -e visible false

# The first start analyses the songs (waveforms, durations, loudness) and
# saves them; the second shows them from the start.
library_ready() {
    shell cat "$files/data/playlists.json" 2>/dev/null | python3 -c '
import json, sys
tracks = json.load(sys.stdin)["library"]
sys.exit(0 if tracks and all(t.get("analyzed") and t.get("loudness") for t in tracks) else 1)'
}
shell am start -n "$activity" >/dev/null
wait_for 180 library_ready || die "the songs were not analysed after 180 s"
shell am force-stop "$package"
shell am start -n "$activity" >/dev/null
sleep 8

mkdir -p "$out"
# The Music screen: the Demo folder, three songs chosen.
music_scene() {
    tap 300 840 1.5 # Demo: the root lists the folders of Music, Carts first
    for y in 866 1017 1320; do
        tap 80 "$y" 0.7
    done
}

tap() { shell input tap "$1" "$2"; sleep "${3:-1}"; }

# shoot <name> <tab> [scene]
shoot() {
    tap "${TABS_X[$2]}" "$TABS_Y" 2
    [[ -z ${3:-} ]] || "$3"
    sleep 2
    adb exec-out screencap -p > "$out/$1.png"
    convert "$out/$1.png" -alpha off "PNG24:$out/$1.png"
    echo "$out/$1.png"
}
shoot 1-live live
shoot 2-carts carts
shoot 3-playlists playlists
shoot 4-music music music_scene
