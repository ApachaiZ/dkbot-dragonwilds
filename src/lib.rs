//! Plugin dkbot — administration d'un serveur RuneScape: Dragonwilds via
//! RSDWServerAPI. ABI `dkbot-host-api` v1 : commandes JSON en entrée/sortie.
//!
//! Commandes : `status, players, player, items, announce, kick, ban, unban,
//! bans, give, tp, tpto, tell, heal, feed, hydrate, godmode, top, cheats`.

mod cards;
mod cheats;
mod leaderboard;
mod rsdw;

use std::ffi::c_char;

use dkbot_host_api::{export_plugin, PluginVtable};

static NAME: &[u8] = b"dragonwilds\0";
// Manifeste des commandes — l'hôte l'utilise pour enregistrer les vraies
// slash commands Discord (/dw status, /dw tp from=… to=…, etc.).
static COMMANDS: &str = r#"{
  "group": "dw",
  "description": "Administration du serveur Dragonwilds",
  "commands": [
    {"name":"status","description":"Identité du serveur, monde, état du moteur","args":[]},
    {"name":"players","description":"Joueurs connectés (détail complet)","args":[]},
    {"name":"player","description":"Fiche d'un joueur","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true}]},
    {"name":"items","description":"Objets du jeu","args":[{"name":"search","description":"Filtre (facultatif)","kind":"string"}]},
    {"name":"announce","description":"Message global","args":[{"name":"message","description":"Texte","kind":"string","required":true}]},
    {"name":"kick","description":"Expulser un joueur","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true}]},
    {"name":"ban","description":"Bannir un joueur","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true}]},
    {"name":"unban","description":"Lever un bannissement","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true}]},
    {"name":"bans","description":"Bannissements persistants","args":[]},
    {"name":"give","description":"Donner un objet","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true},{"name":"item","description":"Nom de l'objet","kind":"string","required":true},{"name":"count","description":"Quantité (défaut 1)","kind":"int"}]},
    {"name":"tp","description":"Téléporter un joueur sur un autre","args":[{"name":"from","description":"Joueur source","kind":"string","required":true},{"name":"to","description":"Joueur cible","kind":"string","required":true}]},
    {"name":"tpto","description":"Téléporter vers des coordonnées","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true},{"name":"x","description":"X","kind":"string","required":true},{"name":"y","description":"Y","kind":"string","required":true},{"name":"z","description":"Z","kind":"string","required":true}]},
    {"name":"tell","description":"Message privé en jeu","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true},{"name":"message","description":"Texte","kind":"string","required":true}]},
    {"name":"heal","description":"Soigner un joueur","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true}]},
    {"name":"feed","description":"Nourrir un joueur","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true}]},
    {"name":"hydrate","description":"Hydrater un joueur","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true}]},
    {"name":"godmode","description":"Bascule le god mode","args":[{"name":"player","description":"Nom ou identifiant","kind":"string","required":true},{"name":"enabled","description":"on/off (défaut on)","kind":"bool"}]},
    {"name":"top","description":"Classement XP","args":[{"name":"n","description":"Nombre (défaut 10)","kind":"int"}]},
    {"name":"cheats","description":"Switch cheat (owner)","args":[{"name":"action","description":"get ou set","kind":"string"},{"name":"enabled","description":"on/off","kind":"bool"}]}
  ]
}"#;

// plugin_list() devient paresseux (conversion CString une fois)
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
    // Si le message est déjà un objet JSON d'erreur (client HTTP…), on ne
    // l'emballe pas une seconde fois.
    if message.trim_start().starts_with("{") {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(message) {
            if v.get("error").is_some() {
                return message.to_string();
            }
        }
    }
    format!("{{\"error\":\"{}\"}}", message.replace('\\', "\\\\").replace('"', "\\\""))
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
        "status" => {
            let status: serde_json::Value =
                serde_json::from_str(&client.get("/api/status")?).unwrap_or(serde_json::json!({}));
            let health: serde_json::Value =
                serde_json::from_str(&client.get("/api/health")?).unwrap_or(serde_json::json!({}));
            let players: serde_json::Value =
                serde_json::from_str(&client.get("/api/players")?).unwrap_or(serde_json::json!({}));
            Ok(serde_json::json!({
                "status": status,
                "health": health,
                "count": players.get("count").cloned().unwrap_or(serde_json::json!(0)),
            })
            .to_string())
        }
        "players" => client.get("/api/players"),
        "items" => {
            let search = arg_str(&args, "search").unwrap_or_default();
            if search.is_empty() {
                client.get("/api/items")
            } else {
                client.get(&format!("/api/items?search={}", urlencode(&search)))
            }
        }
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
    use std::sync::OnceLock;
    static C: OnceLock<std::ffi::CString> = OnceLock::new();
    C.get_or_init(|| std::ffi::CString::new(COMMANDS).expect("manifeste valide"))
        .as_ptr()
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
        Ok(body) => cards::cardify(&cmd, &body),
        Err(message) => cards::cardify(&cmd, &err(&message)),
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
