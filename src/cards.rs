//! Cards: turns each command's raw API JSON into a "card" object
//! (`{"card": {title, description, tone, fields, footer}}`) that the host
//! renders as a standard Discord embed — same look as the core commands.
//!
//! Convention (host ≥ 0.4.2): if a plugin result contains a `card` key, the
//! host renders it as an embed; any other JSON falls back to a code block.

use serde_json::{json, Value};

const MAX_FIELDS: usize = 25; // Discord embed limit

pub fn cardify(cmd: &str, raw: &str) -> String {
    let parsed: Value = serde_json::from_str(raw).unwrap_or_else(|_| json!({}));
    if let Some(error) = parsed.get("error").and_then(|v| v.as_str()) {
        return card(&format!("❌ {}", cmd_title(cmd)), error, "bad", &[]);
    }
    match cmd {
        // Réponse interne d'autocomplétion (tableau de chaînes) : transmise
        // telle quelle à l'hôte, pas une carte.
        "__ac" => raw.to_string(),
        "status" => status_card(&parsed),
        "players" => players_card(&parsed),
        "player" => player_card(&parsed),
        "items" => items_card(&parsed),
        "bans" => bans_card(&parsed),
        "top" => top_card(&parsed),
        "cheats" => cheats_card(&parsed),
        _ => ok_card(cmd, &parsed),
    }
}

fn cmd_title(cmd: &str) -> String {
    match cmd {
        "status" => "🐉 Statut du serveur".into(),
        "players" => "👥 Joueurs connectés".into(),
        "player" => "👤 Fiche joueur".into(),
        "items" => "🎒 Objets".into(),
        "bans" => "🚫 Bannissements".into(),
        "announce" => "📢 Annonce".into(),
        "kick" => "👢 Expulsion".into(),
        "ban" => "🔨 Bannissement".into(),
        "unban" => "🕊️ Débannissement".into(),
        "give" => "🎁 Don d'objet".into(),
        "tp" => "🔄 Téléportation".into(),
        "tpto" => "📍 Téléportation".into(),
        "tell" => "💬 Message privé".into(),
        "heal" => "💚 Soin".into(),
        "feed" => "🍖 Nourriture".into(),
        "hydrate" => "💧 Hydratation".into(),
        "godmode" => "🛡️ God Mode".into(),
        "top" => "🏆 Classement XP".into(),
        "cheats" => "🎛️ Fonctionnalités cheat".into(),
        _ => "🔌 Plugin".into(),
    }
}

fn card(title: &str, description: &str, tone: &str, fields: &[(String, String, bool)]) -> String {
    let mut card = serde_json::Map::new();
    card.insert("title".into(), json!(title));
    card.insert("description".into(), json!(description));
    card.insert("tone".into(), json!(tone));
    let fields: Vec<Value> = fields
        .iter()
        .map(|(name, value, inline)| {
            json!({ "name": name, "value": value, "inline": inline })
        })
        .collect();
    card.insert("fields".into(), json!(fields));
    card.insert("footer".into(), json!("Dragonwilds • DakkaBot"));
    json!({ "card": card }).to_string()
}

fn f_str(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

fn f_num(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(|x| x.as_f64()).unwrap_or(0.0)
}

/// 1234567.8 → "1 234 567.8"
fn fmt_num(n: f64) -> String {
    let neg = n < 0.0;
    let s = format!("{:.1}", n.abs());
    let (int, frac) = match s.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (s.as_str(), None),
    };
    let mut out = String::new();
    for (i, c) in int.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(' ');
        }
        out.push(c);
    }
    let mut res: String = out.chars().rev().collect();
    if let Some(f) = frac {
        if f != "0" {
            res.push('.');
            res.push_str(f);
        }
    }
    if neg {
        res.insert(0, '-');
    }
    res
}

fn fmt_uptime(seconds: f64) -> String {
    let total = seconds as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    match (h, m) {
        (0, 0) => format!("{s} s"),
        (0, _) => format!("{m} min {s} s"),
        _ => format!("{h} h {m:02} min"),
    }
}

fn platform_emoji(platform: &str) -> &'static str {
    match platform.to_lowercase().as_str() {
        "ps5" | "playstation" | "ps4" => "🎮",
        "xbox" | "xboxone" | "xboxseries" => "🎮",
        "steam" | "pc" | "epic" | "egs" => "💻",
        "switch" => "🕹️",
        _ => "👤",
    }
}

