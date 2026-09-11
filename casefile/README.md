# casefile

An email-inbox detective game built on logic grid deduction. Every case has a
handful of suspects, a crime, and one guilty fact. You interrogate the suspects by
email, pin down who did what on the notepad grid, and accuse when you are sure.

Play it in the browser at <https://jbutterwick.github.io/logic-grid-term/> or in
the terminal:

```sh
cargo install casefile
casefile
```

## Command line

Run `casefile` with no flags for the setup screen. Any flag skips setup and opens
a case directly.

| Flag | Meaning |
|------|---------|
| `--seed N` | Case seed. Nanoseconds since the epoch if omitted. |
| `--level L` | `easy`, `medium`, or `hard` (default `easy`) |
| `--theme NAME` | Theme name, case-insensitive. Random if omitted; an unknown name prints the list and exits. |
| `--adult` | Profanity, harsher crimes, innuendo. Nothing explicit. |

```sh
casefile --level hard --theme "Haunted Hotel" --seed 42
```

The web build always starts at the setup screen; type a seed there.

## Seeds

A case is fully determined by seed, level, theme, and adult toggle. Two players
who share those four values get the same suspects, the same statements, and the
same culprit. The setup screen shows the seed it picked so you can pass it on.

## Levels

| Level | Grid | Liars | Notes |
|-------|------|-------|-------|
| easy | 3x4 | 0 | Everyone tells the truth. |
| medium | 4x4 | 1 | The liar adds two false statements. |
| hard | 5x5 | 2 | The culprit is a liar and lies about the guilty fact. Tells are subtle. |

## Liars and pressing

Each suspect holds a share of the truthful clues. The truthful statements alone
always solve the case. Liars mix in false statements on top, and nothing a liar
says removes a true clue from the pool.

Asking a suspect to press on their last answer repeats a truthful statement word
for word. A false statement drifts instead. On hard the drift is small, so read
carefully. Replies arrive after your next action, not immediately, so send a
question and then open another thread or press `w` to wait for mail.

The inbox shows a progress bar of truthful statements collected and a tick
counter. Once you have more than half the facts the chief sends a nudge.

## Accusing

An accusation names a suspect and the guilty fact. A right accusation closes the
case. A wrong one never ends the case, but the accused stops replying and your
final rank drops.

| Wrong accusations | Rank |
|-------------------|------|
| 0 | Inspector |
| 1 | Detective |
| 2 or more | Constable |

## Adult mode

The adult toggle on the setup screen, or `--adult`, swaps in profanity, harsher
crimes, and innuendo. There is no sexually explicit content in either mode. Off by
default.

## Keys

`?` opens the key help on any screen; any key closes it. `Ctrl-C` quits the
terminal build from anywhere.

### Setup

| Key | Action |
|-----|--------|
| `↑`/`↓`, `j`/`k`, `Tab` | Move between Level, Theme, Adult, Seed, Start |
| `←`/`→`, `h`/`l` | Change the selected value |
| digits, `Backspace` | Edit the seed |
| `Enter` | Next row, or start on the Start row |
| `Esc` | Quit |

### Inbox

| Key | Action |
|-----|--------|
| `j`/`k`, arrows | Move between threads |
| `Enter` | Open the thread |
| `n` | Notepad |
| `a` | Accuse |
| `w` | Wait for mail |
| `q` | Quit |

### Thread

| Key | Action |
|-----|--------|
| `j`/`k`, arrows | Scroll |
| `c` | Compose a question (suspect threads only, not silenced) |
| `Esc` | Back to inbox |

### Compose

| Key | Action |
|-----|--------|
| `j`/`k`, arrows | Move through the question menu |
| `Enter` | Send |
| `Esc` | Cancel |

The menu offers the suspect's own facts, each other suspect, each category, and
a press on their last answer once they have replied.

### Notepad

| Key | Action |
|-----|--------|
| `hjkl`, arrows | Move the grid cursor |
| `Space` / `Enter` | Cycle unknown → no → yes |
| `x` / `o` | Mark no / yes |
| `Backspace` | Clear the cell |
| `Tab` | Switch between the grid and free notes |
| `Esc` | Back to inbox |

In the notes pane, typing edits the text, `Enter` adds a newline, and `Tab`
returns to the grid.

### Accuse

| Key | Action |
|-----|--------|
| `j`/`k`, arrows | Move |
| `Enter` | Next step (suspect, then fact) |
| `y` / `n` | Confirm or go back on the final step |
| `Esc` | Back a step, or back to inbox from the first |

## Building for the web

```sh
cargo install trunk
cd casefile && trunk build --release   # static site in casefile/dist
```
