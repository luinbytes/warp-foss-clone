#!/bin/bash
# Warp FOSS Shell Integration Installer
# Detects your shell and provides installation instructions

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║         Warp FOSS Shell Integration Installer             ║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo
}

detect_shell() {
    local shell_name=""
    local shell_version=""

    # Detect current shell
    if [[ -n "$BASH_VERSION" ]]; then
        shell_name="bash"
        shell_version="$BASH_VERSION"
    elif [[ -n "$ZSH_VERSION" ]]; then
        shell_name="zsh"
        shell_version="$ZSH_VERSION"
    elif [[ -n "$FISH_VERSION" ]]; then
        shell_name="fish"
        shell_version="$FISH_VERSION"
    else
        # Try to detect from $SHELL
        case "$SHELL" in
            */bash)
                shell_name="bash"
                ;;
            */zsh)
                shell_name="zsh"
                ;;
            */fish)
                shell_name="fish"
                ;;
            *)
                shell_name="unknown"
                ;;
        esac
    fi

    echo "$shell_name"
}

get_config_file() {
    local shell="$1"
    case "$shell" in
        bash)
            echo "$HOME/.bashrc"
            ;;
        zsh)
            echo "$HOME/.zshrc"
            ;;
        fish)
            echo "$HOME/.config/fish/config.fish"
            ;;
        *)
            echo ""
            ;;
    esac
}

get_integration_file() {
    local shell="$1"
    case "$shell" in
        bash)
            echo "$SCRIPT_DIR/bash/warp-foss.sh"
            ;;
        zsh)
            echo "$SCRIPT_DIR/zsh/warp-foss.zsh"
            ;;
        fish)
            echo "$SCRIPT_DIR/fish/warp-foss.fish"
            ;;
        *)
            echo ""
            ;;
    esac
}

install_bash() {
    local config_file="$HOME/.bashrc"
    local integration_file="$SCRIPT_DIR/bash/warp-foss.sh"
    local source_line="source \"$integration_file\""

    echo -e "${YELLOW}Installing for Bash...${NC}"

    # Check if already installed
    if grep -q "warp-foss.sh" "$config_file" 2>/dev/null; then
        echo -e "${GREEN}✓ Already installed in $config_file${NC}"
        return 0
    fi

    # Add source line to config
    echo "" >> "$config_file"
    echo "# Warp FOSS shell integration" >> "$config_file"
    echo "$source_line" >> "$config_file"

    echo -e "${GREEN}✓ Added to $config_file${NC}"
    echo -e "  Restart your shell or run: ${YELLOW}source $config_file${NC}"
}

install_zsh() {
    local config_file="$HOME/.zshrc"
    local integration_file="$SCRIPT_DIR/zsh/warp-foss.zsh"
    local source_line="source \"$integration_file\""

    echo -e "${YELLOW}Installing for Zsh...${NC}"

    # Check if already installed
    if grep -q "warp-foss.zsh" "$config_file" 2>/dev/null; then
        echo -e "${GREEN}✓ Already installed in $config_file${NC}"
        return 0
    fi

    # Add source line to config
    echo "" >> "$config_file"
    echo "# Warp FOSS shell integration" >> "$config_file"
    echo "$source_line" >> "$config_file"

    echo -e "${GREEN}✓ Added to $config_file${NC}"
    echo -e "  Restart your shell or run: ${YELLOW}source $config_file${NC}"
}

install_fish() {
    local config_dir="$HOME/.config/fish/conf.d"
    local integration_file="$SCRIPT_DIR/fish/warp-foss.fish"
    local target_file="$config_dir/warp-foss.fish"

    echo -e "${YELLOW}Installing for Fish...${NC}"

    # Create conf.d directory if it doesn't exist
    mkdir -p "$config_dir"

    # Check if already installed
    if [[ -f "$target_file" ]]; then
        echo -e "${GREEN}✓ Already installed at $target_file${NC}"
        return 0
    fi

    # Create symlink or copy
    ln -s "$integration_file" "$target_file" 2>/dev/null || cp "$integration_file" "$target_file"

    echo -e "${GREEN}✓ Installed to $target_file${NC}"
    echo -e "  Restart your shell or run: ${YELLOW}source $target_file${NC}"
}

show_manual_instructions() {
    local shell="$1"
    local config_file
    local integration_file

    config_file=$(get_config_file "$shell")
    integration_file=$(get_integration_file "$shell")

    echo
    echo -e "${BLUE}Manual Installation for ${shell^}:${NC}"
    echo -e "  1. Add this line to ${YELLOW}$config_file${NC}:"
    echo
    echo -e "     ${GREEN}source \"$integration_file\"${NC}"
    echo
    echo -e "  2. Restart your shell or run:"
    echo -e "     ${YELLOW}source $config_file${NC}"
    echo
}

print_header

# Detect current shell
CURRENT_SHELL=$(detect_shell)
echo -e "Detected shell: ${GREEN}$CURRENT_SHELL${NC}"
echo

# Check if running inside warp-foss
if [[ "$TERM_PROGRAM" != "warp-foss" ]]; then
    echo -e "${YELLOW}Note: You're not currently running inside Warp FOSS terminal.${NC}"
    echo -e "The integration will activate when you run Warp FOSS."
    echo
fi

# Parse arguments
if [[ "$1" == "--install" ]] || [[ "$1" == "-i" ]]; then
    case "$CURRENT_SHELL" in
        bash)
            install_bash
            ;;
        zsh)
            install_zsh
            ;;
        fish)
            install_fish
            ;;
        *)
            echo -e "${RED}Error: Could not auto-install for shell '$CURRENT_SHELL'${NC}"
            show_manual_instructions "$CURRENT_SHELL"
            exit 1
            ;;
    esac
elif [[ "$1" == "--help" ]] || [[ "$1" == "-h" ]]; then
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -i, --install    Automatically install for current shell"
    echo "  -h, --help       Show this help message"
    echo
    echo "Without options, shows manual installation instructions."
    exit 0
else
    # Show manual instructions
    show_manual_instructions "$CURRENT_SHELL"

    echo -e "${BLUE}Quick Install:${NC}"
    echo -e "  Run ${YELLOW}$0 --install${NC} to automatically add the integration."
    echo
    echo -e "${BLUE}Supported Shells:${NC}"
    echo -e "  ✅ Bash 4.0+"
    echo -e "  ✅ Zsh 5.0+"
    echo -e "  ✅ Fish 3.0+"
fi
