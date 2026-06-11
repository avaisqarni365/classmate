#!/usr/bin/env bash
set -euo pipefail

# ClassMate headless deploy for Ionos/Ubuntu VPS
# Usage (as root): bash ionos-deploy.sh

DOMAIN="${CLASSMATE_DOMAIN:-cm.codes-ai.uk}"
REPO="${CLASSMATE_REPO:-https://github.com/avaisqarni365/classmate.git}"
INSTALL_DIR="/opt/classmate"
DATA_DIR="/var/lib/classmate"

echo "==> Installing packages"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq build-essential pkg-config libssl-dev curl git ca-certificates sqlite3 ufw \
  libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev

if ! command -v cargo >/dev/null 2>&1; then
  echo "==> Installing Rust"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

echo "==> Cloning/updating ClassMate"
if [ -d "$INSTALL_DIR/.git" ]; then
  git -C "$INSTALL_DIR" pull --ff-only
else
  git clone "$REPO" "$INSTALL_DIR"
fi

echo "==> Building classmate-server (this may take several minutes)"
cd "$INSTALL_DIR/src-tauri"
cargo build --release --bin classmate-server

install -m 755 target/release/classmate-server /usr/local/bin/classmate-server
mkdir -p "$DATA_DIR/downloads"

echo "==> Installing systemd service"
cat >/etc/systemd/system/classmate-server.service <<EOF
[Unit]
Description=ClassMate sync and WhatsApp webhook server
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
Environment=CLASSMATE_DATA_DIR=$DATA_DIR
Environment=CLASSMATE_DOWNLOAD_DIR=$DATA_DIR/downloads
ExecStart=/usr/local/bin/classmate-server
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable classmate-server
systemctl restart classmate-server

echo "==> nginx site (when nginx is installed)"
if command -v nginx >/dev/null 2>&1; then
  cp "$INSTALL_DIR/scripts/nginx-cm.codes-ai.uk.conf" "/etc/nginx/sites-available/$DOMAIN"
  ln -sf "/etc/nginx/sites-available/$DOMAIN" "/etc/nginx/sites-enabled/$DOMAIN"
  nginx -t && systemctl reload nginx
fi

echo "==> Firewall"
ufw allow OpenSSH
ufw allow 80/tcp
ufw allow 443/tcp
ufw --force enable || true

# Apply settings after DB exists (server creates DB on first start)
sleep 2
if [ -f "$DATA_DIR/classmate.db" ]; then
  sqlite3 "$DATA_DIR/classmate.db" <<SQL
INSERT OR REPLACE INTO settings (key, value) VALUES ('public_base_url', 'https://$DOMAIN');
INSERT OR REPLACE INTO settings (key, value) VALUES ('public_hub_path', '/hub');
SQL
  systemctl restart classmate-server
fi

echo ""
echo "Deploy complete."
echo "  Domain:     https://$DOMAIN"
echo "  Download:   https://$DOMAIN/download"
echo "  Help:       https://$DOMAIN/help"
echo "  Webhook:    https://$DOMAIN/api/whatsapp/webhook"
echo "  Hub path:   https://$DOMAIN/hub/student (when Class Hub is running)"
echo "  Data:       $DATA_DIR/classmate.db"
echo "  Service:    systemctl status classmate-server"
echo ""
echo "Point GoDaddy DNS A record: cm -> $(curl -s ifconfig.me || hostname -I | awk '{print $1}')"
echo "Change default admin password before going live (admin@classmate.local / admin123)."
