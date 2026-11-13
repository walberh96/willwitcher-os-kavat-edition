-- Made by kavat-theme from the theme's settings; what is changed here is lost.
-- The settings are in ~/.config/kavat/theme.toml.
return {
    general = {
        gaps_in     = {{ gap }},
        gaps_out    = {{ edge }},
        border_size = {{ border_size }},
        col = {
            active_border   = "rgb({{ accent | hex }})",
            inactive_border = "rgb({{ border | hex }})",
        },
    },
    decoration = {
        rounding = {{ radius }},
        blur     = { enabled = {{ blur }} },
        shadow   = { enabled = {{ shadows }}, color = "rgba({{ shadow | hex }}66)" },
    },
    animations = {
        enabled = {{ animations }},
    },
}
