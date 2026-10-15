<div align="center">

# 🐉 dkbot-dragonwilds

### Plugin Dragonwilds pour [dkbot-base](https://github.com/ApachaiZ/dkbot-base) — ABI `dkbot-host-api` v1

[![Licence](https://img.shields.io/badge/licence-MIT-blue)](#-licence)
[![ABI](https://img.shields.io/badge/ABI-dkbot--host--api%20v1-purple)](https://github.com/ApachaiZ/dkbot-base)
[![Jeu](https://img.shields.io/badge/jeu-RuneScape%3A%20Dragonwilds-orange)](https://github.com/ApachaiZ/RSDWServerAPI)

Administration d'un serveur dédié **RuneScape: Dragonwilds** depuis Discord,
via le mod [RSDWServerAPI](https://github.com/ApachaiZ/RSDWServerAPI) :
joueurs, stats, téléportations, soins, bans — en **slash commands typées avec
autocomplétion**, répondues par des **cartes Discord formatées**.

> 🌐 Langues : 🇬🇧 [English](README.md) · 🇫🇷 Français · 🇩🇪 [Deutsch](README.de.md) · 🇪🇸 [Español](README.es.md)

</div>

---

## 📖 Sommaire

1. [📦 Installation](#-installation)
2. [⚙️ Configuration](#️-configuration)
3. [🎮 Commandes](#-commandes)
4. [🛡️ Rôles](#️-rôles)
5. [🎛️ Le switch cheat](#️-le-switch-cheat)
6. [✨ Autocomplétion & cartes](#-autocomplétion--cartes)
7. [🔨 Build & tests](#-build--tests)
8. [📄 Licence](#-licence)

---

## 📦 Installation

```bash
dkbot plugin install dragonwilds   # télécharge le .so, SHA-256 vérifié
```

Installer suffit : le manifeste du plugin devient de vraies slash commands
Discord au démarrage de l'hôte — `/dw …` et `/dw-admin …` apparaissent sans
toucher au cœur du bot.

```text
   ▄▄▄▄▄▄▄▄▄▄▄▄▄▄
 ▄▀█ dkbot-base █▀▄
▄▀  █  ABI v1    █  ▀▄        ┌───────────────┐
█   ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀         │  dkbot-       │
█  /dw status · /dw tp · …    │  dragonwilds  │──HTTP──► RSDWServerAPI
█  /dw-admin cheats           │  (ce .so)     │  Bearer   (dans le processus
▀▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  └───────────────┘  token    du serveur de jeu)
```

---

## ⚙️ Configuration

Variables d'environnement du processus hôte (le `.env` du bot) :

| Variable | Rôle |
|---|---|
| `RSDW_API_BASE_URL` | URL du mod — défaut `http://127.0.0.1:8080` |
| `RSDW_API_TOKEN` | Bearer token — même valeur que `BearerToken` dans le `settings.ini` du mod |
| `RSDW_MOD_ROLE_IDS` | Rôles Discord autorisés lecture/modération (virgules) |
| `RSDW_ADMIN_ROLE_IDS` | Rôles Discord autorisés actions admin |
| `RSDW_REQUEST_TIMEOUT_SECS` | Timeout HTTP — défaut 10 |
| `DKBOT_PLUGIN_STATE_DIR` | État du plugin (switch cheat, classement) — défaut `.config.d/plugins/dragonwilds` |

---

## 🎮 Commandes

**`/dw` — niveau modérateur+ :**

| Commande | Rôle |
|---|---|
| `status` | 🐉 Identité du serveur, monde, joueurs en ligne, uptime, version du mod, objets |
| `players` | 👥 Tous les joueurs connectés (ping, PV, faim, soif, position) |
| `player <joueur>` | 👤 Fiche complète : stats, compétences, attributs |
| `items [recherche]` | 🎒 Catalogue d'objets avec filtre |
| `bans` | 🚫 Bannissements persistants |
| `top [n]` | 🏆 Classement XP (médailles, instantanés persistés) |
| `announce <message>` | 📢 Diffusion globale en jeu |
| `kick` `ban` `unban` | 👢 🔨 🕊️ Modération |
| `tell <joueur> <message>` | 💬 Message privé en jeu |

**`/dw` — ensemble cheat (administrateur+, commutable par le propriétaire) :**

| Commande | Rôle |
|---|---|
| `tp <from> <to>` | 🔄 Téléporter un joueur sur un autre |
| `tpto <joueur> <x> <y> <z>` | 📍 Téléporter vers des coordonnées |
| `give <joueur> <objet> [nombre]` | 🎁 Donner des objets |
| `godmode <joueur> [activé]` | 🛡️ Basculer le god mode |
| `heal` `feed` `hydrate <joueur>` | 💚 🍖 💧 Actions de soin |

**`/dw-admin` — administrateurs uniquement :**

| Commande | Rôle |
|---|---|
| `cheats [state:on|off]` | 🎛️ Switch propriétaire — active/désactive tout l'ensemble cheat (voir ci-dessous) |

---

## 🛡️ Rôles

- **Modérateur** (`RSDW_MOD_ROLE_IDS`) — lecture + modération (status, players,
  player, items, bans, top, announce, kick, ban, unban, tell).
- **Administrateur** (`RSDW_ADMIN_ROLE_IDS`) — tout ce qui précède + les
  actions cheat (tp, tpto, give, godmode, heal, feed, hydrate).
- **Propriétaire** (`DISCORD_OWNER_ID` de l'hôte) — le switch `cheats`.

---

## 🎛️ Le switch cheat

`/dw-admin cheats state:off` **retire les 7 commandes cheat de Discord** —
pas seulement les bloque : le plugin filtre son manifeste et l'hôte
ré-enregistre les commandes de la guilde automatiquement, sans redémarrage.
`state:on` les fait revenir. L'état est persisté sur disque et survit aux
redémarrages du bot.

> Quand il est OFF, même un appel direct est refusé par le plugin
> (« fonctionnalités cheat désactivées par le propriétaire ») — défense en
> profondeur.

---

## ✨ Autocomplétion & cartes

- 🔍 **Autocomplétion partout où c'est pertinent** — les arguments joueur
  suggèrent **uniquement les pseudos** (jamais d'identifiants internes), les
  arguments objet suggèrent les noms d'objets ; les deux en direct depuis
  l'API, filtrés à la frappe, max 25.
- 🃏 **Des cartes, pas des dumps** — chaque réponse est une carte Discord
  standard (icônes, champs, couleurs) ; les erreurs sont des cartes rouges.
- 🏆 **Classement** — instantanés XP fusionnés de façon monotone (jamais
  décroissants), purgés après 60 jours.

---

## 🔨 Build & tests

```bash
cargo build --release   # → target/release/libdkbot_dragonwilds.so
cargo test              # 26 tests unitaires (cartes, autocomplétion, switch, manifeste)
```

---

## 📄 Licence

MIT.
