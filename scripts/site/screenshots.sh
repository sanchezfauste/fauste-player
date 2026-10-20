#!/usr/bin/env bash
# Takes the screenshots of the README and the user guide from a scripted
# scene, in a virtual X server (Linux):
#
#   scripts/site/screenshots.sh [--only main] [--hold] [--lang <locale>] [--out <dir>]
#
#   --only main  only the README image (main-screen.png)
#   --lang       the interface language, a locale tag of crates/fp-app/locales
#                (default en-US); the scene's own text (tracks, carts) stays
#   --out        where the images go: <dir>/main-screen.png and <dir>/guide/*.png
#                (default docs/images, the committed English set)
#   --hold       build the scene, then keep the app running until killed
#                (to look around: the display and port are printed)
#
# It builds the release binary and the demo_session example, generates
# stand-in songs and carts (scripts/site/tones.sh), writes a scratch
# FAUSTE_HOME (the demo session, and a config.json with ui.language (en-US by default),
# the remote HTTP API on, loudness normalization on and every output on the
# silent "null" backend), starts Xvfb on a free display, starts the app
# there, waits for the API and for the analysis to measure the players'
# tracks, starts the app again (a track keeps the gain it was loaded with,
# so the headers show a gain only after a start with the measurements in),
# builds the scene and captures. Nothing is ever played on a sound card: the
# script checks in the log that the null backend is the one in use before
# it sends any play command, and stops otherwise.
#
# Needs Xvfb, xdotool, xwininfo, ImageMagick (import, convert), ffmpeg,
# python3 and curl. The app reaches the songs through SHOTS_MEDIA (default
# /tmp/fauste-demo, a link to the work folder), so no screenshot shows a
# home path. The work folder is SHOTS_WORK (default:
# target/screenshots); the songs are kept there between runs. The scratch
# FAUSTE_HOME is SHOTS_HOME (default: home/ in the work folder), and it is
# replaced on every run.
set -euo pipefail

root=$(cd "$(dirname "$0")/../.." && pwd)
only=all
hold=false
lang=en-US
out=
while [[ $# -gt 0 ]]; do
    case $1 in
        --only) only=${2:?--only needs a name}; shift 2 ;;
        --hold) hold=true; shift ;;
        --lang) lang=${2:?--lang needs a locale}; shift 2 ;;
        --out) out=${2:?--out needs a folder}; shift 2 ;;
        *) echo "usage: screenshots.sh [--only main] [--hold] [--lang <locale>] [--out <dir>]" >&2; exit 2 ;;
    esac
done
case $only in all | main) ;; *) echo "screenshots.sh: unknown --only $only" >&2; exit 2 ;; esac

[[ -f $root/crates/fp-app/locales/$lang/main.ftl ]] \
    || { echo "screenshots.sh: unknown locale $lang (see crates/fp-app/locales)" >&2; exit 2; }

for tool in Xvfb xdotool xwininfo import convert ffmpeg python3 curl; do
    command -v "$tool" >/dev/null || { echo "screenshots.sh: $tool is needed" >&2; exit 1; }
done

work=${SHOTS_WORK:-$root/target/screenshots}
mkdir -p "$work"
work=$(cd "$work" && pwd)
home=${SHOTS_HOME:-$work/home}
# The app shows the paths of the files (Settings > Cartwall, tooltips), so
# the songs are reached through a short neutral link that never names a
# home folder: SHOTS_MEDIA (default /tmp/fauste-demo) -> the work folder.
media=${SHOTS_MEDIA:-/tmp/fauste-demo}
images=${out:-$root/docs/images}
mkdir -p "$images"
images=$(cd "$images" && pwd)
guide=$images/guide
log() { echo "screenshots.sh: $*" >&2; }
die() { log "$*"; exit 1; }

xvfb_pid=
app_pid=
display=
port=
win=

linked=false
cleanup() {
    set +e
    if $linked; then rm -f "$media"; fi
    if [[ -n $app_pid ]]; then kill "$app_pid" 2>/dev/null; wait "$app_pid" 2>/dev/null; fi
    if [[ -n $xvfb_pid ]]; then kill "$xvfb_pid" 2>/dev/null; wait "$xvfb_pid" 2>/dev/null; fi
}
trap cleanup EXIT
trap 'exit 130' INT TERM

