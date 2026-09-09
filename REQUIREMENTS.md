# Requirements

Numbered so plans, briefs, and tests can cite them. MUST = required for v0.1. SHOULD = do if cheap, otherwise defer.

## A. `logicgrid` library (no UI dependencies)

### A1. Puzzle model
- A1.1 MUST: A puzzle has N categories, each with M items (N ≥ 3, M ≥ 3). Category 0 is the anchor (e.g. "Person"); the solution maps every anchor item to exactly one item of every other category, bijectively.
- A1.2 MUST: A `Solution` type that is a bijection per category pair and is checkable for consistency.
- A1.3 MUST: A `Clue` enum with at least these variants, each rendered to English text:
  - `Is(a, b)`: item a is paired with item b.
  - `IsNot(a, b)`: item a is not paired with item b.
  - `Either(a, b, c)`: a is paired with b or c, not both.
  - `Before(a, b, ordered_category)`, `After`, `Adjacent(a, b)`: for one ordered category (numbers, times, positions).
  - `NotSame(a, b)`: two items from different categories are not the same entity.
- A1.4 MUST: `Puzzle { categories, clues, solution }` is serde-serializable to a text format (JSON or TOML) so puzzles can be saved and shared.
- A1.5 MUST: Items and categories carry display names; the library ships a small built-in theme bank (names, colors, pets, times, etc.) sufficient to generate varied puzzles without external data.

### A2. Solver
- A2.1 MUST: A solver that, given categories and clues, returns `Unique(solution)`, `Multiple`, or `None`.
- A2.2 MUST: The solver is complete (finds a solution whenever one exists) on 3×3 through 5×5 puzzles in well under one second.
- A2.3 MUST: Given a partial player grid (Yes/No/Unknown marks), the solver can report which marks contradict the solution. Used for the TUI "check" feature.
- A2.4 SHOULD: The solver reports which deduction techniques it needed (direct, elimination, transitive, ordinal), so difficulty can be measured rather than guessed.

### A3. Generator
- A3.1 MUST: `generate(size, difficulty, seed) -> Puzzle` produces a puzzle with exactly one solution, verified by the solver before return.
- A3.2 MUST: Deterministic: same size, difficulty, and seed always produce the same puzzle.
- A3.3 MUST: Minimal clue set: no clue can be removed without the puzzle losing uniqueness.
- A3.4 MUST: Three difficulty levels (Easy, Medium, Hard) that differ measurably, at minimum in clue count and allowed clue types. Hard must use ordinal and Either clues; Easy may be Is/IsNot only.
- A3.5 MUST: Supports sizes 3×3, 3×4, 4×4, 4×5, 5×5 (categories × items).
- A3.6 MUST: Generation completes in under 2 seconds for 5×5 Hard on a laptop.
- A3.7 MUST: Generated clues never leak the anchor mapping trivially (e.g. no `Is` clue directly stating a full row for Medium/Hard).

### A4. Quality gates
- A4.1 MUST: Property test: 100 random puzzles across all sizes/difficulties each have exactly one solution and are clue-minimal.
- A4.2 MUST: `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` pass. `cargo fmt --all` is clean.
- A4.3 MUST: Public API is documented with `///` on every public item.

## B. `logictui` binary (ratatui)

### B1. Launch and puzzle selection
- B1.1 MUST: `logictui` with no args opens a home screen: choose size and difficulty, press Enter to generate and play.
- B1.2 MUST: `logictui --size 4x4 --difficulty hard [--seed N]` skips the menu and plays immediately.
- B1.3 MUST: `logictui path/to/puzzle.json` loads a saved puzzle.
- B1.4 SHOULD: The seed is shown in-game so a puzzle can be replayed or shared.

### B2. Play screen
- B2.1 MUST: Renders the full triangular logic grid (every category pair as a sub-grid) plus the clue list in a side panel, fitting an 80×24 terminal for 4×4 and scaling up for 5×5.
- B2.2 MUST: Cursor moves with arrows and `hjkl`; a cell cycles Unknown → No → Yes with Space/Enter, or set directly with `x` (No) and `o` (Yes). Backspace clears.
- B2.3 MUST: Placing Yes auto-marks the rest of that row and column in the sub-grid as No (standard logic grid behavior). Removing the Yes does not un-mark those.
- B2.4 MUST: Clues can be toggled struck-through (`s` or number key) to track which are used.
- B2.5 MUST: `c` checks the grid and highlights marks that contradict the solution. `?` shows a help overlay.
- B2.6 MUST: When every anchor item has a Yes in every other category and the grid matches the solution, show a win state (crossword uses confetti; reuse that pattern if trivial, otherwise a banner).
- B2.7 SHOULD: Undo (`u`).
- B2.8 SHOULD: An answer table below/beside the grid that fills in as Yes marks are placed.

### B3. Persistence
- B3.1 MUST: In-progress state (marks, struck clues) autosaves to the platform data dir (`dirs`) keyed by puzzle seed/hash and restores on reopen.
- B3.2 MUST: `w` or `--save path` writes the puzzle (not the progress) to JSON for sharing.
- B3.3 SHOULD: Home screen lists recent/in-progress puzzles with solve status.

### B4. Polish
- B4.1 MUST: `q` quits cleanly and restores the terminal, including on panic.
- B4.2 SHOULD: Theme support in TOML, reusing crosstui's theme.rs approach.
- B4.3 MUST: README documents install, controls, and CLI flags.

## C. Project-wide
- C1 MUST: `logicgrid` never depends on ratatui, crossterm, or clap.
- C2 MUST: All work lands via agent branches merged to `main`; `main` always builds and passes A4.2.
- C3 MUST: Each agent brief cites the requirement IDs it owns and adds tests for them.
