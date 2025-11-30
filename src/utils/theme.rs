#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Theme {
    Winter,
    Black,
    Nord,
    Dracula,
    Night,
    Dim,
}

impl Theme {
    pub fn all() -> Vec<Theme> {
        vec![
            Theme::Winter,
            Theme::Black,
            Theme::Nord,
            Theme::Dracula,
            Theme::Night,
            Theme::Dim,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Theme::Winter => "Winter",
            Theme::Black => "Black",
            Theme::Nord => "Nord",
            Theme::Dracula => "Dracula",
            Theme::Night => "Night",
            Theme::Dim => "Dim",
        }
    }

    pub fn data_theme(&self) -> &'static str {
        match self {
            Theme::Winter => "winter",
            Theme::Black => "black",
            Theme::Nord => "nord",
            Theme::Dracula => "dracula",
            Theme::Night => "night",
            Theme::Dim => "dim",
        }
    }

    pub fn is_dark(&self) -> bool {
        matches!(self, Theme::Dracula | Theme::Night | Theme::Dim)
    }

    pub fn is_light(&self) -> bool {
        !self.is_dark()
    }

    pub fn dark_themes() -> Vec<Theme> {
        vec![Theme::Dracula, Theme::Night, Theme::Dim]
    }

    pub fn light_themes() -> Vec<Theme> {
        vec![Theme::Winter, Theme::Black, Theme::Nord]
    }
}
