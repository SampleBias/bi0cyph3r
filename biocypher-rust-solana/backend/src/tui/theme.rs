use ratatui::style::Color;

/// Semantic colors shared by every screen, including dialogs and DNA bases.
#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub panel: Color,
    pub border: Color,
    pub text: Color,
    pub muted: Color,
    pub accent: Color,
    pub secondary: Color,
    pub positive: Color,
    pub warning: Color,
    pub error: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Theme {
    #[default]
    Original,
    Crimson,
    Paper,
    Monochrome,
    Amber,
}

impl Theme {
    pub const ALL: [Self; 5] = [
        Self::Original,
        Self::Crimson,
        Self::Paper,
        Self::Monochrome,
        Self::Amber,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::Original => "Original",
            Self::Crimson => "Crimson",
            Self::Paper => "Paper",
            Self::Monochrome => "Monochrome",
            Self::Amber => "Amber",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Original => "The original cyan & violet",
            Self::Crimson => "Deep red / warm scarlet",
            Self::Paper => "White background / dark ink",
            Self::Monochrome => "Pure black, white & gray",
            Self::Amber => "Vintage amber phosphor",
        }
    }

    pub fn palette(self) -> Palette {
        use Color::Rgb;
        match self {
            Self::Original => Palette {
                bg: Rgb(13, 17, 26),
                panel: Rgb(18, 24, 36),
                border: Rgb(46, 59, 78),
                text: Rgb(222, 230, 240),
                muted: Rgb(133, 150, 174),
                accent: Rgb(87, 222, 218),
                secondary: Rgb(180, 155, 255),
                positive: Rgb(143, 219, 159),
                warning: Rgb(242, 198, 121),
                error: Rgb(243, 132, 148),
            },
            Self::Crimson => Palette {
                bg: Rgb(22, 10, 15),
                panel: Rgb(33, 16, 23),
                border: Rgb(87, 41, 53),
                text: Rgb(250, 232, 230),
                muted: Rgb(186, 143, 151),
                accent: Rgb(255, 99, 112),
                secondary: Rgb(247, 158, 163),
                positive: Rgb(245, 193, 159),
                warning: Rgb(255, 181, 104),
                error: Rgb(255, 125, 98),
            },
            Self::Paper => Palette {
                bg: Rgb(255, 255, 255),
                panel: Rgb(245, 247, 250),
                border: Rgb(183, 193, 207),
                text: Rgb(25, 34, 48),
                muted: Rgb(84, 100, 120),
                accent: Rgb(0, 104, 112),
                secondary: Rgb(108, 59, 166),
                positive: Rgb(39, 108, 67),
                warning: Rgb(142, 86, 15),
                error: Rgb(182, 42, 66),
            },
            Self::Monochrome => Palette {
                bg: Rgb(0, 0, 0),
                panel: Rgb(14, 14, 14),
                border: Rgb(75, 75, 75),
                text: Rgb(245, 245, 245),
                muted: Rgb(155, 155, 155),
                accent: Rgb(255, 255, 255),
                secondary: Rgb(193, 193, 193),
                positive: Rgb(225, 225, 225),
                warning: Rgb(183, 183, 183),
                error: Rgb(255, 255, 255),
            },
            Self::Amber => Palette {
                bg: Rgb(19, 15, 8),
                panel: Rgb(29, 23, 12),
                border: Rgb(78, 61, 30),
                text: Rgb(246, 229, 191),
                muted: Rgb(171, 150, 109),
                accent: Rgb(255, 195, 83),
                secondary: Rgb(238, 155, 70),
                positive: Rgb(219, 204, 128),
                warning: Rgb(255, 214, 125),
                error: Rgb(255, 135, 91),
            },
        }
    }
}
