#[cfg(feature = "gui")]
use cxx_qt_build::CxxQtBuilder;

fn main() {
    #[cfg(feature = "gui")]
    CxxQtBuilder::new()
        // Link Qt's Network library
        // - Qt Core is always linked
        // - Qt Gui is linked by enabling the qt_gui Cargo feature of cxx-qt-lib.
        // - Qt Qml is linked by enabling the qt_qml Cargo feature of cxx-qt-lib.
        // - Qt Qml requires linking Qt Network on macOS
        .qt_module("Network")
        // .qml_module(QmlModule {
        //     uri: "player",
        //     rust_files: &["src/cxxqt_object.rs", "src/track.rs"],
        //     qml_files: &["qml/main.qml"],
        //     ..Default::default()
        // })
        .build();
}
