use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

use crate::model::AudioStatus;

pub struct Audio {
    sink: MixerDeviceSink,
    player: Option<Player>,
    pub status: AudioStatus,
    pub position: Duration,
}

impl Audio {
    pub fn new() -> anyhow::Result<Self> {
        let mut sink = DeviceSinkBuilder::open_default_sink()?;
        sink.log_on_drop(false);
        Ok(Self { sink, player: None, status: AudioStatus::Idle, position: Duration::ZERO })
    }

    pub fn play(&mut self, path: &Path) -> anyhow::Result<()> {
        self.stop();
        let file = File::open(path)?;
        let decoder = Decoder::new(BufReader::new(file))?;
        let player = Player::connect_new(self.sink.mixer());
        player.append(decoder);
        player.play();
        self.player = Some(player);
        self.status = AudioStatus::Playing;
        self.position = Duration::ZERO;
        Ok(())
    }

    pub fn toggle_pause(&mut self) {
        if let Some(ref player) = self.player {
            if player.is_paused() {
                player.play();
                self.status = AudioStatus::Playing;
            } else {
                player.pause();
                self.status = AudioStatus::Paused;
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(p) = self.player.take() {
            p.stop();
        }
        self.status = AudioStatus::Idle;
        self.position = Duration::ZERO;
    }

    /// Polls playback position; returns `true` if the track just finished.
    pub fn tick(&mut self) -> bool {
        let player = match &self.player {
            Some(p) => p,
            None => return false,
        };
        if self.status != AudioStatus::Playing {
            return false;
        }
        self.position = player.get_pos();
        if player.empty() {
            self.player = None;
            self.status = AudioStatus::Idle;
            self.position = Duration::ZERO;
            return true;
        }
        false
    }
}
