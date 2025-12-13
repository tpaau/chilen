use std::fmt::Display;

pub enum GuiExitStatus {
    DaemonShutdown,
}

impl Display for GuiExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiExitStatus::DaemonShutdown => write!(
                f,
                "GUI stopped because a shutdown command was recieved from the daemon"
            ),
        }
    }
}

pub enum GuiError {
    StoppedUnexpectedly,
}

impl Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiError::StoppedUnexpectedly => write!(f, "GUI stopped unexpectedly"),
        }
    }
}

pub fn start() -> Result<GuiExitStatus, GuiError> {
    Err(GuiError::StoppedUnexpectedly)
}