// ── status ────────────────────────────────────────────────────────────────

fn status_card(v: &Value) -> String {
    let status = &v["status"];
    let health = &v["health"];
    let count = v.get("count").and_then(|c| c.as_u64()).unwrap_or(0);

    let server = f_str(status, "serverName");
    let world = f_str(status, "worldName");
    let ready = health
        .get("engineReady")
        .and_then(|b| b.as_bool())
        .unwrap_or(false);
    let version = f_str(health, "version");
    let uptime = f_num(health, "uptimeSeconds");
    let objects = f_num(status, "objectCount");
    let crossplay = status
        .get("crossplayEnabled")
        .and_then(|b| b.as_bool())
        .unwrap_or(false);
    let dedicated = status
        .get("isDedicatedServer")
        .and_then(|b| b.as_bool())
        .unwrap_or(false);
    let hardcore = status
        .get("hardcoreState")
        .and_then(|h| h.as_i64())
        .unwrap_or(-1);
    let guid = f_str(status, "serverGuid");

    let title = format!("🐉 {server}");
    let description = format!(
        "{} · 🌍 {world}",
        if ready { "🟢 en ligne" } else { "🟡 démarrage" }
    );

    let hardcore_label = match hardcore {
        0 => "désactivé".to_string(),
        1 => "activé".to_string(),
        n => format!("niveau {n}"),
    };

    let fields = vec![
        ("👥 Joueurs".into(), format!("{count}"), true),
        ("⏱️ Uptime".into(), fmt_uptime(uptime), true),
        ("🧩 Mod".into(), version, true),
        ("📦 Objets".into(), fmt_num(objects), true),
        ("🔀 Crossplay".into(), yn(crossplay), true),
        ("💀 Hardcore".into(), hardcore_label, true),
        ("🖥️ Dédié".into(), yn(dedicated), true),
        ("🆔 GUID".into(), short_guid(&guid), true),
    ];
    card(&title, &description, "good", &fields)
}

fn yn(b: bool) -> String {
    if b { "✅ oui".into() } else { "❌ non".into() }
}

fn short_guid(guid: &str) -> String {
    let g = guid.trim_matches('\u{feff}');
    if g.len() > 12 {
        format!("{}…{}", &g[..8], &g[g.len() - 4..])
    } else if g.is_empty() {
        "—".into()
    } else {
        g.to_string()
    }
}

// ── players ───────────────────────────────────────────────────────────────

fn players_card(v: &Value) -> String {
    let count = v.get("count").and_then(|c| c.as_u64()).unwrap_or(0);
    let list = v.get("players").and_then(|p| p.as_array()).cloned().unwrap_or_default();
    if list.is_empty() {
        return card(&cmd_title("players"), "Aucun joueur connecté pour le moment.", "info", &[]);
    }
    let extra = if list.len() > MAX_FIELDS {
        format!(" ({} affichés sur {count})", MAX_FIELDS)
    } else {
        String::new()
    };
    let mut fields = Vec::new();
    for player in list.iter().take(MAX_FIELDS) {
        fields.push(player_line(player));
    }
    card(
        &format!("👥 Joueurs connectés ({count}){extra}"),
        "",
        "good",
        &fields,
    )
}

fn player_line(p: &Value) -> (String, String, bool) {
    let name = f_str(p, "characterName");
    let name = if name.is_empty() { f_str(p, "name") } else { name };
    let platform = platform_emoji(&f_str(p, "platform"));
    let ping = f_num(p, "pingMs");
    let health = p.get("health");
    let hp = health
        .filter(|h| !h.is_null())
        .map(|h| {
            format!(
                "❤️ {:.0} / {:.0}",
                f_num(h, "current"),
                f_num(h, "max")
            )
        })
        .unwrap_or_else(|| "❤️ —".into());
    let food = p
        .get("sustenance")
        .filter(|h| !h.is_null())
        .map(|h| format!("🍖 {:.0}", f_num(h, "current")))
        .unwrap_or_else(|| "🍖 —".into());
    let water = p
        .get("hydration")
        .filter(|h| !h.is_null())
        .map(|h| format!("💧 {:.0}", f_num(h, "current")))
        .unwrap_or_else(|| "💧 —".into());
    let pos = p
        .get("location")
        .filter(|l| !l.is_null())
        .map(|l| {
            format!(
                "📍 {:.0}, {:.0}, {:.0}",
                f_num(l, "x"),
                f_num(l, "y"),
                f_num(l, "z")
            )
        })
        .unwrap_or_else(|| "📍 non apparu".into());
    let spawned = p.get("spawned").and_then(|b| b.as_bool()).unwrap_or(false);
    (
        format!("{platform} {name}"),
        format!(
            "📶 {ping:.0} ms · {hp} · {food} · {water}\n{pos}{}",
            if spawned { "" } else { " · 👻" }
        ),
        false,
    )
}

