use std::fmt::Display;

use crate::argparse::GuiCommand;

pub enum GuiExitStatus {
    Ok,
    StoppedUnexpectedly,
}

impl Display for GuiExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiExitStatus::Ok => write!(f, "GUI exited succesfully"),
            GuiExitStatus::StoppedUnexpectedly => write!(f, "GUI stopped unexpectedly"),
        }
    }
}

pub fn start(cmd: GuiCommand) -> Result<GuiExitStatus, GuiExitStatus> {
    Err(GuiExitStatus::StoppedUnexpectedly)
}
