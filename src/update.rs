use crate::{
    audio::Audio,
    client::Backend,
    model::{AudioStatus, LoopMode, Model, PlayerFocus, SearchFocus, View},
    msg::Message,
    task::Task,
};

pub fn update(
    model: &mut Model,
    msg: Message,
    audio: &mut Audio,
    backend: &Backend,
    task: &Task<Message>,
) -> Message {
    match msg {
        Message::None => Message::None,

        // ── Tick: advance playback state ────────────────────────────────────
        Message::Tick => {
            let ended = audio.tick();
            model.playback.status = audio.status;
            model.playback.position = audio.position;
            if ended {
                match model.playback.loop_mode {
                    LoopMode::One => {
                        if let Some(path) = model.playback.current_path.clone() {
                            match audio.play(&path) {
                                Ok(()) => model.playback.status = AudioStatus::Playing,
                                Err(e) => model.playback.error = Some(e.to_string()),
                            }
                        }
                    }
                    LoopMode::Off | LoopMode::Playlist => return Message::PlayNext,
                }
            }
            Message::None
        }

        // ── Auto-advance queue ──────────────────────────────────────────────
        Message::PlayNext => {
            let next_idx = model
                .queue
                .iter()
                .position(|t| Some(&t.id) == model.playback.current.as_ref().map(|c| &c.id))
                .map_or(0, |i| i + 1);
            if next_idx < model.queue.len() {
                let track = model.queue[next_idx].clone();
                start_playing(model, track, backend, task);
            } else if model.playback.loop_mode == LoopMode::Playlist && !model.queue.is_empty() {
                let track = model.queue[0].clone();
                start_playing(model, track, backend, task);
            } else {
                model.playback.status = AudioStatus::Idle;
                model.playback.current = None;
            }
            Message::None
        }

        // ── User-triggered queue skip (always wraps) ───────────────────────
        Message::SkipNext => {
            if model.queue.is_empty() {
                return Message::None;
            }
            let next_idx = model
                .queue
                .iter()
                .position(|t| Some(&t.id) == model.playback.current.as_ref().map(|c| &c.id))
                .map_or(0, |i| (i + 1) % model.queue.len());
            let track = model.queue[next_idx].clone();
            start_playing(model, track, backend, task);
            Message::None
        }

        Message::SkipPrev => {
            if model.queue.is_empty() {
                return Message::None;
            }
            let prev_idx = model
                .queue
                .iter()
                .position(|t| Some(&t.id) == model.playback.current.as_ref().map(|c| &c.id))
                .map(|i| if i == 0 { model.queue.len() - 1 } else { i - 1 })
                .unwrap_or(0);
            let track = model.queue[prev_idx].clone();
            start_playing(model, track, backend, task);
            Message::None
        }

        // ── Global ──────────────────────────────────────────────────────────
        Message::Quit => {
            model.should_quit = true;
            Message::None
        }
        Message::ToggleView => {
            model.view = match model.view {
                View::Search => View::Player,
                View::Player => View::Search,
            };
            Message::None
        }
        Message::TogglePause => {
            audio.toggle_pause();
            model.playback.status = audio.status;
            Message::None
        }
        Message::ToggleLoop => {
            model.playback.loop_mode = match model.playback.loop_mode {
                LoopMode::Off => LoopMode::One,
                LoopMode::One => LoopMode::Playlist,
                LoopMode::Playlist => LoopMode::Off,
            };
            Message::None
        }

        // ── Navigation routed by view ────────────────────────────────────
        Message::NavUp => {
            nav_prev(model);
            Message::None
        }
        Message::NavDown => {
            nav_next(model);
            Message::None
        }
        Message::FocusLeft => {
            if model.view == View::Player {
                model.player_focus = PlayerFocus::Library;
            }
            Message::None
        }
        Message::FocusRight => {
            if model.view == View::Player {
                model.player_focus = PlayerFocus::Queue;
            }
            Message::None
        }
        Message::DeleteFromLibrary => {
            if let Some(entry) = model.library.get(model.library_selected).cloned() {
                crate::library::delete_track(&backend.music_dir, &entry.id);
                model.library = crate::library::load_downloads(&backend.music_dir);
                if !model.library.is_empty() && model.library_selected >= model.library.len() {
                    model.library_selected = model.library.len() - 1;
                }
            }
            Message::None
        }

        Message::RemoveFromQueue => {
            if !model.queue.is_empty() {
                let idx = model.queue_selected.min(model.queue.len() - 1);
                model.queue.remove(idx);
                if !model.queue.is_empty() {
                    model.queue_selected = idx.min(model.queue.len() - 1);
                } else {
                    model.queue_selected = 0;
                }
            }
            Message::None
        }
        Message::Confirm => {
            handle_confirm(model, backend, task);
            Message::None
        }
        Message::Back => {
            handle_back(model);
            Message::None
        }

        // ── Search view ──────────────────────────────────────────────────
        Message::EnterSearch => {
            model.search_focus = SearchFocus::Input;
            Message::None
        }
        Message::SearchChar(c) => {
            model.query.push(c);
            Message::None
        }
        Message::SearchBackspace => {
            model.query.pop();
            Message::None
        }
        Message::SubmitSearch => {
            model.search_focus = SearchFocus::Results;
            if !model.query.is_empty() {
                let query = model.query.clone();
                task.spawn(async move {
                    let result = crate::ytdlp::search(&query, 10);
                    Message::SearchDone(result.map_err(|e| e.to_string()))
                });
            }
            Message::None
        }

        // ── Async results ─────────────────────────────────────────────────
        Message::SearchDone(Ok(results)) => {
            model.results = results;
            model.results_selected = 0;
            model.playback.error = None;
            Message::None
        }
        Message::SearchDone(Err(e)) => {
            model.playback.error = Some(e);
            Message::None
        }

        Message::AddToQueue => {
            if let Some(track) = model.results.tracks.get(model.results_selected).cloned() {
                if !model.queue.iter().any(|t| t.id == track.id) {
                    model.queue.push(track);
                }
            }
            Message::None
        }

        Message::DownloadReady(ref id, ref path) => {
            if model.playback.pending_id.as_ref() == Some(id) {
                model.playback.pending_id = None;
                model.playback.current_path = Some(path.clone());

                if let Some(ref track) = model.playback.current {
                    crate::library::save_sidecar(path, track);
                }

                model.library = crate::library::load_downloads(&backend.music_dir);
                if !model.library.is_empty() && model.library_selected >= model.library.len() {
                    model.library_selected = model.library.len() - 1;
                }

                match audio.play(path) {
                    Ok(()) => {
                        model.playback.status = AudioStatus::Playing;
                        model.playback.error = None;
                    }
                    Err(e) => {
                        model.playback.status = AudioStatus::Idle;
                        model.playback.error = Some(e.to_string());
                    }
                }
            }
            Message::None
        }
        Message::DownloadFailed(ref id, ref err) => {
            if model.playback.pending_id.as_ref() == Some(id) {
                model.playback.pending_id = None;
                model.playback.status = AudioStatus::Idle;
                model.playback.error = Some(err.clone());
            }
            Message::None
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn nav_prev(model: &mut Model) {
    match model.view {
        View::Search => {
            model.results_selected = model.results_selected.saturating_sub(1);
        }
        View::Player => match model.player_focus {
            PlayerFocus::Library => {
                model.library_selected = model.library_selected.saturating_sub(1);
            }
            PlayerFocus::Queue => {
                model.queue_selected = model.queue_selected.saturating_sub(1);
            }
        },
    }
}

fn nav_next(model: &mut Model) {
    match model.view {
        View::Search => {
            let max = model.results.tracks.len().saturating_sub(1);
            if model.results_selected < max {
                model.results_selected += 1;
            }
        }
        View::Player => match model.player_focus {
            PlayerFocus::Library => {
                let max = model.library.len().saturating_sub(1);
                if model.library_selected < max {
                    model.library_selected += 1;
                }
            }
            PlayerFocus::Queue => {
                let max = model.queue.len().saturating_sub(1);
                if model.queue_selected < max {
                    model.queue_selected += 1;
                }
            }
        },
    }
}

fn handle_confirm(model: &mut Model, backend: &Backend, task: &Task<Message>) {
    match model.view {
        View::Search => {
            if let Some(track) = model.results.tracks.get(model.results_selected).cloned() {
                start_playing(model, track, backend, task);
            }
        }
        View::Player => {
            let Some(entry) = model.library.get(model.library_selected).cloned() else {
                return;
            };
            let track = crate::models::Track {
                id: entry.id,
                title: entry.title,
                artist: entry.artist,
                album: None,
                duration: entry.duration_ms.map(std::time::Duration::from_millis),
                thumbnail: None,
            };
            if !model.queue.iter().any(|t| t.id == track.id) {
                model.queue.push(track.clone());
            }
            if model.playback.status == AudioStatus::Idle {
                start_playing(model, track, backend, task);
            }
        }
    }
}

fn handle_back(model: &mut Model) {
    match model.view {
        View::Search => {
            model.search_focus = SearchFocus::Results;
        }
        View::Player => {}
    }
}

fn start_playing(
    model: &mut Model,
    track: crate::models::Track,
    backend: &Backend,
    task: &Task<Message>,
) {
    model.playback.status = AudioStatus::Loading;
    model.playback.current = Some(track.clone());
    model.playback.pending_id = Some(track.id.clone());
    model.playback.error = None;

    let music_dir = backend.music_dir.clone();
    let track_id = track.id.clone();
    task.spawn(async move {
        let result = crate::ytdlp::ensure_local_audio(&music_dir, &track_id);
        match result {
            Ok(path) => Message::DownloadReady(track_id, path),
            Err(e) => Message::DownloadFailed(track_id, e.to_string()),
        }
    });
}
