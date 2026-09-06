# SHAI-HULUD — THE ARTIFACT

> A minimal computational environment embedded inside a format normally considered data.

## Why "SHAI-HULUD"

The sandworm — from the surface, Arrakis looks like empty, inert desert. Underneath, it conceals an enormous, powerful, living machine. That is the whole premise of the artifact component: a file that looks like plain, non-executable data while secretly being a computer.

Part of **[ARRAKIS](https://github.com/n-3-0-l-d-3-v/arrakis)** — a constrained computing
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

Polyglot file construction, a tiny trusted loader, and running mentat's VM from inside a non-executable substrate.

## Research question

> At what point does a data representation become a computational substrate?

## Sibling repositories

- [mentat](https://github.com/n-3-0-l-d-3-v/mentat) — THE MACHINE (COMPLETE)
- [chakobsa](https://github.com/n-3-0-l-d-3-v/chakobsa) — THE LANGUAGE (QUEUED)
- [muaddib](https://github.com/n-3-0-l-d-3-v/muaddib) — THE KERNEL (QUEUED)
- [sietch](https://github.com/n-3-0-l-d-3-v/sietch) — THE VAULT (ACTIVE)
- [choam](https://github.com/n-3-0-l-d-3-v/choam) — THE DATABASE (QUEUED)
- [distrans](https://github.com/n-3-0-l-d-3-v/distrans) — THE WIRE (QUEUED)
- [landsraad](https://github.com/n-3-0-l-d-3-v/landsraad) — THE COLONY (QUEUED)
- [ghola](https://github.com/n-3-0-l-d-3-v/ghola) — THE HISTORY (QUEUED)

## Development

This is a real, tested, benchmarked systems component — not a demo. See
[docs/DEFINITION_OF_DONE.md](docs/DEFINITION_OF_DONE.md) for the acceptance
bar every piece of this repo must clear before it is considered complete.

```bash
cargo build
cargo test
cargo bench
```
