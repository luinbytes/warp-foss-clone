# 0. Use Architecture Decision Records

Date: 2026-02-28

## Status

Accepted

## Context

We need to document the architectural decisions made in this project to:

1. Help contributors understand why the code is structured this way
2. Avoid revisiting decisions that have already been made
3. Provide context for future changes
4. Create a historical record of the project's evolution

Without formal documentation, important decisions are lost to time or hidden in commit messages and PR discussions.

## Decision

We will use Architecture Decision Records (ADRs) to document all significant architectural decisions.

Each ADR will be:
- Stored in `docs/architecture/`
- Named `NNNN-short-title.md` where NNNN is a sequential number
- Written in Markdown format
- Committed to the repository

## Consequences

**Positive:**
- Clear record of why decisions were made
- Easy onboarding for new contributors
- Reduced bike-shedding on settled decisions
- Better understanding of project evolution

**Negative:**
- Overhead of writing and maintaining ADRs
- May feel bureaucratic for small decisions

**Neutral:**
- ADRs are not automatically updated; they document the decision at the time it was made
- Superseded ADRs should be marked but not removed

## References

- [Documenting Architecture Decisions](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
