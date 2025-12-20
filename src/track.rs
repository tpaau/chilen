use std::path::PathBuf;

use cxx_qt_lib::QString;
use serde::{Deserialize, Serialize};

#[cxx_qt::bridge]
mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        /// An alias to the QString type
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qproperty(QString, path)]
        #[namespace = "track"]
        type QTrack = super::RQTrack;
    }
}

#[derive(Default)]
pub struct RQTrack {
    pub path: QString,
    pub cover_path: QString,
    pub thumbnail_path: QString,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub path: PathBuf,
    pub cover_path: Option<PathBuf>,
    pub thumbnail_path: Option<PathBuf>,
}

impl From<RQTrack> for Track {
    fn from(value: RQTrack) -> Self {
        Track {
            path: PathBuf::from(String::from(&value.path)),
            cover_path: if value.path != QString::default() {
                Some(PathBuf::from(String::from(&value.cover_path)))
            } else {
                None
            },
            thumbnail_path: if value.path != QString::default() {
                Some(PathBuf::from(String::from(&value.thumbnail_path)))
            } else {
                None
            },
        }
    }
}
