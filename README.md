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

## License

MIT OR Apache-2.0

## Related Projects

- [Alacritty](https://github.com/alacritty/alacritty) - GPU-accelerated terminal
- [Kitty](https://github.com/kovidgoyal/kitty) - Feature-rich GPU terminal  
- [WezTerm](https://wezfurlong.org/wezterm/) - Lua-configurable terminal
