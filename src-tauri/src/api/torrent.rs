use librqbit::{Session, SessionOptions, AddTorrent, AddTorrentOptions, AddTorrentResponse};
use std::sync::Arc;
use tauri::State;
use serde::Serialize;
use std::path::PathBuf;

pub struct TorrentState {
    pub session: Arc<Session>,
}

#[derive(Serialize, Clone)]
pub struct TorrentInfo {
    pub id: usize,
    pub name: Option<String>,
    pub progress: f64,
    pub state: String,
    pub total_bytes: u64,
    pub finished_bytes: u64,
    pub download_speed: f64,
}

pub async fn init_torrent_state() -> TorrentState {
    let download_dir = dirs::download_dir().unwrap_or_else(|| PathBuf::from("downloads")).join("torrent");
    std::fs::create_dir_all(&download_dir).ok();

    let options = SessionOptions::default();

    let session = Session::new_with_opts(download_dir, options).await.expect("Failed to start torrent session");
    TorrentState { session }
}

#[tauri::command]
pub async fn add_torrent_file(path: String, state: State<'_, TorrentState>) -> Result<usize, String> {
    let add_opts = AddTorrentOptions::default();
    let add_torrent = AddTorrent::from_cli_argument(&path).map_err(|e| e.to_string())?;
    
    let handle = state.session.add_torrent(add_torrent, Some(add_opts)).await.map_err(|e| e.to_string())?;
    
    match handle {
        AddTorrentResponse::AlreadyManaged(id, _) => Ok(id),
        AddTorrentResponse::Added(id, _) => Ok(id),
        _ => Err("Unknown response".to_string()),
    }
}

#[tauri::command]
pub async fn get_torrents(state: State<'_, TorrentState>) -> Result<Vec<TorrentInfo>, String> {
    let result = state.session.with_torrents(|torrents| {
        let mut result = Vec::new();
        for (id, handle) in torrents {
            let metadata_guard = handle.metadata.load();
            
            let (total_bytes, name) = if let Some(meta) = metadata_guard.as_ref() {
                let name = meta.name.clone();
                // iter_file_lengths returns Result<Iterator, Error>
                let total = if let Ok(iter) = meta.info.iter_file_lengths() {
                    iter.sum::<u64>()
                } else {
                    0
                };
                (total, name)
            } else {
                (0, None)
            };

            let stats = handle.stats();
            
            let finished = stats.progress_bytes;
            let progress = if total_bytes > 0 { (finished as f64 / total_bytes as f64) * 100.0 } else { 0.0 };
            
            let download_speed = stats.live.as_ref()
                .map(|l| l.download_speed.mbps * 125_000.0)
                .unwrap_or(0.0);

            result.push(TorrentInfo {
                id,
                name,
                progress,
                state: format!("{:?}", stats.state),
                total_bytes,
                finished_bytes: finished,
                download_speed,
            });
        }
        result
    });
    
    Ok(result)
}
