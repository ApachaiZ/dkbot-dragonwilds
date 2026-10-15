# 🐉 dkbot-dragonwilds

Plugin **dragonwilds** pour [dkbot-base](https://github.com/ApachaiZ/dkbot-base)
(ABI `dkbot-host-api` v1) — administration d'un serveur **RuneScape: Dragonwilds**
via le mod [RSDWServerAPI](https://github.com/ApachaiZ/RSDWServerAPI).

## Installation

```bash
dkbot plugin install dragonwilds   # télécharge le .so (SHA-256 vérifiée)
```

Puis `/plugin dragonwilds <commande>` depuis Discord (owner) — le contexte
appelant (id, owner, rôles) est transmis par l'hôte.

## Configuration (variables d'environnement du processus hôte)

| Variable | Rôle |
|---|---|
| `RSDW_API_BASE_URL` | URL du mod (défaut `http://127.0.0.1:8080`) |
| `RSDW_API_TOKEN` | bearer token identique au `BearerToken` du mod |
| `RSDW_MOD_ROLE_IDS` / `RSDW_ADMIN_ROLE_IDS` | rôles Discord autorisés |
| `RSDW_REQUEST_TIMEOUT_SECS` | timeout HTTP (défaut 10) |
| `DKBOT_PLUGIN_STATE_DIR` | état du plugin (défaut `.config.d/plugins/dragonwilds`) |

## Commandes

`status · players · player · items · announce · kick · ban · unban · bans ·
give · tp · tpto · tell · heal · feed · hydrate · godmode · top · cheats`

- `cheats get|set` : **propriétaire uniquement** — active/désactive les
  fonctionnalités « cheat » (`tp, tpto, give, godmode, heal, feed, hydrate`),
  persisté, défaut activées.
- `top [n]` : classement XP (instantanés fusionnés, jamais décroissants).
- Autorisation : modérateurs pour la lecture/modération, administrateurs
  pour les actions.

## Build

```bash
cargo build --release   # → target/release/libdkbot_dragonwilds.so
```

Licence : MIT.