// ── player (fiche) ────────────────────────────────────────────────────────

fn player_card(v: &Value) -> String {
    let list = v.get("players").and_then(|p| p.as_array()).cloned().unwrap_or_default();
    match list.len() {
        0 => card(&cmd_title("player"), "Aucun joueur trouvé avec ce nom.", "info", &[]),
        1 => player_profile(&list[0]),
        _ => {
            // Plusieurs correspondances : liste compacte des candidats.
            let mut fields = Vec::new();
            for player in list.iter().take(MAX_FIELDS) {
                let (name_field, value_field, _) = player_line(player);
                fields.push((name_field, value_field, false));
            }
            card(
                &format!("👤 {} correspondances", list.len()),
                "Précise le nom ou l'identifiant.",
                "info",
                &fields,
            )
        }
    }
}

fn player_profile(p: &Value) -> String {
    let name = f_str(p, "characterName");
    let name = if name.is_empty() { f_str(p, "name") } else { name };
    let platform = f_str(p, "platform");
    let id = p.get("playerId").and_then(|i| i.as_u64()).unwrap_or(0);
    let ping = f_num(p, "pingMs");
    let spawned = p.get("spawned").and_then(|b| b.as_bool()).unwrap_or(false);

    let mut fields = Vec::new();
    fields.push(("📶 Ping".into(), format!("{ping:.0} ms · {}", platform_emoji(&platform)), true));
    fields.push(("🕹️ Apparu".into(), yn(spawned), true));

    if let Some(h) = p.get("health").filter(|h| !h.is_null()) {
        let dead = h.get("isDead").and_then(|b| b.as_bool()).unwrap_or(false);
        let value = format!("{:.0} / {:.0}", f_num(h, "current"), f_num(h, "max"));
        fields.push((
            "❤️ PV".into(),
            if dead { format!("{value} · 💀 mort") } else { value },
            true,
        ));
    }
    if let Some(h) = p.get("sustenance").filter(|h| !h.is_null()) {
        fields.push(("🍖 Faim".into(), format!("{:.0} / {:.0}", f_num(h, "current"), f_num(h, "max")), true));
    }
    if let Some(h) = p.get("hydration").filter(|h| !h.is_null()) {
        fields.push(("💧 Soif".into(), format!("{:.0} / {:.0}", f_num(h, "current"), f_num(h, "max")), true));
    }
    if let Some(l) = p.get("location").filter(|l| !l.is_null()) {
        fields.push((
            "📍 Position (monde)".into(),
            format!("{:.1}, {:.1}, {:.1}", f_num(l, "x"), f_num(l, "y"), f_num(l, "z")),
            false,
        ));
    }
    if let Some(skills) = p.get("skills").and_then(|s| s.as_array()) {
        let lines: Vec<String> = skills
            .iter()
            .filter_map(|s| {
                let skill = f_str(s, "skill").trim_start_matches("SKILL_").to_string();
                let xp = s.get("currentXP").and_then(|x| x.as_f64()).unwrap_or(0.0);
                if xp <= 0.0 && s.get("currentXP").is_none() {
                    return None;
                }
                Some(format!("{skill} · {} XP", fmt_num(xp)))
            })
            .collect();
        if !lines.is_empty() {
            fields.push(("🏅 Compétences".into(), lines.join("\n"), false));
        }
    }
    if let Some(attrs) = p.get("attributes").and_then(|a| a.as_array()) {
        let lines: Vec<String> = attrs
            .iter()
            .take(24)
            .map(|a| {
                let n = f_str(a, "name");
                let v = f_num(a, "value");
                format!("{n} · {}", fmt_num(v))
            })
            .collect();
        if !lines.is_empty() {
            fields.push(("🎭 Attributs".into(), lines.join("\n"), false));
        }
    }

    card(
        &format!("👤 {name}"),
        &format!("ID {id} · plateforme {platform}"),
        "good",
        &fields,
    )
}

