use std::time::Duration;

use chilen_backend::music_lib::Timestamp;

pub const UNKNOWN_TRACK_DURATION: &str = "--:--";

pub fn format_track_duration(duration: Duration) -> String {
    let minutes = duration.as_secs() / 60;
    let seconds = duration.as_secs() - minutes * 60;
    format!("{minutes}:{seconds:02}")
}

pub fn format_album_duration(d: Duration) -> String {
    let hours = d.as_secs() / 3600;
    let minutes = d.as_secs() / 60 - hours * 60;
    let seconds = d.as_secs() - minutes * 60 - hours * 3600;

    if hours > 0 {
        if minutes == 0 {
            format!("{hours} hr")
        } else {
            format!("{hours} hr {minutes} min")
        }
    } else {
        if seconds == 0 {
            format!("{minutes} min")
        } else {
            format!("{minutes} min {seconds} sec")
        }
    }
}

pub fn format_date(timestamp: Timestamp) -> String {
    let month = timestamp.month.and_then(|m| match m {
        1 => Some("Jan"),
        2 => Some("Feb"),
        3 => Some("Mar"),
        4 => Some("Apr"),
        5 => Some("May"),
        6 => Some("Jun"),
        7 => Some("Jul"),
        8 => Some("Aug"),
        9 => Some("Sep"),
        10 => Some("Oct"),
        11 => Some("Nov"),
        12 => Some("Dec"),
        _ => None,
    });

    if let Some(month) = month {
        format!("{month} {}", timestamp.year)
    } else {
        timestamp.year.to_string()
    }
}
