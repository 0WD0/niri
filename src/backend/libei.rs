//! libei (EIS) input backend.
//!
//! Accepts local Emulated Input (EI) clients over a Unix socket and feeds
//! their emulated input into the regular compositor input pipeline, exactly
//! like any other input backend.
//!
//! The socket is bound to `$XDG_RUNTIME_DIR/niri-eis.sock`. Clients (e.g.
//! TouchDeck) connect with a libei client and can then inject keyboard,
//! pointer and touch input; touch devices can declare coordinate regions so
//! the client knows which part of the desktop it may address.

use std::path::PathBuf;

use anyhow::Context;
use calloop::PostAction;
use smithay::backend::libei::{EiInput, EiInputEvent};
use smithay::reexports::reis::{calloop::EisListenerSource, eis};
use tracing::info;

use crate::niri::State;

/// Setup the libei/EIS listener on `$XDG_RUNTIME_DIR/niri-eis.sock`.
pub fn setup(state: &mut State) -> anyhow::Result<()> {
    let handle = state.niri.event_loop.clone();
    let xdg_runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .context("XDG_RUNTIME_DIR is not set")?;
    let socket_path = xdg_runtime_dir.join("niri-eis.sock");

    // Remove a stale socket left over from a previous run.
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .with_context(|| format!("error removing stale socket {}", socket_path.display()))?;
    }

    let listener = eis::Listener::bind(&socket_path)
        .with_context(|| format!("error binding EIS socket {}", socket_path.display()))?;
    info!("libei (EIS) listening on: {}", socket_path.display());

    let listener_handle = handle.clone();
    handle
        .insert_source(EisListenerSource::new(listener), move |context, _, _| {
            let source = EiInput::new(context);
            listener_handle
                .insert_source(source, |event, connection, state| match event {
                    EiInputEvent::Connected => {
                        let seat = connection.add_seat("default");
                        // TODO: wire regions to the modeline reservations so
                        // the client only injects touches where the shell can
                        // receive them.
                        seat.add_touch("touchdeck", &[]);
                        state.niri.libei_touch_seat = Some(seat);
                    }
                    EiInputEvent::Disconnected => {
                        state.niri.libei_touch_seat = None;
                    }
                    EiInputEvent::Event(event) => state.process_input_event(event),
                    EiInputEvent::TextKeysym { .. } | EiInputEvent::TextUtf8 { .. } => {}
                })
                .unwrap();

            Ok(PostAction::Continue)
        })
        .unwrap();

    Ok(())
}