// ── items ─────────────────────────────────────────────────────────────────

fn items_card(v: &Value) -> String {
    let list = v.get("items").and_then(|i| i.as_array()).cloned().unwrap_or_default();
    if list.is_empty() {
        return card(&cmd_title("items"), "Aucun objet ne correspond.", "info", &[]);
    }
    let names: Vec<String> = list
        .iter()
        .filter_map(|i| {
            let name = f_str(i, "name");
            let stack = i.get("maxStackSize").and_then(|s| s.as_u64()).unwrap_or(0);
            Some(format!("• {name} · pile {stack}"))
        })
        .collect();
    let shown = names.len().min(40);
    let mut body: Vec<String> = names.into_iter().take(40).collect();
    if shown < list.len() {
        body.push(format!("… et {} autres", list.len() - shown));
    }
    card(
        &format!("🎒 Objets ({})", list.len()),
        &body.join("\n"),
        "good",
        &[],
    )
}

// ── bans ──────────────────────────────────────────────────────────────────

fn bans_card(v: &Value) -> String {
    let list = v.get("bans").and_then(|b| b.as_array()).cloned().unwrap_or_default();
    if list.is_empty() {
        return card(&cmd_title("bans"), "Aucun bannissement enregistré.", "info", &[]);
    }
    let mut fields = Vec::new();
    for ban in list.iter().take(MAX_FIELDS) {
        let name = f_str(ban, "name");
        let net = f_str(ban, "netId");
        let reason = f_str(ban, "reason");
        let at = f_str(ban, "bannedAt");
        fields.push((
            format!("🚫 {name}"),
            format!("🆔 {net}\n📝 {reason}\n🕰️ {at}"),
            false,
        ));
    }
    card(&format!("🚫 Bannissements ({})", list.len()), "", "good", &fields)
}

// ── top ───────────────────────────────────────────────────────────────────

fn top_card(v: &Value) -> String {
    let rows = v.get("top").and_then(|t| t.as_array()).cloned().unwrap_or_default();
    if rows.is_empty() {
        return card(&cmd_title("top"), "Aucune donnée XP pour le moment.", "info", &[]);
    }
    let mut fields = Vec::new();
    for (i, row) in rows.iter().take(MAX_FIELDS).enumerate() {
        let name = f_str(row, "name");
        let xp = f_num(row, "xp");
        let medal = match i {
            0 => "🥇",
            1 => "🥈",
            2 => "🥉",
            _ => "🏅",
        };
        fields.push((format!("{medal} {}", name), format!("{} XP", fmt_num(xp)), false));
    }
    card(&format!("🏆 Classement XP ({})", rows.len()), "", "good", &fields)
}

// ── cheats ────────────────────────────────────────────────────────────────

fn cheats_card(v: &Value) -> String {
    if let Some(enabled) = v.get("cheats_enabled").and_then(|b| b.as_bool()) {
        let description = if enabled {
            "✅ Activées — tp, tpto, give, godmode, heal, feed, hydrate disponibles."
        } else {
            "⛔ Désactivées — les commandes cheat sont refusées par le plugin."
        };
        let tone = if enabled { "good" } else { "wait" };
        if let Some(msg) = v.get("message").and_then(|m| m.as_str()) {
            return card(&cmd_title("cheats"), &format!("{description}\n{msg}"), tone, &[]);
        }
        return card(&cmd_title("cheats"), description, tone, &[]);
    }
    card(&cmd_title("cheats"), "État inconnu.", "wait", &[])
}

// ── ok (POSTs) ────────────────────────────────────────────────────────────

