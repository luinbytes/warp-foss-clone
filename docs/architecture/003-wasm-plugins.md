# 3. WASM Plugin System

Date: 2026-02-28

## Status

Proposed

## Context

We want to make the terminal extensible via plugins. Requirements:

1. **Safety**: Plugins shouldn't crash the terminal
2. **Security**: Plugins should be sandboxed
3. **Flexibility**: Support various plugin types
4. **Performance**: Minimal overhead
5. **Accessibility**: Easy to write plugins

Options considered:
- **Dynamic libraries (.so/.dylib/.dll)**: Fast but unsafe, platform-specific
- **Lua scripting**: Common in terminals (WezTerm), but another language
- **Python scripting**: Popular but heavy runtime
- **WASM (WebAssembly)**: Safe, portable, fast, growing ecosystem
- **JavaScript/QuickJS**: Familiar but larger runtime

## Decision

We will use **WASM (WebAssembly)** with wasmtime for the plugin system.

WASM provides:
- **Safety**: Sandboxed by design, memory-safe
- **Portability**: Platform-independent bytecode
- **Performance**: Near-native speed with JIT compilation
- **Language support**: Can be written in Rust, C, C++, AssemblyScript, etc.
- **Security**: Fine-grained capability control
- **Growing ecosystem**: wgpu, wasm-bindgen, etc.

## Consequences

**Positive:**
- Plugins can't crash the main process
- Cross-platform (same plugin works everywhere)
- Can write plugins in multiple languages
- Good performance with wasmtime
- Secure by default (explicit capabilities)

**Negative:**
- WASM overhead (though minimal with wasmtime)
- Complexity of plugin API design
- WASI limitations (limited system access)
- Smaller ecosystem than JavaScript/Python

**Neutral:**
- Plugin authors need WASM knowledge
- Must carefully design plugin API
- Plugin loading adds startup time

## Plugin Capabilities

Planned plugin capabilities:

1. **Command Transformers**: Modify commands before execution
2. **Output Processors**: Transform/analyze terminal output
3. **UI Extensions**: Add custom UI elements
4. **Theme Providers**: Create color schemes
5. **Keybinding Extensions**: Custom keyboard shortcuts
6. **Status Bar Widgets**: Custom status bar content

## Implementation Plan

### Phase 1: Foundation
- Set up wasmtime runtime
- Define plugin API/interface
- Create simple example plugin
- Basic plugin loading/unloading

### Phase 2: Core Capabilities
- Command transformation API
- Output processing API
- Plugin configuration
- Error handling

### Phase 3: Advanced Features
- UI extension points
- Plugin marketplace/directory
- Plugin hot-reloading
- Performance profiling

## Security Model

- Plugins run in WASM sandbox
- Explicit capability grants (network, filesystem, etc.)
- Plugin permissions requested at install
- Sandboxed filesystem access
- No direct system calls

## Example Plugin

```rust
// A simple command transformer plugin
use warp_foss_plugin::*;

#[plugin]
pub fn transform_command(cmd: &str) -> String {
    if cmd.starts_with("git ") {
        format!("echo 'Running git:' && {}", cmd)
    } else {
        cmd.to_string()
    }
}
```

## References

- [wasmtime documentation](https://docs.wasmtime.dev/)
- [WASI specification](https://wasi.dev/)
- [Extism plugin system](https://extism.org/)
- [WezTerm plugin docs](https://wezfurlong.org/wezterm/plugins.html)
