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

### Windows Binary Crash (Stack Overflow)

**Status:** Unresolved - affecting cross-compiled Windows builds only

**Symptoms:**
The Windows binary crashes immediately on startup with:
```
thread 'main' has overflowed its stack
```

**Root Cause:**
This occurs in `winit 0.30`'s `EventLoop::new()` initialization (line 1859 of `src/main.rs`). The issue is specific to cross-compilation for Windows using the GNU toolchain (`x86_64-pc-windows-gnu`). Windows' default 1MB stack size is exceeded by deep Windows API call chains (RegisterClassExW, CreateWindowExW, COM initialization) during window class registration.

**Affected Builds:**
- Windows cross-compiled from Linux with `x86_64-pc-windows-gnu`
- Linux builds are unaffected
- Native Windows builds (not tested yet)

**Attempted Workarounds (Unsuccessful):**
1. Cached PTY initialization with `OnceLock` - reduced PTY spawning overhead but didn't address the root cause
2. `.stack` section directive in main.rs - not supported by GNU toolchain
3. Linker arguments via `.cargo/config.toml` (`-Wl,--stack,8388608`) - didn't take effect
4. MSVC toolchain (`x86_64-pc-windows-msvc`) - requires native Windows build environment with Visual Studio

**Potential Solutions:**
- Build on Windows with MSVC toolchain and Visual Studio
- Downgrade to an older winit version with less stack usage
- Wait for upstream fix in winit 0.31+
- Explore alternative windowing libraries (miniquartz, sdl2)

**Testing:**
To test on Windows, use the Linux build via WSL or build natively with MSVC. The Windows GUI binary is currently non-functional.

## License

MIT OR Apache-2.0

## Related Projects

- [Alacritty](https://github.com/alacritty/alacritty) - GPU-accelerated terminal
- [Kitty](https://github.com/kovidgoyal/kitty) - Feature-rich GPU terminal  
- [WezTerm](https://wezfurlong.org/wezterm/) - Lua-configurable terminal
