<div align="center">

# 🐉 dkbot-dragonwilds

### Dragonwilds plugin for [dkbot-base](https://github.com/ApachaiZ/dkbot-base) — ABI `dkbot-host-api` v1

[![License](https://img.shields.io/badge/licence-MIT-blue)](#-license)
[![ABI](https://img.shields.io/badge/ABI-dkbot--host--api%20v1-purple)](https://github.com/ApachaiZ/dkbot-base)
[![Game](https://img.shields.io/badge/game-RuneScape%3A%20Dragonwilds-orange)](https://github.com/ApachaiZ/RSDWServerAPI)

Administration of a **RuneScape: Dragonwilds** dedicated server from Discord,
through the [RSDWServerAPI](https://github.com/ApachaiZ/RSDWServerAPI) mod:
players, stats, teleports, heal, bans — as **typed slash commands with
autocomplete**, answered by **formatted embed cards**.

> 🌐 Languages: 🇫🇷 [Français](README.fr.md) · 🇩🇪 [Deutsch](README.de.md) · 🇪🇸 [Español](README.es.md)

</div>

---

## 📖 Contents

1. [📦 Install](#-install)
2. [⚙️ Configuration](#️-configuration)
3. [🎮 Commands](#-commands)
4. [🛡️ Roles](#️-roles)
5. [🎛️ The cheat switch](#️-the-cheat-switch)
6. [✨ Autocomplete & cards](#-autocomplete--cards)
7. [🔨 Build & tests](#-build--tests)
8. [📄 License](#-license)

---

## 📦 Install

```bash
dkbot plugin install dragonwilds   # downloads the .so, SHA-256 verified
```

Installing is all it takes: the plugin's manifest is turned into real Discord
slash commands by the host at startup — `/dw …` and `/dw-admin …` appear
without touching the bot core.

```text
   ▄▄▄▄▄▄▄▄▄▄▄▄▄▄
 ▄▀█ dkbot-base █▀▄
▄▀  █  ABI v1    █  ▀▄        ┌───────────────┐
█   ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀         │  dkbot-       │
█  /dw status · /dw tp · …    │  dragonwilds  │──HTTP──► RSDWServerAPI
█  /dw-admin cheats           │  (this .so)   │  Bearer   (in the game
▀▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  └───────────────┘  token    server process)
```

---

## ⚙️ Configuration

Environment variables of the host process (the bot's `.env`):

| Variable | Role |
|---|---|
| `RSDW_API_BASE_URL` | Mod URL — default `http://127.0.0.1:8080` |
| `RSDW_API_TOKEN` | Bearer token — same value as `BearerToken` in the mod's `settings.ini` |
| `RSDW_MOD_ROLE_IDS` | Discord roles allowed read/moderation (comma-separated) |
| `RSDW_ADMIN_ROLE_IDS` | Discord roles allowed admin actions |
| `RSDW_REQUEST_TIMEOUT_SECS` | HTTP timeout — default 10 |
| `DKBOT_PLUGIN_STATE_DIR` | Plugin state (cheat switch, leaderboard) — default `.config.d/plugins/dragonwilds` |

---

## 🎮 Commands

**`/dw` — everyone with moderator+ role:**

| Command | Role |
|---|---|
| `status` | 🐉 Server identity, world, players online, uptime, mod version, objects |
| `players` | 👥 Every connected player (ping, HP, food, water, position) |
| `player <player>` | 👤 Full profile: stats, skills, attributes |
| `items [search]` | 🎒 Item catalogue with filter |
| `bans` | 🚫 Persistent bans |
| `top [n]` | 🏆 XP leaderboard (medals, persisted snapshots) |
| `announce <message>` | 📢 Global in-game broadcast |
| `kick` `ban` `unban` | 👢 🔨 🕊️ Moderation |
| `tell <player> <message>` | 💬 Private in-game message |

**`/dw` — cheat set (administrator+, owner-switchable):**

| Command | Role |
|---|---|
| `tp <from> <to>` | 🔄 Teleport a player onto another |
| `tpto <player> <x> <y> <z>` | 📍 Teleport to coordinates |
| `give <player> <item> [count]` | 🎁 Give items |
| `godmode <player> [enabled]` | 🛡️ Toggle god mode |
| `heal` `feed` `hydrate <player>` | 💚 🍖 💧 Care actions |

**`/dw-admin` — administrators only:**

| Command | Role |
|---|---|
| `cheats [state:on|off]` | 🎛️ Owner switch — enables/disables the whole cheat set (see below) |

---

## 🛡️ Roles

- **Moderator** (`RSDW_MOD_ROLE_IDS`) — read + moderation (status, players,
  player, items, bans, top, announce, kick, ban, unban, tell).
- **Administrator** (`RSDW_ADMIN_ROLE_IDS`) — everything above + cheat
  actions (tp, tpto, give, godmode, heal, feed, hydrate).
- **Owner** (`DISCORD_OWNER_ID` of the host) — the `cheats` switch.

---

## 🎛️ The cheat switch

`/dw-admin cheats state:off` **removes the 7 cheat commands from Discord** —
not just blocks them: the plugin filters its manifest and the host
re-registers the guild commands automatically, no restart. `state:on` brings
them back. The state is persisted on disk and survives bot restarts.

> When OFF, even a direct call is refused by the plugin (« cheat features
> disabled by the owner ») — defense in depth.

---

## ✨ Autocomplete & cards

- 🔍 **Autocomplete everywhere it makes sense** — player arguments suggest
  **nicknames only** (never internal ids), item arguments suggest item names;
  both live from the API, filtered as you type, max 25.
- 🃏 **Cards, not dumps** — every response is a standard Discord embed card
  (icons, fields, colors); errors render as red cards.
- 🏆 **Leaderboard** — XP snapshots merged monotonically (never decreases),
  pruned after 60 days.

---

## 🔨 Build & tests

```bash
cargo build --release   # → target/release/libdkbot_dragonwilds.so
cargo test              # 26 unit tests (cards, autocomplete, switch, manifest)
```

---

## 📄 License

MIT.
