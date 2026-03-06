# Warp FOSS shell integration for Fish
# Source this file or add to ~/.config/fish/conf.d/

if test "$TERM_PROGRAM" = "warp-foss"
    # OSC 7: Report current directory
    function __warp_foss_osc7 --on-variable PWD
        printf '\e]7;file://%s%s\e\\' $hostname $PWD
    end

    # Initial directory report
    __warp_foss_osc7
end
