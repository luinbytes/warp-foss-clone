# Warp FOSS Clone

A free and open-source clone of [Warp](https://warp.dev/) terminal with AI integration.

> ⚡ **Status:** Early development. Text rendering works, terminal emulation functional. See [Current Status](#current-status) below.

<!-- Screenshots will be added once UI is more polished -->
<!--
## Screenshots

![Split panes](screenshots/split-panes.png)
![AI command palette](screenshots/ai-palette.png)
![Search functionality](screenshots/search.png)
-->

## Current Status

**Working Features:**
- ✅ GPU-accelerated text rendering (wgpu)
- ✅ Terminal emulation with VTE parser
- ✅ PTY I/O (spawn shell, read/write)
- ✅ Split panes with layout management
- ✅ AI command palette (Ctrl+Space)
- ✅ Search functionality (Ctrl+Shift+F)
- ✅ Copy/paste with clipboard support
- ✅ Status bar with git integration
- ✅ Multiple AI providers (OpenAI, Anthropic, Ollama)

**In Progress:**
- 🚧 Shell integration for directory tracking
- 🚧 Scrollback buffer
- 🚧 Configuration system
- 🚧 WASM plugin system

## Features (Full Vision)

- 🖥️ GPU-accelerated rendering (wgpu)
- 🤖 BYOK AI integration (OpenAI, Anthropic, Ollama)
- 🔌 WASM plugin system
- 📦 Block-based output
- ⚡ Fast, written in Rust
- 🎨 Customizable themes and keybindings
- 🔍 Advanced search with regex support
- 📋 Smart copy with formatting options

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
│  - Text rendering                   │
│  - Layout management                │
│  - Input handling                   │
│  - AI command palette               │
│  - Status bar                       │
└─────────────────────────────────────┘
                  │
┌─────────────────────────────────────┐
│        Terminal Core (vte-rs)       │
│  - PTY session management           │
│  - Grid buffer (cells, colors)      │
│  - VTE parser (escape sequences)    │
│  - Selection & clipboard            │
└─────────────────────────────────────┘
                  │
┌─────────────────────────────────────┐
│         AI Integration Layer        │
│  - OpenAI provider                  │
│  - Anthropic provider               │
│  - Ollama (local) provider          │
│  - BYOK (bring your own key)        │
└─────────────────────────────────────┘
                  │
┌─────────────────────────────────────┐
│          Plugin System (WASM)       │
│  - Custom commands                  │
│  - Output transformers              │
│  - UI extensions                    │
└─────────────────────────────────────┘
```

### Key Modules

- `src/main.rs` - Application entry point and event loop
- `src/ui/` - GPU rendering, input handling, overlays
- `src/terminal/` - PTY management, grid buffer, VTE parser
- `src/ai/` - AI provider integrations
- `src/config/` - Configuration management
- `src/plugin/` - WASM plugin system (planned)
- `src/search/` - Search functionality

## Development Setup

### Prerequisites

- Rust 1.70+ (uses 2021 edition)
- System dependencies for wgpu (see [wgpu docs](https://github.com/gfx-rs/wgpu))

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install build-essential cmake pkg-config libfreetype6-dev

# Fedora
sudo dnf install cmake freetype-devel
```

**macOS:**
```bash
# Xcode command line tools
xcode-select --install
```

**Windows:**
- Visual Studio Build Tools 2019+ with C++ development tools
- Or use cross-compilation from Linux (see below)

### Building

```bash
# Clone the repo
git clone https://github.com/luinbytes/warp-foss-clone.git
cd warp-foss-clone

# Build
cargo build --release

# Run
cargo run --release
```

### Cross-Compilation for Windows

```bash
# Install target
rustup target add x86_64-pc-windows-gnu

# Build
cargo build --release --target x86_64-pc-windows-gnu
```

See [STACK_OVERFLOW_FIX.md](STACK_OVERFLOW_FIX.md) for Windows-specific notes.

### Running Tests

```bash
cargo test
```

### AI Configuration

Set environment variables for AI providers:

```bash
# OpenAI
export OPENAI_API_KEY="your-key"

# Anthropic
export ANTHROPIC_API_KEY="your-key"

# Ollama (runs locally, no key needed)
# Ensure Ollama is running on localhost:11434
```

Then use `Ctrl+Space` in the terminal to open the AI command palette.

## Status

🚧 Early development - core features functional, many enhancements planned.

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

## Roadmap

### Phase 1: Core Terminal ✅ (In Progress)
- [x] GPU text rendering
- [x] PTY spawning and I/O
- [x] VTE escape sequence parsing
- [x] Split panes and layout management
- [x] Basic AI integration
- [ ] Shell integration (directory tracking)
- [ ] Scrollback buffer
- [ ] Configuration system

### Phase 2: Enhanced Experience
- [ ] Theme system with presets
- [ ] Custom keybindings
- [ ] Advanced search (regex, case-sensitive)
- [ ] Better error handling and feedback
- [ ] Performance optimizations

### Phase 3: Advanced Features
- [ ] WASM plugin system
- [ ] Block-based output (like Warp)
- [ ] Command autocomplete
- [ ] Session management
- [ ] Remote connection support

### Phase 4: Polish & Distribution
- [ ] Cross-platform packages (deb, rpm, dmg, msi)
- [ ] Auto-update system
- [ ] Documentation website
- [ ] Accessibility features

See [GitHub Issues](https://github.com/luinbytes/warp-foss-clone/issues) for detailed tracking.

## Contributing

Contributions welcome! Here's how to help:

### Development
1. Fork the repo
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Commit with clear messages
6. Push to your fork
7. Open a Pull Request

### Code Style
- Follow standard Rust conventions (`cargo fmt`)
- Run clippy before committing (`cargo clippy`)
- Add tests for new functionality
- Update documentation as needed

### Areas Needing Help
- 🪟 **Windows testing** - Help verify the stack overflow fix
- 🎨 **Theme design** - Create color schemes and themes
- 📝 **Documentation** - Improve docs and examples
- 🧪 **Testing** - Add comprehensive test coverage
- 🐛 **Bug reports** - Report issues with detailed reproduction steps

See [GitHub Issues](https://github.com/luinbytes/warp-foss-clone/issues) for open tasks.

## License

MIT OR Apache-2.0

## Related Projects

- [Alacritty](https://github.com/alacritty/alacritty) - GPU-accelerated terminal
- [Kitty](https://github.com/kovidgoyal/kitty) - Feature-rich GPU terminal  
- [WezTerm](https://wezfurlong.org/wezterm/) - Lua-configurable terminal
