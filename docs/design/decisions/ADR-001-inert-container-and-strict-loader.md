# ADR-001: Inert data containers + explicit strict loader

## Decision
Machine programs are embedded as **inert data** in ordinary documents (PDF
embedded-file attachment; SVG namespaced `<metadata>` element). Execution
happens only when the `artifact` loader is explicitly invoked. Viewers see a
plain page; the files contain no JavaScript, actions, launch entries, scripts
or event handlers (asserted in tests).

## Loader rules (fail closed)
Exactly one payload; size cap 1 MiB; SHA-256 must match; JSON must parse and
`Program::validate` must pass; run with a memory cap and step budget. The
Machine VM has no host I/O beyond exit and a captured output buffer.

## Why not a viewer-native execution path
PDF Type 4 functions / SVG SMIL can compute, but Type 4 has no loops and
viewer behaviour differs per renderer. Relying on that would be unreliable
and drifts toward viewer exploitation, which is out of scope. Not built.

## Findings and limits
- Invariant tested by property: any single-byte corruption of a container
  yields an error or the identical payload, never a different one. Mutation
  check (hash verification disabled) makes 2 tests fail.
- The PDF checksum key `/IMC_SHA256` is non-standard inside `/Params`;
  viewers ignore it. PDF parsing is deliberately minimal (not a general PDF
  reader): it only accepts files written by this tool's layout.
- A payload containing the literal PDF marker is refused at pack time.
- Integrity only, not authenticity: the hash is inside the file, so it detects
  corruption, not deliberate replacement. Signing is not implemented.
- Not validated with an external PDF tool here; xref offsets are checked by a
  self-test only.
