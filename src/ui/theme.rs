use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: &'static str,
    pub border: Color,
    pub header: Color,
    pub primary: Color,
    pub secondary: Color,
    pub text: Color,
    pub dim: Color,
    pub selected_bg: Color,
    pub selected_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::by_name("default")
    }
}

impl Theme {
    pub fn by_name(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "gruvbox" => Theme {
                name: "gruvbox",
                border: Color::Rgb(146, 131, 116),
                header: Color::Rgb(250, 189, 47),
                primary: Color::Rgb(131, 165, 152),
                secondary: Color::Rgb(211, 134, 155),
                text: Color::Rgb(235, 219, 178),
                dim: Color::Rgb(124, 111, 100),
                selected_bg: Color::Rgb(80, 73, 69),
                selected_fg: Color::Rgb(251, 241, 199),
            },
            "dracula" => Theme {
                name: "dracula",
                border: Color::Rgb(98, 114, 164),
                header: Color::Rgb(189, 147, 249),
                primary: Color::Rgb(139, 233, 253),
                secondary: Color::Rgb(255, 121, 198),
                text: Color::Rgb(248, 248, 242),
                dim: Color::Rgb(98, 114, 164),
                selected_bg: Color::Rgb(68, 71, 90),
                selected_fg: Color::Rgb(248, 248, 242),
            },
            "nord" => Theme {
                name: "nord",
                border: Color::Rgb(76, 86, 106),
                header: Color::Rgb(136, 192, 208),
                primary: Color::Rgb(143, 188, 187),
                secondary: Color::Rgb(180, 142, 173),
                text: Color::Rgb(216, 222, 233),
                dim: Color::Rgb(76, 86, 106),
                selected_bg: Color::Rgb(67, 76, 94),
                selected_fg: Color::Rgb(236, 239, 244),
            },
            "catppuccin" => Theme {
                name: "catppuccin",
                border: Color::Rgb(108, 112, 134),
                header: Color::Rgb(203, 166, 247),
                primary: Color::Rgb(137, 180, 250),
                secondary: Color::Rgb(245, 194, 231),
                text: Color::Rgb(205, 214, 244),
                dim: Color::Rgb(108, 112, 134),
                selected_bg: Color::Rgb(49, 50, 68),
                selected_fg: Color::Rgb(245, 224, 220),
            },
            "solarized" => Theme {
                name: "solarized",
                border: Color::Rgb(88, 110, 117),
                header: Color::Rgb(38, 139, 210),
                primary: Color::Rgb(42, 161, 152),
                secondary: Color::Rgb(211, 54, 130),
                text: Color::Rgb(147, 161, 161),
                dim: Color::Rgb(101, 123, 131),
                selected_bg: Color::Rgb(7, 54, 66),
                selected_fg: Color::Rgb(238, 232, 213),
            },
            _ => Theme {
                name: "default",
                border: Color::DarkGray,
                header: Color::Cyan,
                primary: Color::Cyan,
                secondary: Color::Magenta,
                text: Color::Reset,
                dim: Color::DarkGray,
                selected_bg: Color::Rgb(40, 40, 40),
                selected_fg: Color::White,
            },
        }
    }
}
