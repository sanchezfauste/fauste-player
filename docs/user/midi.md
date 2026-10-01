# MIDI control surfaces

Fauste Player can be played from MIDI controllers: pad and fader
controllers, DJ-style surfaces or keyboards. Each player's transport buttons
and volume fader can be bound to a button, a key or a fader, and buttons with
lights show what each player is doing.

## Turning it on

Open **Settings → MIDI** and tick **Use MIDI control surfaces**. The list
under **Input ports** shows every MIDI input the computer has and whether it
is connected. Controllers can be plugged in or out while the program runs:
every couple of seconds the ports are looked for again, and a controller that
comes back is connected by its name, with its lights set again.

On Linux, MIDI goes through ALSA: your user must be allowed to open the
sequencer (`/dev/snd/seq`, usually by being in the `audio` group).

## Binding a control

For every player there is a row per action: **Play / Next**, **Pause**,
**Stop**, **Fade stop**, **Restart**, **Previous**, **CUE** and **Volume**.

1. Click **Learn** on the row.
2. Press the button or move the fader you want (it says **Move a control…**
   meanwhile). A button takes a key, a pad or a button that sends a control
   change; **Volume** takes a fader or knob (a control change) or a pitch-bend
   fader.
3. The row shows the device and the control, for example
   `APC mini · Note 36, channel 1`.

If that control was already bound to another action, it moves to this one.
`Esc` or clicking the button again cancels learning. **Clear** removes a
binding.

## How the controls behave

- A button acts when it is pressed (a key or pad going down, or a control
  change going up through the middle of its range), exactly like the
  player's button on screen. A button that is dimmed on screen does nothing.
- A fader moves the volume on the same scale as the on-screen fader. To
  avoid jumps, it only takes over when it reaches or passes the current
  volume (soft takeover); if the volume is changed on screen, the fader has
  to catch it up again. Nothing changes until you move a control: starting
  the program never sends anything on air.
- With **Light the buttons (LED feedback)** on, bound buttons light up:
  Play while the player is on air, Pause blinking while paused, CUE while
  pre-listening, and Stop, Fade stop, Restart and Previous while they can
  act. The lights go to the controller's output port of the same name; a
  different one can be set in `config.json` (`midi.devices`).
