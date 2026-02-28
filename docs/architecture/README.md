# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the Warp FOSS Clone project.

## What is an ADR?

An ADR is a document that captures an important architectural decision made along with its context and consequences. We use ADRs to:

- Document why decisions were made
- Provide context for future contributors
- Avoid repeating past discussions
- Create a decision history

## ADR Format

Each ADR follows this structure:

- **Title**: A short noun phrase describing the decision
- **Status**: Proposed, Accepted, Deprecated, Superseded
- **Context**: The issue motivating this decision
- **Decision**: The change being proposed or made
- **Consequences**: What becomes easier or harder as a result

## Index

- [ADR-000: Use ADRs for architecture decisions](000-use-adrs.md)
- [ADR-001: Use wgpu for GPU rendering](001-wgpu-rendering.md)
- [ADR-002: Use vte-rs for terminal emulation](002-vte-terminal.md)
- [ADR-003: WASM plugin system](003-wasm-plugins.md)

## Creating a New ADR

1. Copy `template.md` to `NNNN-short-title.md`
2. Fill in the sections
3. Submit as part of a PR
4. Update this index

## References

- [Michael Nygard's ADRs](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [GitHub's ADRs](https://github.blog/2020-08-13-why-write-adrs/)
