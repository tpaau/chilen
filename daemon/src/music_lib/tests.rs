use std::collections::{HashMap, HashSet};

use crate::music_lib::{Playlist, Track, state::MusicLibrary};

#[cfg(test)]
fn unique_playlists(count: usize) -> Vec<Playlist> {
    let mut playlists = Vec::new();
    for i in 0..count {
        playlists.push(Playlist {
            name: "Test".to_owned() + &i.to_string(),
            tracks: Vec::new(),
        });
    }
    playlists
}

#[test]
fn playlist_track_removal() {
    let tracks = Track::unique_tracks(10);
    let tracks_cloned = tracks.clone();
    let mut playlist = Playlist {
        name: "Test".to_string(),
        tracks: tracks_cloned,
    };
    playlist.remove_tracks(vec![3, 5, 8]).unwrap();
    assert_eq!(playlist.tracks[0], tracks[0]);
    assert_eq!(playlist.tracks[1], tracks[1]);
    assert_eq!(playlist.tracks[2], tracks[2]);
    assert_eq!(playlist.tracks[3], tracks[4]);
    assert_eq!(playlist.tracks[4], tracks[6]);
    assert_eq!(playlist.tracks[5], tracks[7]);
    assert_eq!(playlist.tracks[6], tracks[9]);
}

#[test]
fn playlist_removal() {
    let playlists = unique_playlists(10);
    let mut lib = MusicLibrary {
        tracks: HashSet::new(),
        playlists: playlists.clone(),
        tracks_by_path: HashMap::new(),
    };
    for playlist in &playlists {
        eprintln!("playlist: {}", playlist.name);
    }
    lib.remove_playlists(vec![
        "Test1".to_string(),
        "Test5".to_string(),
        "Test8".to_string(),
    ])
    .unwrap();
    assert_eq!(lib.playlists[0], playlists[0]);
    assert_eq!(lib.playlists[1], playlists[2]);
    assert_eq!(lib.playlists[2], playlists[3]);
    assert_eq!(lib.playlists[3], playlists[4]);
    assert_eq!(lib.playlists[4], playlists[6]);
    assert_eq!(lib.playlists[5], playlists[7]);
    assert_eq!(lib.playlists[6], playlists[9]);
}
