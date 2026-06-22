---
status: Draft
date: 2026-06-22
deciders:
  - aaronsb
  - claude
related:
  - ADR-007
  - ADR-100
---

# ADR-103: friendly change-detail diff in the CLI

## Context

ADR-100 specced a read-only "change-detail" diff view — the changed file with
the edited line-area in context and inline `+added` / `-removed` — but bound it
to the TUI projection (`syntect` highlighting, `similar` intra-line spans, `gix`,
Ratatui spans). ADR-007 then retired the TUI and projection, which superseded
ADR-100. The *target experience* in that ADR (the human's words — "present the
changed file, the general line-area being edited, with +added / -removed shown
inline; read-only") was never delivered, because its only specced home was
removed.

Meanwhile the CLI's `diff` exposed detail by piping raw `git diff --color=always`,
which carries git's plumbing on every file (`diff --git`, `index`, `--- a/`,
`+++ b/`) and looks nothing like the rest of the tool. The goal from ADR-100
still stands; only its mechanism and surface are now wrong.

## Decision

Deliver ADR-100's read-only change-detail experience in the CLI, with a
mechanism sized to the CLI's scope (ADR-007): render git's unified diff
ourselves rather than reconstruct the TUI stack.

- `diff` keeps a stat summary by default; `--details` selects the friendly
  detailed view; `--git` renders that detail as native `git diff` output
  instead (an escape hatch, and what `--details` used to do). `--git` implies
  `--details`.
- The friendly view parses the unified diff into rows, resolves a single
  line-number gutter (old number for removals, new number otherwise), and draws
  a muted red/green background band behind removed/added rows while leaving the
  foreground text untouched. Git's plumbing header lines are dropped in favor of
  a tool-styled per-file header with a `+added / -removed` stat. Context lines
  carry git's surrounding lines so the edit's location is visible.
- Presentation degrades to plain, uncolored, marker-only text when stdout is not
  a terminal, consistent with the rest of the CLI's output gating.

Deliberately *not* carried over from ADR-100: syntax highlighting (`syntect`),
intra-line span diffing (`similar`), and a filesystem watcher (`notify`). The
view renders a diff that already exists on demand; it does not watch, and it
does not recolor code.

## Consequences

### Positive

- The change-detail experience ADR-100 described now exists, on the surface that
  survived ADR-007, with zero new dependencies — git produces the diff, we only
  render it.
- `diff` output looks like the tool (line numbers, bands, a stat header) instead
  of raw git plumbing, while `--git` preserves the exact native view for anyone
  who wants it or is piping into other git tooling.

### Negative

- A hand-written unified-diff parser is custom glue with its own edge cases
  (renames, binary files, mode-only changes, very long lines) that git's own
  renderer handled for us.
- No syntax highlighting or intra-line emphasis, so a dense single-character
  change is less pinpointed than ADR-100's `similar`-based span view would have
  been. Accepted for the dependency-free, CLI-sized surface.

### Neutral

- The 256-color band palette assumes a terminal that supports it; the plain
  fallback covers non-terminals, but a 16-color-only terminal would see the
  bands approximated by its palette.
- Intra-line and syntax-aware emphasis remain available as a later enhancement
  if the need arises, without changing the command surface.

## Alternatives Considered

- **Keep piping raw `git diff --color=always`.** The status quo. Rejected: it
  never delivers ADR-100's tool-native experience; it leaks git plumbing and
  carries no line-number gutter. Preserved as the `--git` escape hatch rather
  than the default.
- **Re-create ADR-100's `syntect` + `similar` stack in the CLI.** Rejected:
  pulls heavy highlighting/diffing dependencies into a tool ADR-007 deliberately
  slimmed; the marginal precision does not justify the weight for short config
  diffs.
