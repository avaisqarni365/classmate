# Deploying ClassMate on a domain

ClassMate is a **desktop app** (Tauri) with embedded HTTP servers for student access and integrations. To expose Class Hub, LAN sync, and WhatsApp webhooks on a public domain with HTTPS, run ClassMate on a always-on PC or VM and put a reverse proxy in front of it.

## Architecture

| Service | Local port | Purpose |
|---------|------------|---------|
| Class Hub (student PWA) | **8765** | Live class join, materials, quizzes, polls |
| LAN sync + WhatsApp webhook | **8766** | Peer backup sync, Meta webhook callback |

Typical public URLs (with default hub path `/hub`):

| Public URL | Proxied to |
|------------|------------|
| `https://classmate.yourschool.com/hub/` | `http://127.0.0.1:8765/` |
| `https://classmate.yourschool.com/api/sync/` | `http://127.0.0.1:8766/api/sync/` |
| `https://classmate.yourschool.com/api/whatsapp/webhook` | `http://127.0.0.1:8766/api/whatsapp/webhook` |

## 1. Build the Windows app

Requirements: [Node.js 20+](https://nodejs.org), [Rust](https://rustup.rs), WebView2 (included on Windows 11).

```powershell
cd D:\ClassMate
npm install
npm run tauri:build
```

Output files:

| File | Path |
|------|------|
| **Installer (.exe setup)** | `src-tauri\target\release\bundle\nsis\ClassMate_0.26.0_x64-setup.exe` |
| **Portable app** | `src-tauri\target\release\classmate.exe` |

Install on the machine that will run 24/7 (or during school hours). Log in as admin, start **Settings → LAN peer sync → Start sync server**, and start Class Hub from **Class Hub** when needed.

## 2. Configure public URLs in ClassMate

1. Open **Settings → LAN peer sync**
2. Set **Public base URL** to `https://classmate.yourschool.com` (no trailing slash)
3. Set **Hub URL path** to `/hub` (must match your reverse proxy)
4. Click **Save public URLs**
5. Copy the displayed **WhatsApp webhook** URL into your Meta app dashboard

## 3. Reverse proxy (Caddy — recommended)

Install [Caddy](https://caddyserver.com/) on the same Windows machine or a gateway server that can reach it.

`C:\caddy\Caddyfile`:

```caddy
classmate.yourschool.com {
    handle /hub/* {
        uri strip_prefix /hub
        reverse_proxy 127.0.0.1:8765
    }

    handle /api/sync/* {
        reverse_proxy 127.0.0.1:8766
    }

    handle /api/whatsapp/webhook {
        reverse_proxy 127.0.0.1:8766
    }
}
```

Caddy obtains and renews Let's Encrypt certificates automatically when port 443 is reachable from the internet.

Run: `caddy run --config C:\caddy\Caddyfile`

## 4. Reverse proxy (nginx)

```nginx
server {
    listen 443 ssl http2;
    server_name classmate.yourschool.com;

    ssl_certificate     /etc/letsencrypt/live/classmate.yourschool.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/classmate.yourschool.com/privkey.pem;

    location /hub/ {
        proxy_pass http://127.0.0.1:8765/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /api/sync/ {
        proxy_pass http://127.0.0.1:8766/api/sync/;
        proxy_set_header Host $host;
    }

    location /api/whatsapp/webhook {
        proxy_pass http://127.0.0.1:8766/api/whatsapp/webhook;
        proxy_set_header Host $host;
    }
}
```

## 5. DNS and firewall

1. Create an **A record**: `classmate.yourschool.com` → your public IP
2. Forward **443** (and optionally 80 for ACME) to the proxy host
3. Allow **8765** and **8766** only on localhost unless the proxy runs on another machine (then restrict by IP)

## 6. WhatsApp webhooks

In Meta Developer Console → WhatsApp → Configuration:

- **Callback URL**: value shown in ClassMate Settings (e.g. `https://classmate.yourschool.com/api/whatsapp/webhook`)
- **Verify token**: same as in ClassMate Settings → WhatsApp Business API
- Subscribe to: `messages`, `group_lifecycle_update`, `group_participants_update`, `group_settings_update`, `group_status_update`

The sync server must be **running** for Meta to verify and deliver webhooks.

## 7. Security notes

- Change default demo passwords before going live
- Use a strong **sync token**; it protects backup import/export
- Class Hub uses a session PIN — share join links only with enrolled students
- The admin desktop UI is **not** exposed via the proxy; only Hub and API routes above are intended to be public

## Troubleshooting

| Issue | Check |
|-------|--------|
| Webhook verify fails | Sync server running? Token matches? Port 8766 reachable from proxy? |
| Students can't join Hub | Hub session started? `/hub/student` path matches proxy? |
| HTTPS certificate errors | DNS propagated? Port 443 open? |

See [README.md](./README.md) for feature overview and demo accounts.
