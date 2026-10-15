<div align="center">

# 🐉 dkbot-dragonwilds

### Dragonwilds-Plugin für [dkbot-base](https://github.com/ApachaiZ/dkbot-base) — ABI `dkbot-host-api` v1

[![Lizenz](https://img.shields.io/badge/lizenz-MIT-blue)](#-lizenz)
[![ABI](https://img.shields.io/badge/ABI-dkbot--host--api%20v1-purple)](https://github.com/ApachaiZ/dkbot-base)
[![Spiel](https://img.shields.io/badge/spiel-RuneScape%3A%20Dragonwilds-orange)](https://github.com/ApachaiZ/RSDWServerAPI)

Verwaltung eines **RuneScape: Dragonwilds**-Dedicated-Servers aus Discord über
den Mod [RSDWServerAPI](https://github.com/ApachaiZ/RSDWServerAPI): Spieler,
Stats, Teleports, Heilung, Bans — als **typisierte Slash Commands mit
Autovervollständigung**, beantwortet mit **formatierten Embed-Karten**.

> 🌐 Sprachen: 🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇩🇪 Deutsch · 🇪🇸 [Español](README.es.md)

</div>

---

## 📖 Inhalt

1. [📦 Installation](#-installation)
2. [⚙️ Konfiguration](#️-konfiguration)
3. [🎮 Befehle](#-befehle)
4. [🛡️ Rollen](#️-rollen)
5. [🎛️ Der Cheat-Schalter](#️-der-cheatschalter)
6. [✨ Autovervollständigung & Karten](#-autovervollständigung--karten)
7. [🔨 Build & Tests](#-build--tests)
8. [📄 Lizenz](#-lizenz)

---

## 📦 Installation

```bash
dkbot plugin install dragonwilds   # lädt das .so, SHA-256-geprüft
```

Installieren genügt: Das Plugin-Manifest wird beim Host-Start zu echten
Discord-Slash-Commands — `/dw …` und `/dw-admin …` erscheinen, ohne den
Bot-Kern anzufassen.

```text
   ▄▄▄▄▄▄▄▄▄▄▄▄▄▄
 ▄▀█ dkbot-base █▀▄
▄▀  █  ABI v1    █  ▀▄        ┌───────────────┐
█   ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀         │  dkbot-       │
█  /dw status · /dw tp · …    │  dragonwilds  │──HTTP──► RSDWServerAPI
█  /dw-admin cheats           │  (dieses .so) │  Bearer   (im Spiel-Server-
▀▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  └───────────────┘  token    Prozess)
```

---

## ⚙️ Konfiguration

Umgebungsvariablen des Host-Prozesses (die `.env` des Bots):

| Variable | Rolle |
|---|---|
| `RSDW_API_BASE_URL` | Mod-URL — Standard `http://127.0.0.1:8080` |
| `RSDW_API_TOKEN` | Bearer-Token — gleicher Wert wie `BearerToken` in der `settings.ini` des Mods |
| `RSDW_MOD_ROLE_IDS` | Discord-Rollen für Lesen/Moderation (kommagetrennt) |
| `RSDW_ADMIN_ROLE_IDS` | Discord-Rollen für Admin-Aktionen |
| `RSDW_REQUEST_TIMEOUT_SECS` | HTTP-Timeout — Standard 10 |
| `DKBOT_PLUGIN_STATE_DIR` | Plugin-Zustand (Cheat-Schalter, Bestenliste) — Standard `.config.d/plugins/dragonwilds` |

---

## 🎮 Befehle

**`/dw` — Moderator+: **

| Befehl | Rolle |
|---|---|
| `status` | 🐉 Server-Identität, Welt, Spieler online, Uptime, Mod-Version, Objekte |
| `players` | 👥 Alle verbundenen Spieler (Ping, HP, Hunger, Durst, Position) |
| `player <spieler>` | 👤 Vollprofil: Werte, Skills, Attribute |
| `items [suche]` | 🎒 Objektkatalog mit Filter |
| `bans` | 🚫 Permanente Bans |
| `top [n]` | 🏆 XP-Bestenliste (Medaillen, persistierte Snapshots) |
| `announce <nachricht>` | 📢 Globale In-Game-Durchsage |
| `kick` `ban` `unban` | 👢 🔨 🕊️ Moderation |
| `tell <spieler> <nachricht>` | 💬 Private In-Game-Nachricht |

**`/dw` — Cheat-Set (Administrator+, owner-schaltbar):**

| Befehl | Rolle |
|---|---|
| `tp <from> <to>` | 🔄 Spieler auf einen anderen teleportieren |
| `tpto <spieler> <x> <y> <z>` | 📍 Zu Koordinaten teleportieren |
| `give <spieler> <item> [anzahl]` | 🎁 Items geben |
| `godmode <spieler> [an]` | 🛡️ God-Mode umschalten |
| `heal` `feed` `hydrate <spieler>` | 💚 🍖 💧 Fürsorge-Aktionen |

**`/dw-admin` — nur Administratoren:**

| Befehl | Rolle |
|---|---|
| `cheats [state:on|off]` | 🎛️ Owner-Schalter — aktiviert/deaktiviert das ganze Cheat-Set (siehe unten) |

---

## 🛡️ Rollen

- **Moderator** (`RSDW_MOD_ROLE_IDS`) — Lesen + Moderation (status, players,
  player, items, bans, top, announce, kick, ban, unban, tell).
- **Administrator** (`RSDW_ADMIN_ROLE_IDS`) — alles obige + Cheat-Aktionen
  (tp, tpto, give, godmode, heal, feed, hydrate).
- **Owner** (`DISCORD_OWNER_ID` des Hosts) — der `cheats`-Schalter.

---

## 🎛️ Der Cheat-Schalter

`/dw-admin cheats state:off` **entfernt die 7 Cheat-Befehle aus Discord** —
nicht nur sperren: Das Plugin filtert sein Manifest und der Host registriert
die Guild-Befehle automatisch neu, ohne Neustart. `state:on` holt sie zurück.
Der Zustand ist auf Disk persistiert und übersteht Bot-Neustarts.

> Bei OFF wird selbst ein direkter Aufruf vom Plugin abgelehnt
> („Cheat-Funktionen vom Eigentümer deaktiviert“) — Defense in Depth.

---

## ✨ Autovervollständigung & Karten

- 🔍 **Autovervollständigung überall dort, wo sinnvoll** — Spieler-Argumente
  schlagen **nur Nicknames** vor (niemals interne IDs), Item-Argumente
  Item-Namen; beides live aus der API, beim Tippen gefiltert, max. 25.
- 🃏 **Karten statt Dumps** — jede Antwort ist eine Standard-Discord-Karte
  (Icons, Felder, Farben); Fehler erscheinen als rote Karten.
- 🏆 **Bestenliste** — XP-Snapshots monoton zusammengeführt (nie
  abnehmend), nach 60 Tagen bereinigt.

---

## 🔨 Build & Tests

```bash
cargo build --release   # → target/release/libdkbot_dragonwilds.so
cargo test              # 26 Unit-Tests (Karten, Autocomplete, Schalter, Manifest)
```

---

## 📄 Lizenz

MIT.
