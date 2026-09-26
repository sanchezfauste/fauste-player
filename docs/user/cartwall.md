# Cartwall

The cartwall is the strip of buttons under the players. Each button, a
**cart**, plays one sound instantly: jingles, effects, spots. Carts play on
their own outputs, independently of the players.

## Using it

- **Click a cart** to fire it. **Click it again** to stop it.
- While a cart plays, its border turns red, a red bar shrinks as it plays,
  and its time counts down.
- Carts **overlap** by default: firing a second one does not stop the first.
  A cart set to **Stop other carts when fired** stops every other cart on air
  first, on any page.
- A cart set to **Loop** starts again from its cue-in when it reaches its end,
  without a gap, until you stop it.
- **Right-click** a cart for more options:

| Item | Action |
|---|---|
| Pre-listen on CUE | Play it on the cartwall's CUE output |
| Stop | Stop it |
| Edit… | Open it in Settings |

- **Pages:** the tabs next to **CARTWALL** switch pages. A red dot shows
  that a cart on that page is playing.
- Click **CARTWALL** to collapse the strip or expand it again.

| Button look | Meaning |
|---|---|
| Violet dot | Jingle |
| Amber dot | Effect |
| Grey dot | Spot (commercial) |
| ↻ after the type | Loops |
| ✋ after the type | Stops the other carts when fired |
| Warning sign | The file is missing or cannot be decoded |
| "Empty", dimmed | No file assigned |

Carts use the same markers as tracks. They start at their cue-in and end at
their cue-out, which you can edit on a player's waveform when the file is
loaded there.

## Keyboard

By default **F1**…**F12** fire carts 1–12 of the page shown, and
**Ctrl+Space** stops every cart. See [Keyboard](keyboard.md) to change them.

## Setting up carts

Go to **Settings → Cartwall**:

- **Pages:** create, rename, delete (the last page cannot be deleted), and set
  the grid size (rows × columns). A smaller grid is refused if it would drop
  carts that have a file.
- **Import… / Export…** save a page to a `.cartpage.json` file and load it
  back, for example to share it between studios. Relative file paths are
  resolved against the file's folder.
- **Carts:** click a cart in the grid, then set its name, file (**Choose…**
  or **Clear**), type, **Loop** and **Stop other carts when fired**.

The cartwall's own Main and Cue outputs are in **Settings → Audio outputs**
(the **Cartwall** row).
