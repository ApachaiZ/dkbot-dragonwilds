//! Plugin dkbot — administration d'un serveur RuneScape: Dragonwilds via
//! RSDWServerAPI. ABI `dkbot-host-api` v1 : commandes JSON en entrée/sortie.
//!
//! Commandes : `status, players, player, items, announce, kick, ban, unban,
//! bans, give, tp, tpto, tell, heal, feed, hydrate, godmode, top, cheats`.

mod cheats;
mod leaderboard;
mod rsdw;

use std::ffi::c_char;

use dkbot_host_api::{export_plugin, PluginVtable};

static NAME: &[u8] = b"dragonwilds\0";
static COMMANDS: &[u8] = b"[\"status\",\"players\",\"player\",\"items\",\"announce\",\"kick\",\"ban\",\"unban\",\"bans\",\"give\",\"tp\",\"tpto\",\"tell\",\"heal\",\"feed\",\"hydrate\",\"godmode\",\"top\",\"cheats\"]\0";

// ── Autorisation ──────────────────────────────────────────────────────────

struct Caller {
    is_owner: bool,
    roles: Vec<String>,
}

fn parse_caller(args_json: &str) -> (Caller, serde_json::Value) {
    let root: serde_json::Value = serde_json::from_str(args_json).unwrap_or(serde_json::json!({}));
    let caller = root
        .get("caller")
        .cloned()
        .unwrap_or(serde_json::json!({ "is_owner": true }));
    let is_owner = caller.get("is_owner").and_then(|b| b.as_bool()).unwrap_or(false);
    let roles: Vec<String> = caller
        .get("roles")
        .and_then(|r| r.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let args = root.get("args").cloned().unwrap_or(serde_json::json!({}));
    (Caller { is_owner, roles }, args)
}

fn env_ids(name: &str) -> Vec<String> {
    std::env::var(name)
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .collect()
}

impl Caller {
    fn is_admin(&self) -> bool {
        self.is_owner || self.roles.iter().any(|r| env_ids("RSDW_ADMIN_ROLE_IDS").contains(r))
    }
    fn is_mod(&self) -> bool {
        self.is_admin()
            || (env_ids("RSDW_MOD_ROLE_IDS").is_empty() && self.is_owner)
            || self.roles.iter().any(|r| env_ids("RSDW_MOD_ROLE_IDS").contains(r))
    }
}

// ── Helpers JSON ──────────────────────────────────────────────────────────

fn err(message: &str) -> String {
    format!("{{\"error\":\"{}\"}}", message.replace('\\', "\\\\").replace('"', "\\\""))
}

fn ok_passthrough(body: String) -> String {
    body
}

fn arg_str(args: &serde_json::Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(String::from).filter(|s| !s.is_empty())
}

// ── Dispatch ──────────────────────────────────────────────────────────────

fn run_command(cmd: &str, args_json: &str) -> Result<String, String> {
    let (caller, args) = parse_caller(args_json);
    let mut switch = cheats::CheatSwitch::load();

    if cmd == "cheats" {
        // Propriétaire uniquement : lecture ou bascule du switch.
        if !caller.is_owner {
            return Err("cheats : réservé au propriétaire".into());
        }
        let action = arg_str(&args, "action").unwrap_or_else(|| "get".into());
        return match action.as_str() {
            "get" => Ok(format!("{{\"cheats_enabled\":{}}}", switch.enabled())),
            "set" => {
                let enabled = args.get("enabled").and_then(|b| b.as_bool()).unwrap_or(true);
                switch.set(enabled);
                Ok(format!(
                    "{{\"ok\":true,\"cheats_enabled\":{enabled},\"message\":\"Fonctionnalités cheat {}\"}}",
                    if enabled { "activées" } else { "désactivées" }
                ))
            }
            _ => Err("cheats : action inconnue (get|set)".into()),
        };
    }

    if !cheat_allowed(cmd, switch.enabled()) {
        return Err("fonctionnalités cheat désactivées par le propriétaire".into());
    }

    const KNOWN: &[&str] = &[
        "status", "players", "player", "items", "announce", "kick", "ban", "unban", "bans",
        "give", "tp", "tpto", "tell", "heal", "feed", "hydrate", "godmode", "top",
    ];
    if !KNOWN.contains(&cmd) {
        return Err(format!("commande inconnue : {cmd}"));
    }

    let client = rsdw::Client::new().map_err(|e| e.to_json())?;
    if !client.is_configured() {
        return Err("API non configurée — renseigner RSDW_API_TOKEN".into());
    }

    // Lecture / modération / admin — niveaux miroirs de l'ancien module.
    let needs_mod = matches!(
        cmd,
        "status" | "players" | "player" | "items" | "announce" | "kick" | "ban" | "unban" | "bans" | "top"
    );
    if needs_mod && !caller.is_mod() {
        return Err("accès refusé : niveau modérateur requis".into());
    }
    if !needs_mod && !caller.is_admin() {
        return Err("accès refusé : niveau administrateur requis".into());
    }

    match cmd {
        "status" => client.get("/api/status"),
        "players" => client.get("/api/players"),
        "items" => client.get("/api/items"),
        "bans" => client.get("/api/bans"),
        "player" => {
            let name = arg_str(&args, "player").ok_or("player : argument manquant")?;
            client
                .get(&format!("/api/players?name={}", urlencode(&name)))
                
        }
        "announce" => {
            let message = arg_str(&args, "message").ok_or("announce : message manquant")?;
            let body = serde_json::json!({ "message": message }).to_string();
            client.post("/api/broadcast", &body)
        }
        "kick" => {
            let player = arg_str(&args, "player").ok_or("kick : player manquant")?;
            let body = serde_json::json!({ "player": player }).to_string();
            client.post("/api/kick", &body)
        }
        "ban" => {
            let player = arg_str(&args, "player").ok_or("ban : player manquant")?;
            let body = serde_json::json!({ "player": player }).to_string();
            client.post("/api/ban", &body)
        }
        "unban" => {
            let player = arg_str(&args, "player").ok_or("unban : player manquant")?;
            let body = serde_json::json!({ "player": player }).to_string();
            client.post("/api/unban", &body)
        }
        "give" => {
            let player = arg_str(&args, "player").ok_or("give : player manquant")?;
            let item = arg_str(&args, "item").ok_or("give : item manquant")?;
            let count = args.get("count").and_then(|c| c.as_i64()).unwrap_or(1);
            let body = serde_json::json!({ "player": player, "item": item, "count": count }).to_string();
            client.post("/api/give", &body)
        }
        "tp" => {
            let from = arg_str(&args, "from").ok_or("tp : from manquant")?;
            let to = arg_str(&args, "to").ok_or("tp : to manquant")?;
            let body = serde_json::json!({ "from": from, "to": to }).to_string();
            client.post("/api/teleport", &body)
        }
        "tpto" => {
            let player = arg_str(&args, "player").ok_or("tpto : player manquant")?;
            let (x, y, z) = (
                args.get("x").and_then(|v| v.as_f64()).ok_or("tpto : x manquant")?,
                args.get("y").and_then(|v| v.as_f64()).ok_or("tpto : y manquant")?,
                args.get("z").and_then(|v| v.as_f64()).ok_or("tpto : z manquant")?,
            );
            let body = serde_json::json!({ "player": player, "x": x, "y": y, "z": z }).to_string();
            client.post("/api/tpto", &body)
        }
        "tell" => {
            let player = arg_str(&args, "player").ok_or("tell : player manquant")?;
            let message = arg_str(&args, "message").ok_or("tell : message manquant")?;
            let body = serde_json::json!({ "player": player, "message": message }).to_string();
            client.post("/api/tell", &body)
        }
        "heal" | "feed" | "hydrate" => {
            let player = arg_str(&args, "player").ok_or("player manquant")?;
            let body = serde_json::json!({ "player": player }).to_string();
            client
                .post(&format!("/api/{cmd}"), &body)
                
        }
        "godmode" => {
            let player = arg_str(&args, "player").ok_or("godmode : player manquant")?;
            let enabled = args.get("enabled").and_then(|b| b.as_bool()).unwrap_or(true);
            let body = serde_json::json!({ "player": player, "enabled": enabled }).to_string();
            client.post("/api/godmode", &body)
        }
        "top" => {
            let n = args
                .get("n")
                .and_then(|v| v.as_u64())
                .unwrap_or(10)
                .clamp(1, 25) as usize;
            let players_json = client.get("/api/players")?;
            let players: serde_json::Value = serde_json::from_str(&players_json)
                .map_err(|e| format!("réponse players illisible : {e}"))?;
            let mut snapshot = leaderboard::load();
            if let Some(list) = players.get("players").and_then(|p| p.as_array()) {
                for player in list {
                    let net_id = player
                        .get("uniqueNetId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    if net_id.is_empty() {
                        continue;
                    }
                    let name = player
                        .get("characterName")
                        .and_then(|v| v.as_str())
                        .map(String::from)
                        .or_else(|| player.get("name").and_then(|v| v.as_str()).map(String::from))
                        .unwrap_or_else(|| net_id.clone());
                    let xp: f64 = player
                        .get("skills")
                        .and_then(|s| s.as_array())
                        .map(|skills| {
                            skills
                                .iter()
                                .filter_map(|s| s.get("currentXP").and_then(|v| v.as_f64()))
                                .sum()
                        })
                        .unwrap_or(0.0);
                    leaderboard::merge_player(&mut snapshot, &net_id, &name, xp);
                }
            }
            leaderboard::save(&snapshot);
            let rows: Vec<serde_json::Value> = leaderboard::top(&snapshot, n)
                .into_iter()
                .map(|(name, xp)| serde_json::json!({ "name": name, "xp": xp }))
                .collect();
            Ok(serde_json::json!({ "top": rows }).to_string())
        }
        _ => Err(format!("commande inconnue : {cmd}")),
    }
}

/// Commande « cheat » autorisée ? (fonction pure — testable sans état).
fn cheat_allowed(cmd: &str, enabled: bool) -> bool {
    !cheats::CHEAT_COMMANDS.contains(&cmd) || enabled
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

// ── Entrée ABI ────────────────────────────────────────────────────────────

unsafe extern "C" fn plugin_name() -> *const c_char {
    NAME.as_ptr() as *const c_char
}

unsafe extern "C" fn plugin_list() -> *const c_char {
    COMMANDS.as_ptr() as *const c_char
}

unsafe extern "C" fn plugin_run(
    cmd: *const c_char,
    args_json: *const c_char,
    out: *mut u8,
    out_cap: usize,
) -> i32 {
    let cmd = dkbot_host_api::read_cstr(cmd);
    let args_json = dkbot_host_api::read_cstr(args_json);
    let response = match run_command(&cmd, &args_json) {
        Ok(body) => body,
        Err(message) => err(&message),
    };
    // SAFETY : out est valide pour out_cap octets (contrat hôte).
    unsafe { dkbot_host_api::write_out(out, out_cap, response.as_bytes()) }
}

unsafe extern "C" fn plugin_shutdown() {}

static VTABLE: PluginVtable = PluginVtable {
    abi_version: dkbot_host_api::ABI_VERSION,
    name: plugin_name,
    list: plugin_list,
    run: plugin_run,
    shutdown: plugin_shutdown,
};

export_plugin!(&VTABLE);

#[cfg(test)]
mod tests {
    use super::*;

    fn owner_args() -> String {
        r#"{"caller":{"id":"1","is_owner":true,"roles":[]},"args":{}}"#.to_string()
    }

    #[test]
    fn cheats_get_requires_owner() {
        let (caller, _) = parse_caller(&owner_args());
        assert!(caller.is_owner);
        let (caller, _) = parse_caller(r#"{"caller":{"id":"2","is_owner":false,"roles":[]},"args":{}}"#);
        assert!(!caller.is_owner);
    }

    #[test]
    fn cheat_gate_blocks_when_disabled() {
        assert!(cheat_allowed("tp", true));
        assert!(!cheat_allowed("tp", false));
        assert!(!cheat_allowed("godmode", false));
        assert!(cheat_allowed("status", false)); // non-cheat : toujours permis
    }

    #[test]
    fn unknown_command_is_an_error() {
        let result = run_command("nimporte", &owner_args());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("inconnue"));
    }

    #[test]
    fn urlencode_keeps_safe_and_encodes_spaces() {
        assert_eq!(urlencode("ApachaiZ"), "ApachaiZ");
        assert_eq!(urlencode("a b"), "a%20b");
    }

    #[test]
    fn unconfigured_client_errors_before_network() {
        // Sans token dans l'environnement, une commande admin échoue proprement.
        let result = run_command("heal", &owner_args());
        assert!(result.is_err());
    }
}
