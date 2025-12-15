use std::{fmt::Display, thread::sleep, time::Duration};

use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

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
    // Create the application and engine
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();

    // Load the QML path into the engine
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qrc:/qt/qml/com/kdab/cxx_qt/demo/qml/main.qml"));
    }

    if let Some(engine) = engine.as_mut() {
        // Listen to a signal from the QML Engine
        engine
            .as_qqmlengine()
            .on_quit(|_| {
                println!("QML Quit!");
            })
            .release();
    }

    // Start the app
    if let Some(app) = app.as_mut() {
        app.exec();
    }

    sleep(Duration::from_secs(1));

    Err(GuiError::StoppedUnexpectedly)
}