# ---------------------------------------------------------------- helpers

# Points the neutral media link at the work folder, refusing a path under a
# home folder or one that is something other than our link.
link_media() {
    case $media in
        "$HOME"/* | /home/* | /Users/* | /root/*) die "SHOTS_MEDIA must not be under a home folder: $media" ;;
    esac
    if [[ -e $media || -L $media ]]; then
        [[ -L $media ]] || die "$media exists and is not a link; set SHOTS_MEDIA"
    fi
    ln -sfn "$work" "$media"
    linked=true
}

# api <method> <path> [json]: a request to the remote API; prints the body
# and fails on an HTTP error.
api() {
    local method=$1 path=$2 body=${3:-}
    if [[ -n $body ]]; then
        curl -sSf -X "$method" -H 'Content-Type: application/json' -d "$body" \
            "http://127.0.0.1:$port/api/v1$path"
    else
        curl -sSf -X "$method" "http://127.0.0.1:$port/api/v1$path"
    fi
}

# json <python expression over d>: reads JSON on stdin, prints the value.
json() { python3 -c "import json,sys; d=json.load(sys.stdin); v=($1); print(v if not isinstance(v,(dict,list)) else json.dumps(v))"; }

x() { env -u WAYLAND_DISPLAY DISPLAY=":$display" "$@"; }

# Waits up to <secs> for a command to succeed.
wait_for() {
    local secs=$1; shift
    local end=$((SECONDS + secs))
    until "$@" >/dev/null 2>&1; do
        [[ $SECONDS -ge $end ]] && return 1
        sleep 0.25
    done
}

window_id() {
    x xdotool search --name "^Fauste Player" 2>/dev/null | head -n1
}

# capture <file> [WxH+X+Y]: the app window, or a part of it, as a small PNG.
capture() {
    local file=$1 crop=${2:-}
    local raw=$work/raw.png
    mkdir -p "$(dirname "$file")"
    x import -window "$win" "$raw"
    if [[ -n $crop ]]; then
        convert "$raw" -crop "$crop" +repage "$raw"
    fi
    # An 8-bit palette keeps the files small; the flat UI looks the same.
    convert "$raw" -strip -dither None -colors 256 "PNG8:$file"
    log "captured ${file#"$root"/} ($(du -k "$file" | cut -f1) KB)"
}

# The centres (x) of the bordered buttons at the right end of the top bar,
# left to right: About, then Settings. Their places depend on the length of
# the Settings label in the interface language, so they are found on the
# screen: the first row from the top with long runs of a colour other than
# the bar's is the buttons' top border.
topbar_buttons() {
    local raw=$work/bar.png
    x import -window "$win" "$raw"
    convert "$raw" -crop 960x16+960+0 +repage -depth 8 txt:- | python3 -c '
import re, sys
rows = {}
for line in sys.stdin:
    m = re.match(r"(\d+),(\d+):.*(#[0-9A-Fa-f]{6})", line)
    if m:
        rows.setdefault(int(m[2]), []).append(m[3])
for y in sorted(rows):
    row, bar, runs, start = rows[y], rows[y][0], [], None
    for x, colour in enumerate(row + [bar]):
        if colour != bar and start is None:
            start = x
        elif colour == bar and start is not None:
            if x - start >= 10:
                runs.append((start + x - 1) // 2 + 960)
            start = None
    if len(runs) >= 2:
        print(*runs[-2:])
        break
'
}

# capture_window <file> <x> <y> <radius> [left top right bottom]: the window
# that covers the point (x, y), an empty part of it, found on the screen
# (scripts/site/window_box.py), widened by the margins. A window's size depends on the language of its
# text, so a fixed rectangle would cut it in some languages.
capture_window() {
    local file=$1 box raw=$work/window.png
    shift
    x import -window "$win" "$raw"
    box=$(python3 "$root/scripts/site/window_box.py" "$raw" "$@") \
        || die "cannot find the window at $1,$2 for $file"
    capture "$file" "$box"
}

# The pointer moves first and rests, so that the UI sees it hover before
# the press (a click in the same frame as the move can be lost).
click() { x xdotool mousemove --window "$win" "$1" "$2"; sleep 0.3; x xdotool click 1; sleep 0.8; }
right_click() { x xdotool mousemove --window "$win" "$1" "$2"; sleep 0.3; x xdotool click 3; sleep 0.8; }
# Moves the pointer out of the way (no tooltips, no hover).
park() { x xdotool mousemove --window "$win" 1910 1075; sleep 0.3; }
# Xvfb has no window manager: focus the window, then type (a key sent to
# the window directly is a synthetic event, which the app ignores).
key() { x xdotool windowfocus "$win" 2>/dev/null; x xdotool key "$@" 2>/dev/null; sleep 0.6; }

# Racy if another Xvfb takes the same display meanwhile; fine for a dev script.
free_display() {
    local n
    for n in $(seq 77 140); do
        [[ -e /tmp/.X11-unix/X$n || -e /tmp/.X$n-lock ]] || { echo "$n"; return; }
    done
    return 1
}

# The API's default port when it is free (Settings > Remote shows it), else
# any free one.
free_port() {
    python3 -c "
import socket
s = socket.socket()
try:
    s.bind(('127.0.0.1', 7380))
except OSError:
    s.bind(('127.0.0.1', 0))
print(s.getsockname()[1])"
}

# The scratch state: the demo session, then config.json patched for the
# shots. Every route names the null backend, so no sound card is opened.
write_home() {
    # Only a folder this script made (or an empty one) is ever replaced.
    if [[ -e $home && ! -d $home ]]; then
        die "$home is not a folder; refusing to replace it"
    fi
    if [[ -d $home && -n $(ls -A "$home") && ! -e $home/.screenshots-home ]]; then
        die "$home is not a scratch folder made by this script; refusing to replace it"
    fi
    rm -rf "$home"
    mkdir -p "$home"
    touch "$home/.screenshots-home"
    FAUSTE_HOME=$home "$root/target/release/examples/demo_session" "$media/music" "$media/carts" >&2
    python3 - "$home" "$port" "$lang" <<'EOF'
import json, sys
home, port, lang = sys.argv[1], int(sys.argv[2]), sys.argv[3]
path = f"{home}/config/config.json"
doc = json.load(open(path))
players = [p["id"] for p in json.load(open(f"{home}/data/session.json"))["players"]]
c = doc["config"]
null = lambda ch: {"backend": "null", "device": "null", "first_channel": ch}
c["outputs"]["backend"] = "null"
c["outputs"]["routes"] = [{"player": p, "main": null(0), "cue": null(2)} for p in players]
c["outputs"]["cartwall"] = {"main": null(0), "cue": null(2)}
c["outputs"]["bit_perfect"] = []
c["ui"]["language"] = lang
# Loudness normalization on (the other loudness settings keep their
# defaults), so the players' headers show the gain badge.
c.setdefault("loudness", {})["normalize"] = True
c["remote"]["http"]["enabled"] = True
c["remote"]["http"]["bind"] = "127.0.0.1"
c["remote"]["http"]["port"] = port
json.dump(doc, open(path, "w"), indent=2)
EOF
}

# start_app: Xvfb (once) and the app, then waits for the API and the window.
start_app() {
    if [[ -z $xvfb_pid ]]; then
        display=$(free_display) || die "no free X display"
        Xvfb ":$display" -screen 0 1920x1080x24 -nolisten tcp >/dev/null 2>&1 &
        xvfb_pid=$!
        wait_for 10 x xwininfo -root || die "Xvfb did not start"
    fi
    # The log line check_silent reads must come from this start, not an
    # earlier one.
    local lines_before
    lines_before=$(audio_lines)
    # Belt and braces: the sound servers are out of reach too. Started with
    # env directly (not through the x function, which would run in a
    # subshell), so app_pid is the app itself and stop_app really stops it.
    env -u WAYLAND_DISPLAY DISPLAY=":$display" FAUSTE_HOME="$home" \
        PULSE_SERVER=unix:/nonexistent PIPEWIRE_REMOTE=fauste-none \
        JACK_NO_START_SERVER=1 "$root/target/release/fauste-player" \
        >"$work/app.out" 2>&1 &
    app_pid=$!
    wait_for 30 api GET /players || die "the remote API did not answer (see $work/app.out)"
    wait_for 15 window_id || die "no app window"
    win=$(window_id)
    x xdotool windowmove "$win" 0 0 windowsize "$win" 1920 1080
    sleep 1
    wait_for 10 audio_lines_above "$lines_before" \
        || die "this start wrote no 'audio systems' line in the log; refusing to play"
    check_silent
}

# The number of "audio systems" lines in the logs so far.
audio_lines() {
    cat "$home"/logs/fauste-player*.log 2>/dev/null | grep -c "audio systems" || true
}

audio_lines_above() {
    (($(audio_lines) > $1))
}

# Aborts unless the log says that the audio system in use is null.
check_silent() {
    local line
    line=$(grep -h "audio systems" "$home"/logs/fauste-player*.log 2>/dev/null | tail -n1 || true)
    [[ -n $line ]] || die "no 'audio systems' line in the log; refusing to play"
    case $line in
        *" backend=null "*) log "silent backend confirmed: ${line#*audio systems}" ;;
        *) die "the audio system in use is not null; refusing to play: $line" ;;
    esac
    # Every output bus the log names must be on the null backend.
    if grep -ho 'backend: "[^"]*"' "$home"/logs/fauste-player*.log 2>/dev/null \
        | grep -qv 'backend: "null"'; then
        die "an output of another audio system shows in the log; refusing to play"
    fi
}

stop_app() {
    [[ -n $app_pid ]] || return 0
    kill "$app_pid" 2>/dev/null || true
    # Bounded: an app stuck on shutdown is killed rather than waited for.
    wait_for 10 app_gone || kill -KILL "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    app_pid=
}

app_gone() {
    ! kill -0 "$app_pid" 2>/dev/null
}

# ------------------------------------------------------------------ scene

# Waits until the tracks on the players are analysed (their waveforms) and
# their loudness is measured (the gain badge in the headers, rather than
# "not measured").
wait_analysed() {
    local t tracks
    tracks=$(player_tracks) || die "cannot read the players' tracks"
    [[ -n $tracks ]] || die "no tracks on the players"
    for t in $tracks; do
        wait_for 60 track_ready "$t" || die "track $t is not analysed and measured after 60 s"
    done
}

# The ids of the tracks on the players (current and next).
player_tracks() {
    api GET /players | json "' '.join(str(e['track']['id']) for p in d for e in (p['current'], p['next']) if e)"
}

# saved_measured <track id>...: succeeds when the saved library
# (data/playlists.json under the home; its "library" is the array of tracks)
# has a loudness measurement for every given track. A missing or unreadable
# file is "not yet".
saved_measured() {
    python3 - "$home/data/playlists.json" "$@" <<'PY'
import json, sys
try:
    with open(sys.argv[1]) as f:
        library = json.load(f)["library"]
    saved = {str(t["id"]): t.get("loudness") for t in library}
except Exception:
    sys.exit(1)
sys.exit(0 if all(saved.get(t) is not None for t in sys.argv[2:]) else 1)
PY
}

# Succeeds when the track is analysed and its loudness measured.
track_ready() {
    [[ $(api GET "/tracks/$1" | json "d['analyzed'] and d['loudness'] is not None") == True ]]
}

# The scene (spec O29): players 1 and 2 on air some way into their
# playlists, player 2 at mid-track, player 1's next 3 entries after the one
# on air, player 3 paused, player 4 stopped, and one cart playing.
build_scene() {
    local players p1 p2 list current next half cart
    wait_analysed
    players=$(api GET /players)
    p1=$(json "d[0]['id']" <<<"$players")
    p2=$(json "d[1]['id']" <<<"$players")
    list=$(json "d[0]['playlist']" <<<"$players")
    current=$(json "d[0]['current']['entry']" <<<"$players")
    half=$(json "round(d[1]['current']['track']['duration_secs'] / 2, 1)" <<<"$players")
    next=$(api GET "/playlists/$list" \
        | json "(lambda ids: ids[ids.index($current) + 3])([e['id'] for e in d['entries']])")
    cart=$(api GET /cartwall \
        | json "[c['id'] for p in d['pages'] for c in p['carts'] if c['name'] == 'Spot – Riverside Bakery'][0]")

    check_silent
    api POST "/players/$p1/play" >/dev/null
    api POST "/players/$p2/play" >/dev/null
    api PUT "/players/$p1/next" "{\"entry\": $next}" >/dev/null
    api POST "/players/$p1/seek" '{"secs": 71.5}' >/dev/null
    api POST "/players/$p2/seek" "{\"secs\": $half}" >/dev/null
    api POST "/carts/$cart/fire" >/dev/null
    park
    sleep 2.5
    check_silent
}

shoot_main() {
    capture "$images/main-screen.png"
}

# The guide's screenshots (spec O28), each cropped to its subject. The
# positions are those of the default 1920x1080 layout with four players;
# the windows and menus, whose size depends on the language, and the top
# bar's buttons are found on the screen.
shoot_guide() {
    local p4 n section about_x settings_x
    # From the scene, while the cart still plays.
    capture "$guide/player.png" 472x262+7+41
    capture "$guide/playlist.png" 472x614+7+309
    capture "$guide/cartwall.png" 1920x124+0+932

    # The CUE window of player 4 (stopped: it pre-listens its next track).
    p4=$(api GET /players | json "d[3]['id']")
    api PUT "/players/$p4/cue" '{"on": true}' >/dev/null
    park
    sleep 1.5
    # The empty header between the title and the close button.
    capture_window "$guide/cue-window.png" 414 234 10
    api PUT "/players/$p4/cue" '{"on": false}' >/dev/null
    sleep 0.5

    # A track's menu, then its tag editor (player 4, row 2).
    right_click 1560 406
    # The menu with the rows it opened over on its left.
    capture_window "$guide/track-menu.png" 1640 440 40 118 10 12 12
    click 1620 537
    park
    sleep 1
    # Right of the Save and Cancel buttons.
    capture_window "$guide/tag-editor.png" 1110 836 10
    key Escape

    # The About and Settings buttons, wherever the language puts them.
    read -r about_x settings_x < <(topbar_buttons) \
        || die "cannot find the About and Settings buttons in the top bar"

    # Settings, one capture per section (the list on the left).
    click "$settings_x" 17
    n=0
    for section in outputs players meters analysis loudness playlists cartwall shortcuts midi remote; do
        click 600 $((291 + n * 36))
        park
        capture "$guide/settings-$section.png" 900x640+510+220
        n=$((n + 1))
    done
    key Escape

    # About (the info button left of Settings).
    click "$about_x" 17
    park
    capture_window "$guide/about.png" 960 540 40
    key Escape
}

# ---------------------------------------------------------------- main

log "building"
(cd "$root" && cargo build --release -q -p fp-app --bins --example demo_session)
"$root/scripts/site/tones.sh" "$work/music" "$work/carts" >&2
link_media
port=$(free_port)
write_home
start_app
# The players load their tracks when the app starts, before the analysis has
# measured them, and a track keeps the gain it was loaded with: start again
# once the measurements are in and the autosave has written them to the
# saved library (the restarted app loads the tracks from it with their
# loudness), so the headers show each track's gain.
wait_analysed
tracks=$(player_tracks) || die "cannot read the players' tracks"
[[ -n $tracks ]] || die "no tracks on the players"
# shellcheck disable=SC2086 # one argument per track id
wait_for 30 saved_measured $tracks \
    || die "the saved library ($home/data/playlists.json) has no loudness for the players' tracks after 30 s"
stop_app
start_app
log "app on display :$display, API on port $port, home $home"
build_scene
if $hold; then
    log "holding; kill $$ to stop"
    while kill -0 "$app_pid" 2>/dev/null; do sleep 1; done
    exit 0
fi
shoot_main
[[ $only == main ]] && exit 0
shoot_guide
check_silent
log "done"