fn ok_card(cmd: &str, v: &Value) -> String {
    let message = f_str(v, "message");
    if message.is_empty() {
        // Réponse inattendue : garder le JSON brut pour le diagnostic.
        let raw = v.to_string();
        return card(&cmd_title(cmd), &format!("```json\n{raw}\n```"), "info", &[]);
    }
    card(&cmd_title(cmd), &format!("✅ {message}"), "good", &[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_becomes_a_card() {
        let raw = r#"{"status":{"serverName":"Dakka Dakka","worldName":"ApaWorld","serverGuid":"3DAFAF1749FCB7C62F08728D18933FAC","isDedicatedServer":true,"crossplayEnabled":true,"hardcoreState":1,"objectCount":296850},"health":{"status":"healthy","version":"0.2.1","uptimeSeconds":4872.5,"engineReady":true},"count":2}"#;
        let out = cardify("status", raw);
        let v: Value = serde_json::from_str(&out).unwrap();
        let card = &v["card"];
        assert_eq!(card["title"], "🐉 Dakka Dakka");
        assert!(card["description"].as_str().unwrap().contains("ApaWorld"));
        let fields = card["fields"].as_array().unwrap();
        assert!(fields.iter().any(|f| f["name"] == "👥 Joueurs" && f["value"] == "2"));
        assert!(fields
            .iter()
            .any(|f| f["name"] == "📦 Objets" && f["value"] == "296 850"));
    }

    #[test]
    fn player_fiche_uses_icons() {
        let raw = r#"{"count":1,"players":[{"name":"ApachaiZ","characterName":"ApachaiZ","playerId":256,"platform":"PS5","pingMs":60,"spawned":true,"health":{"current":180.0,"max":180.0,"canDie":true,"isDead":false},"sustenance":{"current":74.2,"max":100.0},"hydration":{"current":64.4,"max":100.0},"location":{"x":66887.4,"y":185273.2,"z":138.2},"skills":[{"skill":"SKILL_Woodcutting","currentXP":69561}],"attributes":[{"name":"Health","value":180.0,"baseValue":103.5,"shared":true}]}]}"#;
        let out = cardify("player", raw);
        let v: Value = serde_json::from_str(&out).unwrap();
        let card = &v["card"];
        assert_eq!(card["title"], "👤 ApachaiZ");
        let fields = card["fields"].as_array().unwrap();
        assert!(fields.iter().any(|f| f["name"] == "❤️ PV" && f["value"] == "180 / 180"));
        assert!(fields.iter().any(|f| f["name"] == "🍖 Faim"));
        assert!(fields.iter().any(|f| f["name"] == "💧 Soif"));
        assert!(fields.iter().any(|f| f["name"] == "📍 Position (monde)"));
        assert!(fields
            .iter()
            .any(|f| f["name"] == "🏅 Compétences" && f["value"].as_str().unwrap().contains("Woodcutting")));
    }

    #[test]
    fn players_empty_shows_info_card() {
        let out = cardify("players", r#"{"count":0,"players":[]}"#);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["card"]["tone"], "info");
        assert!(v["card"]["description"].as_str().unwrap().contains("Aucun joueur"));
    }

    #[test]
    fn ok_post_becomes_good_card() {
        let out = cardify("heal", r#"{"status":"ok","message":"healed Seiylvana"}"#);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["card"]["title"], "💚 Soin");
        assert_eq!(v["card"]["tone"], "good");
        assert!(v["card"]["description"]
            .as_str()
            .unwrap()
            .contains("healed Seiylvana"));
    }

    #[test]
    fn error_becomes_bad_card() {
        let out = cardify("kick", r#"{"error":"HTTP 500: pas trouvé"}"#);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["card"]["tone"], "bad");
        assert!(v["card"]["title"].as_str().unwrap().starts_with("❌"));
    }

    #[test]
    fn top_has_medals() {
        let raw = r#"{"top":[{"name":"A","xp":1000},{"name":"B","xp":500},{"name":"C","xp":100}]}"#;
        let out = cardify("top", raw);
        let v: Value = serde_json::from_str(&out).unwrap();
        let fields = v["card"]["fields"].as_array().unwrap();
        assert_eq!(fields[0]["name"], "🥇 A");
        assert_eq!(fields[1]["name"], "🥈 B");
        assert_eq!(fields[2]["name"], "🥉 C");
    }

    #[test]
    fn uptime_and_numbers_formatted() {
        assert_eq!(fmt_uptime(4872.0), "1 h 21 min");
        assert_eq!(fmt_uptime(90.0), "1 min 30 s");
        assert_eq!(fmt_num(296850.0), "296 850");
        assert_eq!(fmt_num(1234.5), "1 234.5");
    }

    #[test]
    fn cheats_switch_card() {
        let out = cardify("cheats", r#"{"cheats_enabled":false}"#);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["card"]["tone"], "wait");
        assert!(v["card"]["description"].as_str().unwrap().contains("⛔"));
    }
}
