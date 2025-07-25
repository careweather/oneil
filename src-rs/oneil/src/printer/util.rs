use colored::Colorize;

pub enum ColorChoice {
    EnableColors,
    DisableColors,
}

impl ColorChoice {
    pub fn bold(&self, text: &str) -> String {
        match self {
            ColorChoice::EnableColors => text.bold().to_string(),
            ColorChoice::DisableColors => text.to_string(),
        }
    }

    pub fn bold_red(&self, text: &str) -> String {
        match self {
            ColorChoice::EnableColors => text.bold().red().to_string(),
            ColorChoice::DisableColors => text.to_string(),
        }
    }

    pub fn bold_blue(&self, text: &str) -> String {
        match self {
            ColorChoice::EnableColors => text.bold().blue().to_string(),
            ColorChoice::DisableColors => text.to_string(),
        }
    }
}
