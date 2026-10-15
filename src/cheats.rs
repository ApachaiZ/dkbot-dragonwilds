//! Switch « cheat » — propriétaire uniquement. Active/désactive les
//! fonctionnalités tp/tpto/give/godmode/heal/feed/hydrate du plugin.
//! État persisté en JSON dans le dossier d'état du plugin.

use std::path::PathBuf;

pub const CHEAT_COMMANDS: &[&str] = &["tp", "tpto", "give", "godmode", "heal", "feed", "hydrate"];

pub struct CheatSwitch {
    path: PathBuf,
    enabled: bool,
}

impl CheatSwitch {
    pub fn load() -> Self {
        let path = state_file();
        let enabled = std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|v| v.get("cheats_enabled").and_then(|b| b.as_bool()))
            .unwrap_or(true); // défaut : activées (comportement historique)
        CheatSwitch { path, enabled }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set(&mut self, enabled: bool) {
        self.enabled = enabled;
        let data = serde_json::json!({ "cheats_enabled": enabled }).to_string();
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // écriture atomique : fichier temporaire puis rename
        let tmp = self.path.with_extension("tmp");
        if std::fs::write(&tmp, data).is_ok() {
            let _ = std::fs::rename(&tmp, &self.path);
        }
    }
}

fn state_file() -> PathBuf {
    let dir = std::env::var("DKBOT_PLUGIN_STATE_DIR")
        .unwrap_or_else(|_| ".config.d/plugins/dragonwilds".to_string());
    PathBuf::from(dir).join("state.json")
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// Verrou partagé avec lib.rs : les tests qui touchent au fichier d'état
    /// du switch (variable d'environnement process-globale) se sérialisent.
    pub static SWITCH_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn defaults_to_enabled() {
        let _guard = SWITCH_LOCK.lock().expect("verrou switch");
        let sw = CheatSwitch::load();
        assert!(sw.enabled());
    }

    #[test]
    fn cheat_commands_are_listed() {
        assert!(CHEAT_COMMANDS.contains(&"tp"));
        assert!(CHEAT_COMMANDS.contains(&"godmode"));
        assert!(!CHEAT_COMMANDS.contains(&"status"));
    }
}
