# 2. Use vte-rs for Terminal Emulation

Date: 2026-02-28

## Status

Accepted

## Context

We need to parse and interpret terminal escape sequences from PTY output. This requires:

1. **Correctness**: Must handle all standard escape sequences properly
2. **Performance**: High-throughput parsing for fast terminal output
3. **Maintainability**: Shouldn't reinvent the wheel
4. **Compatibility**: Must work with standard Unix tools and shells

Options considered:
- **vte-rs**: Rust bindings for GNOME's VTE library (parser only)
- **Write custom parser**: Full control but high effort
- **alacritty's parser**: Fork Alacritty's escape sequence parser
- **termwiz**: Terminal library from WezTerm

## Decision

We will use **vte-rs** for terminal escape sequence parsing.

vte-rs provides:
- Battle-tested parser from GNOME's VTE
- Zero-copy parsing for performance
- Simple `Perform` trait for handling sequences
- Good Rust idioms
- Active maintenance

## Consequences

**Positive:**
- Correct handling of complex escape sequences
- Excellent performance (zero-copy)
- Well-tested in production (GNOME Terminal, etc.)
- Simple API via `Perform` trait
- No C dependencies (pure Rust port)

**Negative:**
- Less control than custom parser
- Must implement `Perform` trait for all sequences
- May parse sequences we don't need to support

**Neutral:**
- Parser is separate from grid state management
- We still implement our own grid buffer and state

## Implementation Notes

- `TerminalParser` wraps `vte::Parser`
- `Perform` trait implementation in `src/terminal/parser.rs`
- Grid state managed separately in `TerminalGrid`
- Parser updates grid directly for performance

## Unsupported Sequences

For simplicity, we initially ignore some sequences:
- Complex cursor shape changes
- Some DEC private modes
- Hyperlinks (OSC 8)
- Some lesser-used SGR sequences

These can be added as needed.

## References

- [vte-rs repository](https://github.com/alacritty/vte)
- [VTE documentation](https://gnome.pages.gitlab.gnome.org/vte/)
- [ECMA-48 standard](https://www.ecma-international.org/publications-and-standards/standards/ecma-48/)
