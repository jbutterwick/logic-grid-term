# logic-grid-term

Logic grid puzzles in your terminal. `logicgrid` generates and solves puzzles;
`logictui` plays them.

## Install

```sh
cargo install --path logictui
```

Or run from the workspace: `cargo run -p logictui -- [args]`.

## Usage

```sh
logictui                                  # home screen: pick size, difficulty, theme
logictui --size 4x4 --difficulty hard     # play immediately (random seed)
logictui --size 5x5 --difficulty easy --seed 42
logictui --theme "Haunted Hotel"           # pick a theme (see --list-themes)
logictui puzzle.json                      # play a saved puzzle
logictui --size 4x4 --difficulty medium --save puzzle.json   # write puzzle JSON and exit
```

| Flag | Meaning |
|------|---------|
| `FILE` | Saved puzzle JSON to play |
| `--size CxI` | Categories x items: `3x3`, `3x4`, `4x4`, `4x5`, `5x5` (default `4x4`) |
| `--difficulty` | `easy`, `medium`, `hard` (default `medium`) |
| `--seed N` | Generator seed, shown in the status bar so a puzzle can be replayed |
| `--theme NAME` | Theme to use (case-insensitive); random if omitted |
| `--list-themes` | Print the built-in theme names and exit |
| `--save PATH` | Write the puzzle (not progress) to PATH and exit |

Progress (marks and struck clues) autosaves to the platform data dir
(`~/Library/Application Support/logictui` on macOS, `~/.local/share/logictui` on Linux)
keyed by seed, size, and difficulty, and restores when the same puzzle is reopened.
The home screen lists recent puzzles with their solve status.

## Controls

| Key | Action |
|-----|--------|
| Arrows / `hjkl` | Move cursor |
| `Space` / `Enter` | Cycle Unknown → No → Yes |
| `x` / `o` | Mark No / Yes |
| `Backspace` | Clear cell |
| `u` | Undo |
| `Tab` / `n` / `p` | Select next / previous clue |
| `s` | Strike selected clue |
| `1`–`9` | Strike clue N |
| `c` | Check: highlight marks that contradict the solution |
| `i` | Show the story intro again |
| `?` | Help overlay |
| `w` | Write puzzle JSON to the current directory |
| `Esc` | Back to home screen |
| `q` | Quit |

Placing a Yes marks the rest of its row and column in that sub-grid as No.
A 4x4 puzzle fits an 80x24 terminal; 5x5 fits 120x40.

## casefile

`casefile` is an email-inbox detective game built on the same puzzles: interrogate
suspects, spot the liars, and accuse. Play it in the browser at
<https://jbutterwick.github.io/logic-grid-term/> or `cargo install casefile`.
See [casefile/README.md](casefile/README.md) for flags and keys.
