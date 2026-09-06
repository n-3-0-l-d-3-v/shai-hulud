# impossible-artifact — THE ARTIFACT

> A minimal computational environment embedded inside a format normally considered data.

Part of **[The Impossible Computer](https://github.com/n-3-0-l-d-3-v/impossible-computer)** — a constrained computing
ecosystem built by removing assumptions ordinary computers depend on. This
repository is developed standalone and mirrored into the combined ecosystem
repo commit-for-commit.

## Status

**Phase 9 — STRETCH**

See [tickets/](tickets/) for the live phase-by-phase ticket board and
[docs/design/](docs/design/) for constraints, invariants and architecture
decision records.

## The constraint

A standard, ordinary-looking data file (PDF/SVG/font/etc.) becomes the container for a real computation environment: loader, VM, and shell.

## What the constraint forces

Polyglot file construction, a tiny trusted loader, and running impossible-machine's VM from inside a non-executable substrate.

## Research question

> At what point does a data representation become a computational substrate?

## Sibling repositories

- [impossible-machine](https://github.com/n-3-0-l-d-3-v/impossible-machine) — THE MACHINE (ACTIVE)
- [impossible-language](https://github.com/n-3-0-l-d-3-v/impossible-language) — THE LANGUAGE (QUEUED)
- [impossible-kernel](https://github.com/n-3-0-l-d-3-v/impossible-kernel) — THE KERNEL (QUEUED)
- [impossible-vault](https://github.com/n-3-0-l-d-3-v/impossible-vault) — THE VAULT (QUEUED)
- [impossible-database](https://github.com/n-3-0-l-d-3-v/impossible-database) — THE DATABASE (QUEUED)
- [impossible-wire](https://github.com/n-3-0-l-d-3-v/impossible-wire) — THE WIRE (QUEUED)
- [impossible-colony](https://github.com/n-3-0-l-d-3-v/impossible-colony) — THE COLONY (QUEUED)
- [impossible-history](https://github.com/n-3-0-l-d-3-v/impossible-history) — THE HISTORY (QUEUED)

## Development

This is a real, tested, benchmarked systems component — not a demo. See
[docs/DEFINITION_OF_DONE.md](docs/DEFINITION_OF_DONE.md) for the acceptance
bar every piece of this repo must clear before it is considered complete.

```bash
cargo build
cargo test
cargo bench
```
