# ClassMate

Local-first online and desktop classroom platform built with **Tauri 2**, **SvelteKit**, and **SQLite**.

MIT licensed — safe for commercial use.

## Features (v0.26)

- Everything in v0.25, plus:
- **Public domain URLs** — configure HTTPS base URL for Class Hub join links and WhatsApp webhooks
- **Release build** — Windows NSIS installer via `npm run tauri:build`
- **Deployment guide** — [DEPLOY.md](./DEPLOY.md) for reverse proxy + domain setup

### Platform

- Desktop admin with **login**, dashboard, users, courses, gradebook, class hub
- Embedded **SQLite** database (offline, no cloud required)
- **Gradebook** — matrix scoring + CSV export with rubric breakdown
- **Assignment rubrics** — criteria-based grading; visible to students and parents
- **Enrollments**, **materials**, **announcements**, **forums**
- **Quizzes** — MCQ + short answer, auto-grade, **manual review** for open answers
- **Analytics** dashboard + at-risk student flags
- **Timetable** (`/schedule`) — weekly course slots
- **Live polls** during Class Hub sessions
- **Assignment submissions** — text + **file attachments** (2 MB)
- **Student portal** (`/my-courses`) — grades, rubrics, materials, submit work
- **Parent portal** — linked students, grade table, HTML export + **email digest**
- **WhatsApp** — sharing, Business API, templates, inbound routing, scheduling, GDPR compliance
- **Multi-school tenancy** — campuses, school switcher, org admin
- **Class Hub** — LAN server on port `8765` with student PWA
- **LAN peer sync** — backup v2 on port `8766`
- **Scheduled auto-backup**, **cloud backup push**, **live video** fallback chain
- **Certificates**, **OneRoster CSV**, **LTI tools**, **i18n** (EN / ES)

See [ROADMAP.md](./ROADMAP.md) for queued work.

## Build Windows installer

```powershell
npm install
npm run tauri:build
```

Outputs:

- **Setup:** `src-tauri\target\release\bundle\nsis\ClassMate_0.26.0_x64-setup.exe`
- **Portable:** `src-tauri\target\release\classmate.exe`

Requires [Rust](https://rustup.rs) and Node.js 20+.

## Run on a domain

ClassMate exposes Class Hub (port 8765) and sync/webhooks (port 8766). Put a reverse proxy (Caddy or nginx) in front and set **Settings → Public base URL**.

Full steps: **[DEPLOY.md](./DEPLOY.md)**

## Quick start

```bash
npm install
npm run tauri:dev
```

### Mobile push (optional)

1. **Settings → Mobile push** — paste FCM service account JSON and/or APNs .p8 credentials
2. Send a **test push** with a real device token
3. Enable **assignment reminders** for daily due-date notifications
4. Mobile apps or Class Hub PWA register tokens via `POST /api/student/push/register`

### Cash book (admin)

1. **Cash book** (admin nav) — record student fee income and teacher salary expenses
2. Filter by date, view balance summary, export CSV
3. Optional **Invoice Ninja** (self-hosted, AGPL) — paste URL + API token in Cash book settings for formal invoicing alongside the local ledger

Other open-source options you can pair via CSV export: **Crater**, **Akaunting**, **Firefly III**.

### Demo accounts

| Role    | Email                   | Password    |
|---------|-------------------------|-------------|
| Admin   | admin@classmate.local   | admin123    |
| Teacher | teacher@classmate.local | teacher123  |
| Student | student@classmate.local | student123  |
| Parent  | parent@classmate.local  | parent123   |

Fresh DB includes a demo rubric on Lab Report 1 and student WhatsApp opt-in.

## License

MIT — see [LICENSE](./LICENSE).
