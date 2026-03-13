# Warp FOSS Clone

A free and open-source terminal emulator inspired by [Warp](https://warp.dev/), built with Go and Bubble Tea TUI.

> ⚡ **Status:** Early development. Basic TUI with shell execution works, AI integration stubbed.

## Features

- 🎨 **Warp-inspired dark theme** with Lip Gloss styling
- 🖥️ **Bubble Tea TUI** - clean, responsive terminal interface
- 🤖 **AI integration** - BYOK, stubbed and ready for your API key
- 📦 **Command block grouping** - each command + output in styled blocks
- ⚡ **Fast, written in Go** - single binary, no runtime dependencies

## Stack

- **Go 1.21+**
- [github.com/charmbracelet/bubbletea](https://github.com/charmbracelet/bubbletea) — TUI framework
- [github.com/charmbracelet/lipgloss](https://github.com/charmbracelet/lipgloss) — styling/theming
- [github.com/charmbracelet/bubbles](https://github.com/charmbracelet/bubbles) — textinput, viewport, spinner

## Quick Start

```bash
# Clone the repo
git clone https://github.com/luinbytes/warp-foss-clone.git
cd warp-foss-clone

# Run
go run .

# Build
go build -o warp-clone .
```

## Keybindings

| Keybinding | Action |
|------------|--------|
| `Tab` | Toggle AI mode |
| `Enter` | Execute command / Submit AI prompt |
| `Esc` | Exit AI mode |
| `Ctrl+C` | Quit (or exit AI mode) |

## AI Integration

AI integration is stubbed and ready to wire up. Look for `stubAICall` in `main.go`:

```go
func (m *Model) stubAICall(prompt string) tea.Cmd {
    return func() tea.Msg {
        // TODO: Wire up actual AI API here
        // Example: OpenAI, Anthropic, Ollama, etc.
        response := fmt.Sprintf("AI Response to: %q", prompt)
        return AIResponseMsg{Response: response}
    }
}
```

### Supported Providers (TODO)

- [ ] OpenAI
- [ ] Anthropic Claude
- [ ] Ollama (local)

## Terminal Execution

Commands are executed using `exec.Command` with platform-appropriate shells:
- Unix systems: `sh -c "command"`
- Windows: `cmd /c "command"`

Execution happens asynchronously via `executeCommand()` in `main.go`, returning results as `CommandExecMsg` messages that are appended to the command history.

## Project Structure

```
warp-foss-clone/
├── main.go          # Entry point + Bubble Tea app
├── go.mod           # Go module with dependencies
├── README.md        # This file
├── CONTRIBUTING.md  # Contribution guidelines
└── .github/         # GitHub templates
```

## Roadmap

- [x] Shell command execution (async, platform-aware)
- [ ] Wire up AI API (configurable provider)
- [ ] Scrollback buffer
- [ ] Tab completion
- [ ] Command history
- [ ] Split panes
- [ ] Tab management
- [ ] Configuration file (~/.warp-clone/config.yaml)
- [ ] Theme customization

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Inspired by [Warp](https://warp.dev/)
- Built with [Bubble Tea](https://github.com/charmbracelet/bubbletea) by [Charm](https://charm.sh/)
- Theme based on [Tokyo Night](https://github.com/enkia/tokyo-night-vscode-theme)
