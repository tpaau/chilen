use std::sync::Arc;

use mpipc::library::LibraryError;

#[cfg(test)]
use crate::music_lib::{
    Track,
    state::{MusicLibrary, Playlist},
};

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
    let tracks: Vec<_> = Track::unique_tracks(10).into_iter().map(Arc::new).collect();
    let mut playlist = Playlist {
        name: "Test".to_string(),
        tracks: tracks.clone(),
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
    let mut lib = MusicLibrary::new_from_tracks(Vec::new());
    for p in playlists.iter() {
        lib.create_playlist(p.name.clone(), &None).unwrap();
    }
    lib.playlists = playlists.clone().into_iter().map(Arc::new).collect();

    for playlist in &playlists {
        eprintln!("playlist: {}", playlist.name);
    }
    lib.remove_playlists(vec![
        "Test1".to_string(),
        "Test5".to_string(),
        "Test8".to_string(),
    ])
    .unwrap();
    let mut modified_playlists: Vec<_> = lib
        .playlists
        .into_iter()
        .map(|t| t.as_ref().clone())
        .collect();
    modified_playlists.sort_by_key(|p| p.name.clone());

    assert_eq!(modified_playlists[0], playlists[0]);
    assert_eq!(modified_playlists[1], playlists[2]);
    assert_eq!(modified_playlists[2], playlists[3]);
    assert_eq!(modified_playlists[3], playlists[4]);
    assert_eq!(modified_playlists[4], playlists[6]);
    assert_eq!(modified_playlists[5], playlists[7]);
    assert_eq!(modified_playlists[6], playlists[9]);
}

#[test]
fn playlist_creation() {
    let tracks = Track::unique_tracks(10);
    let mut lib = MusicLibrary::new_from_tracks(tracks.clone());

    assert_eq!(lib.playlists.len(), 0);
    lib.create_playlist("Test1".to_string(), &Some(vec![tracks[6].path.clone()]))
        .unwrap();
    assert_eq!(lib.playlists.len(), 1);

    assert_eq!(
        lib.create_playlist("Test1".to_string(), &None).unwrap_err(),
        LibraryError::PlaylistExists
    );
    assert_eq!(lib.playlists.len(), 1);

    assert_eq!(
        lib.create_playlist("Test2".to_string(), &Some(vec!["/nonexistent/path".into()]))
            .unwrap_err(),
        LibraryError::NoSuchTrack
    );
    assert_eq!(lib.playlists.len(), 1);
}

#[test]
fn playlist_deletion() {
    let mut lib = MusicLibrary::new_from_tracks(Track::unique_tracks(10));

    lib.create_playlist("Test1".to_string(), &None).unwrap();
    lib.create_playlist("Test2".to_string(), &None).unwrap();
    lib.create_playlist("Test3".to_string(), &None).unwrap();
    assert_eq!(lib.playlists.len(), 3);
    lib.remove_playlists(vec!["Test1".to_string()]).unwrap();
    assert_eq!(lib.playlists.len(), 2);
    assert_eq!(
        lib.remove_playlists(vec![
            "Test2".to_string(),
            "Test2".to_string(),
            "Test3".to_string()
        ])
        .unwrap_err(),
        LibraryError::DuplicateItems
    );
    assert_eq!(lib.playlists.len(), 2);
    assert_eq!(
        lib.remove_playlists(vec!["Test1".to_string()]).unwrap_err(),
        LibraryError::NoSuchPlaylist
    );
    assert_eq!(lib.playlists.len(), 2);
    lib.remove_playlists(vec!["Test2".to_string(), "Test3".to_string()])
        .unwrap();
    assert_eq!(lib.playlists.len(), 0);
}
