use std::sync::Arc;

use ratatui_image::picker::Picker;

use crate::{
    audio::Audio,
    client::Backend,
    model::{AudioStatus, InputMode, LoopMode, Model},
    msg::Message,
    task::Task,
};

pub fn update(
    model: &mut Model,
    msg: Message,
    audio: &mut Audio,
    backend: &Backend,
    task: &Task<Message>,
    picker: &Option<Picker>,
) -> Message {
    match msg {
        Message::None => Message::None,

        Message::Tick => {
            let ended = audio.tick();
            model.playback.status = audio.status;
            model.playback.position = audio.position;
            if ended {
                if model.playback.loop_mode == LoopMode::One {
                    if let Some(path) = model.playback.current_path.clone() {
                        match audio.play(&path) {
                            Ok(()) => model.playback.status = AudioStatus::Playing,
                            Err(e) => model.playback.error = Some(e.to_string()),
                        }
                    }
                }
            }
            Message::None
        }

        Message::Quit => {
            model.should_quit = true;
            Message::None
        }

        Message::EnterSearch => {
            model.mode = InputMode::Searching;
            model.query.clear();
            Message::None
        }
        Message::CancelSearch => {
            model.mode = InputMode::Normal;
            Message::None
        }
        Message::SubmitSearch => {
            model.mode = InputMode::Normal;
            if !model.query.is_empty() {
                let query = model.query.clone();
                task.spawn(async move {
                    let result = crate::ytdlp::search(&query, 10);
                    Message::SearchDone(result.map_err(|e| e.to_string()))
                });
            }
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

        Message::SelectNext => {
            if model.selected + 1 < model.results.tracks.len() {
                model.selected += 1;
            }
            Message::None
        }
        Message::SelectPrev => {
            model.selected = model.selected.saturating_sub(1);
            Message::None
        }

        Message::PlaySelected => {
            if let Some(track) = model.results.tracks.get(model.selected).cloned() {
                model.playback.status = AudioStatus::Loading;
                model.playback.current = Some(track.clone());
                model.playback.current_path = None;
                model.playback.error = None;
                model.artwork = None;

                let music_dir = backend.music_dir.clone();
                let track_id = track.id.clone();
                task.spawn(async move {
                    let result = crate::ytdlp::ensure_local_audio(&music_dir, &track_id);
                    match result {
                        Ok(path) => Message::DownloadReady(track_id, path),
                        Err(e) => Message::DownloadFailed(track_id, e.to_string()),
                    }
                });

                if let Some(thumb) = &track.thumbnail {
                    let b = backend.clone();
                    let tid = track.id.clone();
                    let url = thumb.url.clone();
                    task.spawn(async move {
                        match crate::artwork::fetch_artwork(&b, &url).await {
                            Ok(img) => Message::ArtworkReady(tid, Arc::new(img)),
                            Err(_) => Message::ArtworkFailed(tid),
                        }
                    });
                }
            }
            Message::None
        }

        Message::DownloadReady(_, path) => {
            model.playback.current_path = Some(path.clone());
            match audio.play(&path) {
                Ok(()) => {
                    model.playback.status = AudioStatus::Playing;
                    model.playback.error = None;
                }
                Err(e) => {
                    model.playback.status = AudioStatus::Idle;
                    model.playback.error = Some(e.to_string());
                }
            }
            Message::None
        }
        Message::DownloadFailed(_, err) => {
            model.playback.status = AudioStatus::Idle;
            model.playback.error = Some(err);
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
                LoopMode::One => LoopMode::Off,
            };
            Message::None
        }

        Message::SearchDone(Ok(results)) => {
            model.results = results;
            model.selected = 0;
            model.playback.error = None;
            Message::None
        }
        Message::SearchDone(Err(e)) => {
            model.playback.error = Some(e);
            Message::None
        }

        Message::ArtworkReady(_, img) => {
            model.artwork = picker.as_ref().map(|p| p.new_resize_protocol((*img).clone()));
            Message::None
        }
        Message::ArtworkFailed(_) => {
            model.artwork = None;
            Message::None
        }
    }
}
