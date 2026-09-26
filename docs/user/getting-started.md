# Getting started

## Install

Download the archive for your system from the project's **Releases** page:

| System | Archive |
|---|---|
| Linux (x86-64) | `fauste-player-<version>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (ARM64) | `fauste-player-<version>-aarch64-unknown-linux-gnu.tar.gz` |
| Windows (x86-64) | `fauste-player-<version>-x86_64-pc-windows-msvc.zip` |
| macOS (Intel) | `fauste-player-<version>-x86_64-apple-darwin.tar.gz` |
| macOS (Apple silicon) | `fauste-player-<version>-aarch64-apple-darwin.tar.gz` |

Each archive has a `.sha256` file. To check the download:

- Linux: `sha256sum -c <archive>.sha256`
- macOS: `shasum -a 256 -c <archive>.sha256`

Extract the archive and run `fauste-player` (`fauste-player.exe` on Windows).

- **Linux:** the ALSA library must be installed (`libasound2` on Debian and
  Ubuntu, `alsa-lib` on Fedora), and so must the D-Bus library (`libdbus-1`),
  which is used to request real-time priority. Desktop systems have both.
- **macOS:** the binaries are not signed yet (signing comes with Phase 5
  packaging). The first time, right-click the binary and choose **Open**, or
  run `xattr -d com.apple.quarantine fauste-player`.
- **Windows:** SmartScreen may warn about an unknown publisher. Choose **More
  info → Run anyway**.

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
