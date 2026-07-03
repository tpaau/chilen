//! Module for keeping track of some properties of the handler not readily available to the daemon.
//!
//! Eg. the handler is responsible for managing the GUI interface of the app, so the daemon keeps
//! track of whether the app is in fullscreen mode or not, and then supplies that information to
//! clients when asked for it.

use std::sync::{LazyLock, RwLock};

use crate::handler::FullscreenState;

#[derive(Default)]
struct State {
    fullscreen: FullscreenState,
}

static STATE: LazyLock<RwLock<State>> = LazyLock::new(|| RwLock::new(State::default()));

pub(crate) fn fullscreen() -> FullscreenState {
    STATE.read().unwrap().fullscreen
}

pub(crate) fn set_fullscreen(state: FullscreenState) {
    STATE.write().unwrap().fullscreen = state;
}
