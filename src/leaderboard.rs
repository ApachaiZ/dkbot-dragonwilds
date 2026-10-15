//! Classement XP — instantanés fusionnés à chaque lecture (simple et synchrone).

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const MAX_AGE_DAYS: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub total_xp: f64,
    pub last_seen: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub players: HashMap<String, Entry>,
}

fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn state_file() -> PathBuf {
    let dir = std::env::var("DKBOT_PLUGIN_STATE_DIR")
        .unwrap_or_else(|_| ".config.d/plugins/dragonwilds".to_string());
    PathBuf::from(dir)
}

fn snapshot_path() -> PathBuf {
    state_file().join("leaderboard.json")
}

pub fn load() -> Snapshot {
    std::fs::read_to_string(snapshot_path())
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

pub fn save(snapshot: &Snapshot) {
    let path = snapshot_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let tmp = path.with_extension("tmp");
    if let Ok(data) = serde_json::to_vec_pretty(snapshot) {
        if std::fs::write(&tmp, data).is_ok() {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

/// Fusionne une entrée joueur (XP jamais décroissante).
pub fn merge_player(snapshot: &mut Snapshot, net_id: &str, name: &str, xp: f64) {
    let now = now_unix();
    let entry = snapshot.players.entry(net_id.to_string()).or_insert(Entry {
        name: name.to_string(),
        total_xp: 0.0,
        last_seen: now,
    });
    if xp > entry.total_xp {
        entry.total_xp = xp;
    }
    if !name.is_empty() {
        entry.name = name.to_string();
    }
    entry.last_seen = now;
    snapshot
        .players
        .retain(|_, e| now.saturating_sub(e.last_seen) < MAX_AGE_DAYS * 86400);
}

/// Top N, trié par XP décroissante.
pub fn top(snapshot: &Snapshot, n: usize) -> Vec<(String, f64)> {
    let mut list: Vec<(String, f64)> = snapshot
        .players
        .values()
        .map(|e| (e.name.clone(), e.total_xp))
        .collect();
    list.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    list.truncate(n);
    list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_never_decreases() {
        let mut snap = Snapshot::default();
        merge_player(&mut snap, "a", "A", 100.0);
        merge_player(&mut snap, "a", "A2", 80.0);
        assert_eq!(snap.players["a"].total_xp, 100.0);
        assert_eq!(snap.players["a"].name, "A2");
    }

    #[test]
    fn top_sorts_desc() {
        let mut snap = Snapshot::default();
        merge_player(&mut snap, "a", "A", 50.0);
        merge_player(&mut snap, "b", "B", 200.0);
        let rows = top(&snap, 10);
        assert_eq!(rows[0].0, "B");
        assert_eq!(rows[1].0, "A");
    }
}
