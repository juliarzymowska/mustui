use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::models::TrackId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaylistEntry {
    pub id: TrackId,
    pub title: String,
    pub artist: String,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<PlaylistEntry>,
}

pub struct PlaylistStore {
    dir: PathBuf,
}

impl PlaylistStore {
    pub fn new(dir: PathBuf) -> std::io::Result<Self> {
        fs::create_dir_all(&dir)?;
        Ok(Self { dir })
    }

    pub fn load_all(&self) -> Vec<Playlist> {
        let Ok(entries) = fs::read_dir(&self.dir) else { return vec![] };
        entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
            .filter_map(|e| {
                let text = fs::read_to_string(e.path()).ok()?;
                serde_json::from_str(&text).ok()
            })
            .collect()
    }

    pub fn save(&self, playlist: &Playlist) -> std::io::Result<()> {
        let path = self.playlist_path(&playlist.name);
        let json = serde_json::to_string_pretty(playlist)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, json)
    }

    pub fn create(&self, name: &str) -> std::io::Result<Playlist> {
        let playlist = Playlist { name: name.to_owned(), tracks: vec![] };
        self.save(&playlist)?;
        Ok(playlist)
    }

    pub fn delete(&self, name: &str) -> std::io::Result<()> {
        let path = self.playlist_path(name);
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn add_track(&self, playlist_name: &str, entry: PlaylistEntry) -> std::io::Result<()> {
        let mut playlist = self.load(playlist_name)?;
        if !playlist.tracks.iter().any(|t| t.id == entry.id) {
            playlist.tracks.push(entry);
            self.save(&playlist)?;
        }
        Ok(())
    }

    pub fn remove_track(&self, playlist_name: &str, id: &TrackId) -> std::io::Result<()> {
        let mut playlist = self.load(playlist_name)?;
        playlist.tracks.retain(|t| &t.id != id);
        self.save(&playlist)
    }

    fn load(&self, name: &str) -> std::io::Result<Playlist> {
        let text = fs::read_to_string(self.playlist_path(name))?;
        serde_json::from_str(&text)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }

    fn playlist_path(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{}.json", sanitize_name(name)))
    }
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
