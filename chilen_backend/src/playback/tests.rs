use std::{sync::Arc, time::Duration};

use crate::{
    music_lib::{Track, get_library},
    playback::{
        LoopState, PLAYER_HANDLE, PlaybackState, ShuffleState, set_queue,
        state::{PLAYER_STATE, PlayerState},
    },
    tests::setup_test_env,
};

// Higher values will reduce the chance of false positives but will increase the runtime of some
// tests.
const TEST_ITER_COUNT: u32 = 100;

#[test]
fn skip_next() {
    let mut state = PlayerState {
        tracks: Track::unique_tracks(5).into_iter().map(Arc::new).collect(),
        ..Default::default()
    };
    while state.can_go_next() {
        println!("Position: {}", state.position);
        state.next_track().unwrap();
    }
    assert!(state.next_track().is_none());
    state.set_shuffle_state(ShuffleState::On);
    state.position = 0;
    while state.can_go_next() {
        println!("Position: {}", state.position);
        state.next_track().unwrap();
    }
    assert!(state.next_track().is_none());
}

#[test]
fn skip_previous() {
    let mut state = PlayerState {
        tracks: Track::unique_tracks(5).into_iter().map(Arc::new).collect(),
        ..Default::default()
    };
    while state.can_go_previous() {
        state.previous_track().unwrap();
    }
    assert!(state.previous_track().is_none());
    state.set_shuffle_state(ShuffleState::On);
    state.position = 0;
    while state.can_go_previous() {
        state.previous_track().unwrap();
    }
    assert!(state.previous_track().is_none());
}

#[test]
fn track_loop() {
    let mut state = PlayerState {
        tracks: Track::unique_tracks(3).into_iter().map(Arc::new).collect(),
        ..Default::default()
    };
    state.position = 1;
    state.loop_state = LoopState::Track;
    let track = state.tracks[state.position].clone();
    for _ in 0..state.tracks.len() {
        assert_eq!(state.next_track().unwrap(), track);
    }
    for _ in 0..state.tracks.len() {
        assert_eq!(state.previous_track().unwrap(), track);
    }

    state.position = 1;
    state.set_shuffle_state(ShuffleState::On);
    for _ in 0..state.tracks.len() {
        assert_eq!(state.next_track().unwrap(), track);
    }
    for _ in 0..state.tracks.len() {
        assert_eq!(state.previous_track().unwrap(), track);
    }
}

#[test]
fn playlist_loop() {
    let mut state = PlayerState {
        tracks: Track::unique_tracks(4).into_iter().map(Arc::new).collect(),
        ..Default::default()
    };
    state.loop_state = LoopState::Playlist;
    for _ in 0..state.tracks.len() + 2 {
        state.next_track().unwrap();
    }
    assert_eq!(state.tracks[2], state.current().unwrap());
    for _ in 0..state.tracks.len() + 2 {
        state.previous_track().unwrap();
    }
    assert_eq!(state.tracks[0], state.current().unwrap());
}

#[test]
fn shuffle_works() {
    let tracks: Vec<_> = Track::unique_tracks(10).into_iter().map(Arc::new).collect();
    let mut state = PlayerState {
        tracks: tracks.clone(),
        ..Default::default()
    };
    state.set_shuffle_state(ShuffleState::On);
    for _ in 0..TEST_ITER_COUNT {
        state.shuffle();
        assert_eq!(state.tracks, tracks);
        if state.tracks != state.shuffled_tracks {
            return;
        }
    }
    panic!("Shuffle doesn't seem to be working!");
}

#[test]
fn shuffle_track_stays_the_same() {
    let mut state = PlayerState {
        tracks: Track::unique_tracks(10).into_iter().map(Arc::new).collect(),
        ..Default::default()
    };
    state.position = rand::random_range(0..state.tracks.len() - 1);
    state.set_shuffle_state(ShuffleState::On);
    let loops = [LoopState::Off, LoopState::Track, LoopState::Playlist];
    for loop_state in loops {
        state.set_loop_state(loop_state);
        println!("For loop state: {loop_state}");
        for i in 0..TEST_ITER_COUNT {
            let pre_track = if loop_state == LoopState::Track {
                Some(state.current().unwrap())
            } else {
                None
            };
            if rand::random_range(0..5) == 1 {
                if state.shuffle_state.enabled() {
                    println!("Disabling shuffle!");
                    let track = &state.shuffled_tracks[state.position].clone();
                    state.set_shuffle_state(ShuffleState::Off);
                    assert_eq!(state.current().unwrap(), track.clone());
                } else {
                    println!("Enabling shuffle!");
                    let track = &state.tracks[state.position].clone();
                    state.set_shuffle_state(ShuffleState::On);
                    assert_eq!(state.current().unwrap(), track.clone());
                }
            }
            println!(
                "i: {i}, position: {}, shuffle: {}, tracks: {}, shuffled_tracks: {}",
                state.position,
                state.shuffle_state,
                state.tracks.len(),
                state.shuffled_tracks.len()
            );
            let track = if rand::random() && state.can_go_next() {
                println!("Right!");
                state.next_track()
            } else {
                println!("Left!");
                state.previous_track()
            };
            if loop_state == LoopState::Track {
                assert_eq!(track, pre_track);
            }
        }
    }
}

#[test]
fn test_set_queue() {
    setup_test_env();
    let lib = get_library().expect("Couldn't get the music library");
    let tracks: Vec<_> = lib.tracks;
    set_queue(tracks.clone()).expect("Couldn't set the track queue");
    let mut state_guard = PLAYER_STATE.write().unwrap();
    let state = state_guard.as_mut().unwrap();
    let player_guard = PLAYER_HANDLE.read().unwrap();
    let player = player_guard.as_ref().unwrap();
    assert!(player.is_paused());
    assert_eq!(state.playback_state, PlaybackState::Stopped);
    assert_eq!(state.player_position, Duration::default());
    assert_eq!(state.position, 0);
    assert_eq!(state.tracks, tracks);
}

#[test]
fn test_play_new_queue() {
    const TEST_ITER_COUNT: usize = 100;

    let tracks: Vec<_> = Track::unique_tracks(10).into_iter().map(Arc::new).collect();
    let mut state = PlayerState::default();

    for _ in 0..TEST_ITER_COUNT {
        state.set_shuffle_state(ShuffleState::Off);
        let index = rand::random_range(0..tracks.len() - 1);
        let expected = tracks.get(index).cloned();
        state.play_new_queue(tracks.clone(), index);
        assert_eq!(state.current(), expected);
        assert_eq!(state.position, index);

        state.set_shuffle_state(ShuffleState::On);
        let index = rand::random_range(0..tracks.len() - 1);
        let expected = tracks.get(index).cloned();
        state.play_new_queue(tracks.clone(), index);
        assert_eq!(state.current(), expected);
        assert_eq!(state.position, 0);
    }
}
