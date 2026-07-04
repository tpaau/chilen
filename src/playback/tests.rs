use std::time::Duration;

use chilen_ipc::playback::ShuffleState;
use chilen_ipc::playback::{LoopState, PlaybackState};

use crate::{
    music_lib::state::{Track, get_library},
    playback::{
        PLAYER_HANDLE, set_queue,
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
        tracks: Track::unique_tracks(5),
        ..Default::default()
    };
    while state.can_go_next() {
        println!("Position: {}", state.position);
        state.next().unwrap();
    }
    assert!(state.next().is_none());
    state.set_shuffle_state(ShuffleState::On);
    state.shuffle();
    state.position = 0;
    while state.can_go_next() {
        println!("Position: {}", state.position);
        state.next().unwrap();
    }
    assert!(state.next().is_none());
}

#[test]
fn skip_previous() {
    let mut state = PlayerState {
        tracks: Track::unique_tracks(5),
        ..Default::default()
    };
    while state.can_go_previous() {
        state.previous().unwrap();
    }
    assert!(state.previous().is_none());
    state.set_shuffle_state(ShuffleState::On);
    state.shuffle();
    state.position = 0;
    while state.can_go_previous() {
        state.previous().unwrap();
    }
    assert!(state.previous().is_none());
}

#[test]
fn track_loop() {
    let mut state = PlayerState {
        tracks: Track::unique_tracks(3),
        ..Default::default()
    };
    state.position = 1;
    state.loop_state = LoopState::Track;
    let track = state.tracks[state.position].clone();
    for _ in 0..state.tracks.len() {
        assert_eq!(state.next().unwrap(), &track);
    }
    for _ in 0..state.tracks.len() {
        assert_eq!(state.previous().unwrap(), &track);
    }

    state.position = 1;
    state.set_shuffle_state(ShuffleState::On);
    state.shuffle();
    for _ in 0..state.tracks.len() {
        assert_eq!(state.next().unwrap(), &track);
    }
    for _ in 0..state.tracks.len() {
        assert_eq!(state.previous().unwrap(), &track);
    }
}

#[test]
fn playlist_loop() {
    let mut state = PlayerState {
        tracks: Track::unique_tracks(4),
        ..Default::default()
    };
    state.loop_state = LoopState::Playlist;
    for _ in 0..state.tracks.len() + 2 {
        state.next().unwrap();
    }
    assert_eq!(&state.tracks[2].clone(), state.current().unwrap());
    for _ in 0..state.tracks.len() + 2 {
        state.previous().unwrap();
    }
    assert_eq!(&state.tracks[0].clone(), state.current().unwrap());
}

#[test]
fn shuffle_works() {
    let tracks = Track::unique_tracks(10);
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
        tracks: Track::unique_tracks(10),
        ..Default::default()
    };
    state.position = rand::random_range(0..state.tracks.len() - 1);
    state.set_shuffle_state(ShuffleState::On);
    state.shuffle();
    let loops = [LoopState::Off, LoopState::Track, LoopState::Playlist];
    for loop_state in loops {
        state.set_loop_state(loop_state);
        println!("For loop state: {loop_state}");
        for i in 0..TEST_ITER_COUNT {
            let pre_track = if loop_state == LoopState::Track {
                Some(state.current().cloned().unwrap())
            } else {
                None
            };
            if rand::random_range(0..5) == 1 {
                if state.shuffle_state == ShuffleState::Off {
                    println!("Enabling shuffle!");
                    let track = &state.tracks[state.position].clone();
                    state.set_shuffle_state(ShuffleState::On);
                    state.shuffle();
                    assert_eq!(state.current().unwrap(), track);
                } else {
                    println!("Disabling shuffle!");
                    let track = &state.shuffled_tracks[state.position].clone();
                    state.set_shuffle_state(ShuffleState::Off);
                    assert_eq!(state.current().unwrap(), track);
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
                println!("Right (1)!");
                state.next()
            } else if state.can_go_previous() {
                println!("Left!");
                state.previous()
            } else {
                println!("Right (2)!");
                state.next()
            }
            .cloned();
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
    let tracks: Vec<_> = lib.tracks.into_iter().map(|t| t.as_ref().clone()).collect();
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
