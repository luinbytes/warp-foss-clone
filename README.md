# Warp FOSS Clone

A free and open-source clone of [Warp](https://warp.dev/) terminal with AI integration.

## Features (Planned)

- 🖥️ GPU-accelerated rendering (wgpu)
- 🤖 BYOK AI integration (OpenAI, Anthropic, Ollama)
- 🔌 WASM plugin system
- 📦 Block-based output
- ⚡ Fast, written in Rust

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust |
| Rendering | wgpu + winit |
| Terminal | vte-rs |
| Async | tokio |
| Plugins | wasmtime (WASM) |

## Architecture

```
┌─────────────────────────────────────┐
│           UI Layer (wgpu)           │
└─────────────────────────────────────┘
                  │
┌─────────────────────────────────────┐
│        Terminal Core (vte-rs)       │
└─────────────────────────────────────┘
                  │
┌─────────────────────────────────────┐
│         AI Integration Layer        │
└─────────────────────────────────────┘
                  │
┌─────────────────────────────────────┐
│          Plugin System (WASM)       │
└─────────────────────────────────────┘
```

## Status

🚧 Early development - research and architecture phase.

## Known Issues

### Windows Binary - Stack Overflow Fix Applied ✅

**Status:** Fixed - needs testing on actual Windows

**What was wrong:**
The Windows binary was crashing on startup with `thread 'main' has overflowed its stack` due to:
1. Large stack-allocated arrays in text rendering (4KB ANSI palette per call)
2. Stack-allocated PTY read buffer (4KB)

**The Fix:**
1. ANSI palette now uses `LazyLock` for heap allocation (see `src/ui/text.rs`)
2. PTY buffer changed to `vec![0u8; 4096]` heap allocation (see `src/main.rs`)

See `STACK_OVERFLOW_FIX.md` for full details.

**Build Status:**
- ✅ Linux builds work
- ✅ Windows cross-compile (`x86_64-pc-windows-gnu`) succeeds
- ❓ Needs testing on actual Windows to confirm fix

**To build for Windows:**
```bash
cargo build --release --target x86_64-pc-windows-gnu
```

## License

MIT OR Apache-2.0

## Related Projects

- [Alacritty](https://github.com/alacritty/alacritty) - GPU-accelerated terminal
- [Kitty](https://github.com/kovidgoyal/kitty) - Feature-rich GPU terminal  
- [WezTerm](https://wezfurlong.org/wezterm/) - Lua-configurable terminal
