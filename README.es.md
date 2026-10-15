<div align="center">

# 🐉 dkbot-dragonwilds

### Plugin Dragonwilds para [dkbot-base](https://github.com/ApachaiZ/dkbot-base) — ABI `dkbot-host-api` v1

[![Licencia](https://img.shields.io/badge/licencia-MIT-blue)](#-licencia)
[![ABI](https://img.shields.io/badge/ABI-dkbot--host--api%20v1-purple)](https://github.com/ApachaiZ/dkbot-base)
[![Juego](https://img.shields.io/badge/juego-RuneScape%3A%20Dragonwilds-orange)](https://github.com/ApachaiZ/RSDWServerAPI)

Administración de un servidor dedicado **RuneScape: Dragonwilds** desde
Discord, a través del mod [RSDWServerAPI](https://github.com/ApachaiZ/RSDWServerAPI):
jugadores, stats, teletransportes, curas, baneos — como **slash commands
tipadas con autocompletado**, respondidas con **tarjetas Discord formateadas**.

> 🌐 Idiomas: 🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇩🇪 [Deutsch](README.de.md) · 🇪🇸 Español

</div>

---

## 📖 Índice

1. [📦 Instalación](#-instalación)
2. [⚙️ Configuración](#️-configuración)
3. [🎮 Comandos](#-comandos)
4. [🛡️ Roles](#️-roles)
5. [🎛️ El interruptor de cheats](#️-el-interruptor-de-cheats)
6. [✨ Autocompletado y tarjetas](#-autocompletado-y-tarjetas)
7. [🔨 Build y tests](#-build-y-tests)
8. [📄 Licencia](#-licencia)

---

## 📦 Instalación

```bash
dkbot plugin install dragonwilds   # descarga el .so, SHA-256 verificado
```

Instalar es todo lo que hace falta: el manifiesto del plugin se convierte en
slash commands reales de Discord al arrancar el host — `/dw …` y `/dw-admin …`
aparecen sin tocar el núcleo del bot.

```text
   ▄▄▄▄▄▄▄▄▄▄▄▄▄▄
 ▄▀█ dkbot-base █▀▄
▄▀  █  ABI v1    █  ▀▄        ┌───────────────┐
█   ▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀         │  dkbot-       │
█  /dw status · /dw tp · …    │  dragonwilds  │──HTTP──► RSDWServerAPI
█  /dw-admin cheats           │  (este .so)   │  Bearer   (en el proceso del
▀▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  └───────────────┘  token    servidor de juego)
```

---

## ⚙️ Configuración

Variables de entorno del proceso host (el `.env` del bot):

| Variable | Rol |
|---|---|
| `RSDW_API_BASE_URL` | URL del mod — por defecto `http://127.0.0.1:8080` |
| `RSDW_API_TOKEN` | Bearer token — mismo valor que `BearerToken` en el `settings.ini` del mod |
| `RSDW_MOD_ROLE_IDS` | Roles de Discord autorizados lectura/moderación (comas) |
| `RSDW_ADMIN_ROLE_IDS` | Roles de Discord autorizados acciones admin |
| `RSDW_REQUEST_TIMEOUT_SECS` | Timeout HTTP — por defecto 10 |
| `DKBOT_PLUGIN_STATE_DIR` | Estado del plugin (interruptor de cheats, ranking) — por defecto `.config.d/plugins/dragonwilds` |

---

## 🎮 Comandos

**`/dw` — nivel moderador+:**

| Comando | Rol |
|---|---|
| `status` | 🐉 Identidad del servidor, mundo, jugadores online, uptime, versión del mod, objetos |
| `players` | 👥 Todos los jugadores conectados (ping, vida, hambre, sed, posición) |
| `player <jugador>` | 👤 Ficha completa: stats, habilidades, atributos |
| `items [búsqueda]` | 🎒 Catálogo de objetos con filtro |
| `bans` | 🚫 Baneos persistentes |
| `top [n]` | 🏆 Ranking de XP (medallas, instantáneas persistidas) |
| `announce <mensaje>` | 📢 Difusión global en el juego |
| `kick` `ban` `unban` | 👢 🔨 🕊️ Moderación |
| `tell <jugador> <mensaje>` | 💬 Mensaje privado en el juego |

**`/dw` — conjunto cheat (administrador+, conmutable por el propietario):**

| Comando | Rol |
|---|---|
| `tp <from> <to>` | 🔄 Teletransportar un jugador sobre otro |
| `tpto <jugador> <x> <y> <z>` | 📍 Teletransportar a coordenadas |
| `give <jugador> <objeto> [cantidad]` | 🎁 Dar objetos |
| `godmode <jugador> [activado]` | 🛡️ Conmutar el modo dios |
| `heal` `feed` `hydrate <jugador>` | 💚 🍖 💧 Acciones de cuidado |

**`/dw-admin` — solo administradores:**

| Comando | Rol |
|---|---|
| `cheats [state:on|off]` | 🎛️ Interruptor del propietario — activa/desactiva todo el conjunto cheat (ver abajo) |

---

## 🛡️ Roles

- **Moderador** (`RSDW_MOD_ROLE_IDS`) — lectura + moderación (status, players,
  player, items, bans, top, announce, kick, ban, unban, tell).
- **Administrador** (`RSDW_ADMIN_ROLE_IDS`) — todo lo anterior + acciones
  cheat (tp, tpto, give, godmode, heal, feed, hydrate).
- **Propietario** (`DISCORD_OWNER_ID` del host) — el interruptor `cheats`.

---

## 🎛️ El interruptor de cheats

`/dw-admin cheats state:off` **elimina los 7 comandos cheat de Discord** — no
solo los bloquea: el plugin filtra su manifiesto y el host re-registra los
comandos del guild automáticamente, sin reiniciar. `state:on` los trae de
vuelta. El estado se persiste en disco y sobrevive a los reinicios del bot.

> En OFF, incluso una llamada directa es rechazada por el plugin
> («funcionalidades cheat desactivadas por el propietario») — defensa en
> profundidad.

---

## ✨ Autocompletado y tarjetas

- 🔍 **Autocompletado donde tiene sentido** — los argumentos de jugador
  sugieren **solo apodos** (nunca IDs internos), los de objeto sugieren
  nombres de objetos; ambos en vivo desde la API, filtrados al teclear, máx. 25.
- 🃏 **Tarjetas, no volcados** — cada respuesta es una tarjeta estándar de
  Discord (iconos, campos, colores); los errores son tarjetas rojas.
- 🏆 **Ranking** — instantáneas de XP fusionadas de forma monótona (nunca
  decrecientes), podadas tras 60 días.

---

## 🔨 Build y tests

```bash
cargo build --release   # → target/release/libdkbot_dragonwilds.so
cargo test              # 26 tests unitarios (tarjetas, autocompletado, interruptor, manifiesto)
```

---

## 📄 Licencia

MIT.
