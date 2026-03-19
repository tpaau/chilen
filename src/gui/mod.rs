pub enum GuiExitStatus {
    Quit,
}

impl std::fmt::Display for GuiExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Quit => write!(f, "GUI finished successfully"),
        }
    }
}

pub enum GuiError {
    Unknown,
}

impl std::fmt::Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "An unknown error has occured in the GUI"),
        }
    }
}

pub fn start() -> Result<GuiExitStatus, GuiError> {
    panic!("The GUI is not implemented!");
}
