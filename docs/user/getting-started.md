# Getting started

## Install

Download a package for your system from the project's **Releases** page.
Every file has a `.sha256` next to it. To check a download, run
`sha256sum -c <file>.sha256` on Linux, or `shasum -a 256 -c <file>.sha256`
on macOS.

### Linux

| Package | Install | Update | Remove |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | install the newer `.deb` the same way | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | install the newer `.rpm` | `sudo dnf remove fauste-player` |
| AppImage (any distribution) | `chmod +x fauste-player-<version>-x86_64.AppImage`, then run it | replace the file | delete the file |
| Flatpak bundle | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | install the newer bundle | `flatpak uninstall org.fauste.FaustePlayer` |
| Archive | extract it and run `fauste-player` | extract the newer one | delete the folder |

- **Desktop integration:** the packages add **Fauste Player** to the
  applications menu, and let you open `.m3u`, `.m3u8` and `.pls` playlists
  with it, which imports them as new playlists.
- **Libraries needed:** the ALSA and D-Bus libraries. Every desktop has them,
  and the packages declare them. JACK is used when it is installed, and is
  never required.
- **AppImage:** if it does not start because FUSE is missing, run it with
  `--appimage-extract-and-run`.
- **Flatpak:**
  - The bundle needs the freedesktop runtime from Flathub. If the Flathub
    remote is not set up, run
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`
    first.
  - The sandbox can read your home folder (to play your music where it is).
  - It plays through PulseAudio, or through ALSA directly for bit-perfect
    devices.
- **Desktop libraries:** the window uses libxkbcommon and EGL or OpenGL, with
  Wayland or X11. Every desktop has them, and the `.deb` and `.rpm` declare
  them. On a very minimal system, install them before using the AppImage.

### Windows

Run `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. It installs for
all users under *Program Files* and adds a Start menu entry.
- **Updating:** run the newer installer. It replaces the installed version.
- **Removing:** use **Settings → Apps**.
- **Unsigned installer:** if the release is not signed, SmartScreen warns
  about an unknown publisher. Choose **More info → Run anyway**.

The `.zip` archive is a portable alternative: extract it and run
`fauste-player.exe`.

### macOS

Open `fauste-player-<version>-macos-universal.dmg` and drag **Fauste
Player** to **Applications**. The same app runs on Apple silicon and Intel
(macOS 11 or later).
- **Updating:** replace the app the same way.
- **Removing:** move it to the Bin.
- **Unsigned app:** if the release is not signed and notarised, macOS refuses
  the first start.
  - macOS 15 and later: open **System Settings → Privacy & Security**,
    scroll to the message about Fauste Player, choose **Open Anyway**, and
    confirm.
  - macOS 14 and earlier: right-click the app, choose **Open**, then confirm.
  - Or run `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`.
- **JACK on macOS:** a signed and notarised app can only load a JACK library
  that is itself signed. Otherwise JACK shows as unavailable; use Core
  Audio.
- **Playlists:** on macOS they are imported from Settings → Playlists.
  Opening a playlist file with the app from Finder is not supported.

### One instance at a time

Only one Fauste Player runs per data folder. Opening a playlist from the file
manager while it runs imports that playlist into the running application.
Starting it again with no playlist shows a message that it is already
running. To run separate instances side by side (for example two studios on
one computer), give each its own folder with `FAUSTE_HOME`.

### Command line

`fauste-player --version` prints the version. `fauste-player --help` lists
the options. Playlist files given as arguments are imported as new
playlists.

## First start

The window opens with four players, each showing an empty playlist. Nothing
plays until you press Play: this is also true after a restart or a crash.

## Add music

- Click **+ Add** at the bottom of a player and pick files, or
- drag audio files or folders from your file manager onto a track list.
  Folders add the audio files directly inside them, not their sub-folders.

Supported formats: WAV, AIFF, FLAC, MP3, OGG Vorbis, AAC/M4A and ALAC.

Each file is analysed in the background. The analysis reads the title,
artist, album and cover art, draws the waveform, and finds where the sound
starts and ends and where to mix into the next track. You can play a track
before its analysis finishes.

## Play

- Press **Play** (or the number key of the player, `1` for P1) to start the
  **next** track, marked green in the list.
- Press **Play** again while a track is on air to fade into the next one.
- **Double-click** a track to make it the next one.

In **CONT** (continuous) mode the player mixes into the next track by itself
at the MIX point. In **SINGLE** mode it stops at the end of every track.
Continue with [Players](players.md).
