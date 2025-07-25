use colored::Colorize;

pub enum ColorChoice {
    EnableColors,
    DisableColors,
}

impl ColorChoice {
    pub fn red(&self, text: &str) -> String {
        match self {
            ColorChoice::EnableColors => text.red().to_string(),
            ColorChoice::DisableColors => text.to_string(),
        }
    }

    pub fn bold(&self, text: &str) -> String {
        match self {
            ColorChoice::EnableColors => text.bold().to_string(),
            ColorChoice::DisableColors => text.to_string(),
        }
    }
}
