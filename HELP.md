# ClassMate Help

Complete setup guide for local desktop use, domain deployment, sync, Class Hub, and WhatsApp.

## In the app

Open **Help** in the sidebar (or go to `/help` from the login screen).

The guide shows your configured public URLs, sync status, and download links when available.

## On the web (public)

When your server is running and DNS points to it:

| Page | URL |
|------|-----|
| **Download** | `https://cm.codes-ai.uk/download` |
| **Setup guide** | `https://cm.codes-ai.uk/help` |
| **Windows installer** | `https://cm.codes-ai.uk/download/win` (after uploading the `.exe` to the server) |

### Upload installer to the server

From your Windows PC (after `npm run tauri:build`):

```powershell
scp D:\ClassMate\src-tauri\target\release\bundle\nsis\ClassMate_0.26.0_x64-setup.exe root@212.227.54.250:/var/lib/classmate/downloads/
```

Then `https://cm.codes-ai.uk/download/win` serves the file.

## Quick local setup

1. **Install** — run the Windows setup `.exe` or build with `npm run tauri:build`
2. **Sign in** — `admin@classmate.local` / `admin123` (change immediately)
3. **Courses & users** — create your school data
4. **Backup** — Settings → export JSON regularly
5. **SQLite** — all data is local; no MySQL/Postgres needed

## Domain setup (cm.codes-ai.uk)

1. GoDaddy DNS: **A record** `cm` → `212.227.54.250`
2. Ionos server runs `classmate-server` (port 8766)
3. nginx proxies `/download`, `/help`, `/api/sync/`, `/api/whatsapp/webhook`, `/hub/`
4. Desktop app: **Settings → Public base URL** = `https://cm.codes-ai.uk`
5. HTTPS: `certbot --nginx -d cm.codes-ai.uk` (after DNS propagates)

See [DEPLOY.md](./DEPLOY.md) for full nginx/Caddy examples.

## Architecture

| Component | Where it runs |
|-----------|----------------|
| Admin UI (courses, grades, users) | Windows desktop app |
| SQLite database | Desktop PC and/or server (`/var/lib/classmate/`) |
| Students / webhooks | Public domain → VPS |
| Class Hub live sessions | Desktop starts hub (port 8765); nginx exposes `/hub/` |

## More

- Source: [github.com/avaisqarni365/classmate](https://github.com/avaisqarni365/classmate)
- Roadmap: [ROADMAP.md](./ROADMAP.md)
