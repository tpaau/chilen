#[cfg(test)]
use std::time::Duration;
use std::{path::PathBuf, sync::Arc};

use crate::testing_init_config;
#[cfg(test)]
use crate::{
    Error,
    music_lib::{
        Track,
        indexer::covers::Cover,
        state::{MusicLibrary, Playlist},
    },
};

#[cfg(test)]
fn unique_playlists(count: usize) -> Vec<Playlist> {
    let mut playlists = Vec::new();
    for i in 0..count {
        playlists.push(Playlist {
            name: ("Test".to_owned() + &i.to_string()).into_boxed_str(),
            tracks: Box::new([]),
            duration: Duration::default(),
            unmatched: Box::new([]),
            id: i as u64,
            cover: Cover::none(),
        });
    }
    playlists
}

#[test]
fn playlist_track_removal() {
    let tracks: Box<[Arc<Track>]> = Track::unique_tracks(10).into_iter().map(Arc::new).collect();
    let mut playlist = Playlist {
        name: "Test".to_string().into(),
        tracks: tracks.clone(),
        duration: Duration::default(),
        unmatched: Box::new([]),
        id: 0,
        cover: Cover::none(),
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
    testing_init_config();
    let playlists = unique_playlists(10);
    let mut lib = MusicLibrary::new_testing(Vec::new());
    for p in playlists.iter() {
        lib.create_playlist(&p.name, &None).unwrap();
    }
    let mut playlists: Vec<_> = lib.playlists.clone().into_iter().collect();
    playlists.sort_by(|a, b| a.name.cmp(&b.name));

    for playlist in &playlists {
        eprintln!("original playlist: {}, id: {}", playlist.name, playlist.id);
    }
    lib.remove_playlists(vec!["Test1", "Test5", "Test8"])
        .unwrap();
    let mut modified_playlists: Vec<_> = lib.playlists.into_iter().collect();
    modified_playlists.sort_by_key(|p| p.name.clone());

    for playlist in &modified_playlists {
        eprintln!("modified playlist: {}, id: {}", playlist.name, playlist.id);
    }

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
    testing_init_config();
    let tracks = Track::unique_tracks(10);
    let mut lib = MusicLibrary::new_testing(tracks.clone());

    assert!(lib.find_playlist_by_name("Test1").is_none());
    assert_eq!(lib.playlists.len(), 0);
    lib.create_playlist("Test1", &Some(vec![tracks[6].path.clone()]))
        .unwrap();
    assert_eq!(lib.playlists.len(), 1);
    assert_eq!(
        lib.find_playlist_by_name("Test1").unwrap().name.as_ref(),
        "Test1"
    );

    assert_eq!(
        lib.create_playlist("Test1", &None).unwrap_err(),
        Error::PlaylistExists
    );
    assert_eq!(lib.playlists.len(), 1);
    assert_eq!(
        lib.find_playlist_by_name("Test1").unwrap().name.as_ref(),
        "Test1"
    );

    let path: PathBuf = "/nonexistent/path".into();
    assert_eq!(
        lib.create_playlist("Test2", &Some(vec![path.clone()]))
            .unwrap_err(),
        Error::UnknownTrackPath(path)
    );
    assert_eq!(lib.playlists.len(), 1);
    assert_eq!(
        lib.find_playlist_by_name("Test1").unwrap().name.as_ref(),
        "Test1"
    );
}

#[test]
fn playlist_deletion() {
    testing_init_config();
    let mut lib = MusicLibrary::new_testing(Track::unique_tracks(10));

    assert!(lib.find_playlist_by_name("Test1").is_none());
    lib.create_playlist("Test1", &None).unwrap();
    lib.create_playlist("Test2", &None).unwrap();
    lib.create_playlist("Test3", &None).unwrap();
    assert_eq!(
        lib.find_playlist_by_name("Test1").unwrap().name.as_ref(),
        "Test1"
    );
    assert_eq!(
        lib.find_playlist_by_name("Test2").unwrap().name.as_ref(),
        "Test2"
    );
    assert_eq!(
        lib.find_playlist_by_name("Test3").unwrap().name.as_ref(),
        "Test3"
    );

    assert_eq!(lib.playlists.len(), 3);
    lib.remove_playlists(vec!["Test1"]).unwrap();
    assert_eq!(lib.playlists.len(), 2);
    assert!(lib.find_playlist_by_name("Test1").is_none());

    assert_eq!(
        lib.remove_playlists(vec!["Test2", "Test2", "Test3"])
            .unwrap_err(),
        Error::DuplicateItems
    );
    assert_eq!(lib.playlists.len(), 2);

    let name = "Test1".to_string();
    assert_eq!(
        lib.remove_playlists(vec![&name]).unwrap_err(),
        Error::UnknownPlaylist(name)
    );
    assert_eq!(lib.playlists.len(), 2);

    lib.remove_playlists(vec!["Test2", "Test3"]).unwrap();
    assert_eq!(lib.playlists.len(), 0);
    assert!(lib.find_playlist_by_name("Test1").is_none());
    assert!(lib.find_playlist_by_name("Test2").is_none());
    assert!(lib.find_playlist_by_name("Test3").is_none());
}

#[test]
fn track_append() {
    testing_init_config();
    let tracks = Track::unique_tracks(10);
    let mut lib = MusicLibrary::new_testing(tracks.clone());

    lib.create_playlist("Test1", &None).unwrap();
    lib.add_tracks("Test1", &[tracks[0].hash_self()]).unwrap();
    assert_eq!(
        lib.add_tracks("Test2", &[tracks[0].hash_self()])
            .unwrap_err(),
        Error::UnknownPlaylist("Test2".to_string())
    );

    assert_eq!(
        lib.add_tracks("Test1", &[4]).unwrap_err(),
        Error::UnknownTrack { hash: 4 }
    );
}

#[test]
fn lib_track_removal() {
    testing_init_config();
    let tracks = Track::unique_tracks(10);
    let mut lib = MusicLibrary::new_testing(tracks.clone());

    lib.create_playlist(
        "Test1",
        &Some(vec![
            tracks[0].path.clone(),
            tracks[1].path.clone(),
            tracks[2].path.clone(),
        ]),
    )
    .unwrap();

    lib.remove_tracks("Test1", vec![0]).unwrap();
    assert_eq!(
        lib.remove_tracks("Test2", vec![1]).unwrap_err(),
        Error::UnknownPlaylist("Test2".to_string())
    );
    assert_eq!(
        lib.remove_tracks("Test1", vec![2]).unwrap_err(),
        Error::IndexOutOfBounds
    );
}
