use std::fmt::Display;

use crate::argparse::GuiCommand;

pub enum GuiExitStatus {
    Ok,
}

impl Display for GuiExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiExitStatus::Ok => write!(f, "GUI exited succesfully"),
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
