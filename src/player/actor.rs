use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::mpsc as std_mpsc;
use std::time::Duration;

use rodio::{Decoder, DeviceSinkBuilder, Player};

use crate::{
    client::Backend,
    messages::{LoopMode, PlaybackStatus, PlayerCommand, PlayerState},
};

use super::source;

pub(crate) struct ActorChannels {
    pub cmd_rx: tokio::sync::mpsc::Receiver<PlayerCommand>,
    pub state_tx: tokio::sync::broadcast::Sender<PlayerState>,
    pub backend: Backend,
    pub rt_handle: tokio::runtime::Handle,
}

pub(crate) fn run(channels: ActorChannels) {
    let ActorChannels { cmd_rx, state_tx, backend, rt_handle } = channels;

    let mut device_sink =
        DeviceSinkBuilder::open_default_sink().expect("failed to open audio output device");
    device_sink.log_on_drop(false);

    let (bridge_tx, bridge_rx) = std_mpsc::channel::<PlayerCommand>();
    let mut async_cmd_rx = cmd_rx;
    std::thread::Builder::new()
        .name("ytm-player-bridge".into())
        .spawn(move || {
            while let Some(cmd) = async_cmd_rx.blocking_recv() {
                if bridge_tx.send(cmd).is_err() {
                    break;
                }
            }
        })
        .expect("bridge thread spawn");

    let mut state = PlayerState::default();
    let mut current_player: Option<Player> = None;
    let mut cached_path: Option<PathBuf> = None;

    let publish = |tx: &tokio::sync::broadcast::Sender<PlayerState>, s: &PlayerState| {
        let _ = tx.send(s.clone());
    };

    loop {
        match bridge_rx.recv_timeout(Duration::from_millis(200)) {
            Ok(PlayerCommand::Play(track)) => {
                if let Some(p) = current_player.take() {
                    p.stop();
                }

                state.status = PlaybackStatus::Loading;
                state.error = None;
                publish(&state_tx, &state);

                let path_result = rt_handle.block_on(source::ensure_local(&backend, &track.id));

                match path_result {
                    Err(e) => {
                        state.status = PlaybackStatus::Idle;
                        state.error = Some(e.to_string());
                    }
                    Ok(path) => {
                        match File::open(&path)
                            .map_err(|e| e.to_string())
                            .and_then(|f| {
                                Decoder::new(BufReader::new(f)).map_err(|e| e.to_string())
                            }) {
                            Err(e) => {
                                state.status = PlaybackStatus::Idle;
                                state.error = Some(format!("decode error: {e}"));
                            }
                            Ok(decoded) => {
                                let player = Player::connect_new(device_sink.mixer());
                                player.append(decoded);
                                player.play();
                                current_player = Some(player);
                                cached_path = Some(path);
                                state.current = Some(track);
                                state.status = PlaybackStatus::Playing;
                                state.position = Duration::ZERO;
                            }
                        }
                    }
                }

                publish(&state_tx, &state);
            }

            Ok(PlayerCommand::TogglePause) => {
                if let Some(ref player) = current_player {
                    if player.is_paused() {
                        player.play();
                        state.status = PlaybackStatus::Playing;
                    } else {
                        player.pause();
                        state.status = PlaybackStatus::Paused;
                    }
                    publish(&state_tx, &state);
                }
            }

            Ok(PlayerCommand::Stop) => {
                if let Some(p) = current_player.take() {
                    p.stop();
                }
                state.status = PlaybackStatus::Idle;
                state.position = Duration::ZERO;
                publish(&state_tx, &state);
            }

            Ok(PlayerCommand::SetLoop(mode)) => {
                state.loop_mode = mode;
                publish(&state_tx, &state);
            }

            Ok(PlayerCommand::Shutdown) => break,

            Err(std_mpsc::RecvTimeoutError::Timeout) => {
                if state.status != PlaybackStatus::Playing {
                    continue;
                }

                let is_empty = current_player.as_ref().map_or(true, |p| p.empty());
                let position = current_player.as_ref().map_or(Duration::ZERO, |p| p.get_pos());
                state.position = position;

                if is_empty {
                    if state.loop_mode == LoopMode::One {
                        if let Some(ref path) = cached_path {
                            if let Ok(file) = File::open(path) {
                                if let Ok(decoded) = Decoder::new(BufReader::new(file)) {
                                    let player = Player::connect_new(device_sink.mixer());
                                    player.append(decoded);
                                    player.play();
                                    current_player = Some(player);
                                    state.position = Duration::ZERO;
                                }
                            }
                        }
                    } else {
                        current_player = None;
                        state.status = PlaybackStatus::Idle;
                        state.position = Duration::ZERO;
                    }
                }

                publish(&state_tx, &state);
            }

            Err(std_mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }
}
