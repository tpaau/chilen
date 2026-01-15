mod cxxqt_object;

use std::{fmt::Display, thread::sleep, time::Duration};

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};
use log::trace;

pub enum GuiExitStatus {
    Quit,
}

impl Display for GuiExitStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // GuiExitStatus::DaemonShutdown => write!(
            //     f,
            //     "GUI stopped because a shutdown command was received from the daemon"
            // ),
            GuiExitStatus::Quit => write!(f, "GUI finished successfully"),
        }
    }
}

pub enum GuiError {
    EngineFailed,
    AppFailed,
}

impl Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiError::EngineFailed => write!(f, "The qml engine has failed"),
            GuiError::AppFailed => write!(f, "The qt app has failed"),
        }
    }
}

pub fn start() -> Result<GuiExitStatus, GuiError> {
    trace!("Starting GUI");

    // Create the application and engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the QML path into the engine
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/player/qml/main.qml"));
    } else {
        return Err(GuiError::EngineFailed);
    }

    if let Some(engine) = engine.as_mut() {
        // Listen to a signal from the QML Engine
        engine
            .as_qqmlengine()
            .on_quit(|_| {
                trace!("QML engine quit");
            })
            .release();
    } else {
        return Err(GuiError::EngineFailed);
    }

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    } else {
        return Err(GuiError::AppFailed);
    }

    sleep(Duration::from_secs(1));

    Ok(GuiExitStatus::Quit)
}
