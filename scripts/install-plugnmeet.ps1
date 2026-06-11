# plugNmeet install helper for ClassMate (optional)
# Requires Docker Desktop: https://www.docker.com/products/docker-desktop/

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Dir = Join-Path $env:APPDATA "com.classmate.app\plugnmeet"

Write-Host "ClassMate plugNmeet helper" -ForegroundColor Cyan
Write-Host ""
Write-Host "plugNmeet is a full-featured MIT-licensed virtual classroom (whiteboard, breakout rooms)."
Write-Host "It runs as a separate Docker stack — not bundled inside ClassMate."
Write-Host ""

if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "Docker is not installed. Install Docker Desktop first." -ForegroundColor Yellow
    Write-Host "https://www.docker.com/products/docker-desktop/"
    exit 1
}

New-Item -ItemType Directory -Force -Path $Dir | Out-Null

$compose = @"
# Minimal plugNmeet reference stack — adjust for production use.
# Docs: https://www.plugnmeet.org/docs
services:
  plugnmeet:
    image: mynaparrot/plugnmeet-server:latest
    ports:
      - "8080:8080"
    restart: unless-stopped
"@

$composePath = Join-Path $Dir "docker-compose.yml"
Set-Content -Path $composePath -Value $compose -Encoding UTF8

Write-Host "Wrote compose file:" $composePath
Write-Host ""
Write-Host "Next steps:"
Write-Host "  1. cd `"$Dir`""
Write-Host "  2. docker compose up -d"
Write-Host "  3. In ClassMate Settings, set plugNmeet server URL to http://YOUR-LAN-IP:8080"
Write-Host "  4. Set room ID (e.g. classmate) — used when Galene is not running"
Write-Host ""
Write-Host "For production, follow official plugNmeet docs for LiveKit, Redis, and TLS."
