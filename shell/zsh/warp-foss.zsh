# Warp FOSS shell integration for Zsh
# Source this file in your .zshrc

if [[ "$TERM_PROGRAM" == "warp-foss" ]]; then
    # OSC 7: Report current directory
    __warp_foss_osc7() {
        printf '\e]7;file://%s%s\e\\' "$HOSTNAME" "$PWD"
    }

    # Hook into chpwd for directory changes
    chpwd_functions+=(__warp_foss_osc7)

    # Initial directory report
    __warp_foss_osc7
fi
