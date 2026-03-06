# Warp FOSS shell integration for Bash
# Source this file in your .bashrc

if [[ "$TERM_PROGRAM" == "warp-foss" ]]; then
    # OSC 7: Report current directory
    __warp_foss_osc7() {
        printf '\e]7;file://%s%s\e\\' "$HOSTNAME" "$PWD"
    }

    # Hook into prompt command
    if [[ -z "$PROMPT_COMMAND" ]]; then
        PROMPT_COMMAND="__warp_foss_osc7"
    else
        PROMPT_COMMAND="__warp_foss_osc7;${PROMPT_COMMAND#;}"
    fi

    # Also set on cd
    cd() {
        builtin cd "$@" && __warp_foss_osc7
    }
fi
