use vello::peniko::Color;

const fn color(hex: u32) -> Color {
    let r = ((hex & 0xff0000) >> 16) as u8;
    let g = ((hex & 0xff00) >> 8) as u8;
    let b = (hex & 0xff) as u8;
    Color::rgb8(r, g, b)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub background: Color,
    pub highlight: Color,
    pub lowlight: Color,
}

impl Theme {
    pub const ALL: &'static [Theme] = &[
        Theme::LIGHT,
        Theme::DARK,
        Theme::PEACH,
        Theme::RUST,
        Theme::AQUAMARINE,
        Theme::POLICE,
    ];

    pub const LIGHT: Theme = Theme {
        background: color(0xf3f3f3),
        highlight: color(0x010101),
        lowlight: color(0xced4da),
    };

    pub const DARK: Theme = Theme {
        background: color(0x000000),
        highlight: color(0xffffff),
        lowlight: color(0x808080),
    };

    pub const PEACH: Theme = Theme {
        background: color(0xfef6e4),
        highlight: color(0xf582ae),
        lowlight: color(0xf3d2c1),
    };

    pub const RUST: Theme = Theme {
        background: color(0x271c19),
        highlight: color(0xffc0ad),
        lowlight: color(0x55423d),
    };

    pub const AQUAMARINE: Theme = Theme {
        background: color(0xf3fffe),
        highlight: color(0x5784b1),
        lowlight: color(0xb9dcf0),
    };

    pub const POLICE: Theme = Theme {
        background: color(0x404f69),
        highlight: color(0xd4f1ed),
        lowlight: color(0x667287),
    };
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match *self {
                _ => "Unknown Theme",
            }
        )
    }
}
