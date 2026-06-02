mod actor;
mod source;

use crate::{client::Backend, messages::{PlayerCommand, PlayerState}, CoreError, Result};

pub struct PlayerHandle {
    cmd_tx: tokio::sync::mpsc::Sender<PlayerCommand>,
    state_tx: tokio::sync::broadcast::Sender<PlayerState>,
}

impl PlayerHandle {
    pub fn commands(&self) -> tokio::sync::mpsc::Sender<PlayerCommand> {
        self.cmd_tx.clone()
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<PlayerState> {
        self.state_tx.subscribe()
    }

    pub async fn send(&self, cmd: PlayerCommand) -> Result<()> {
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| CoreError::PlayerDisconnected)
    }
}

pub(crate) fn spawn(backend: Backend) -> PlayerHandle {
    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(32);
    let (state_tx, _) = tokio::sync::broadcast::channel(16);
    let state_tx_thread = state_tx.clone();
    let rt_handle = tokio::runtime::Handle::current();

    std::thread::Builder::new()
        .name("ytm-player".into())
        .spawn(move || {
            actor::run(actor::ActorChannels {
                cmd_rx,
                state_tx: state_tx_thread,
                backend,
                rt_handle,
            })
        })
        .expect("player thread spawn");

    PlayerHandle { cmd_tx, state_tx }
}
