# ClassMate

Local-first online and desktop classroom platform built with **Tauri 2**, **SvelteKit**, and **SQLite**.

MIT licensed — safe for commercial use.

## Features (v1.0)

- **Fully responsive UI** — mobile drawer nav (desktop app), stacked layouts, mobile gradebook cards (web portal), responsive ARTIZAI embeds
- Everything in v0.33 — teacher web lecture notes, speak notes, tablet handwriting, ARTIZAI labs, OpenStax integration

## Features (v0.33)

- Everything in v0.32, plus:
- **Teacher web portal — lecture notes** — speak notes (STT/TTS) and tablet handwriting from the browser
- **Bug fixes** — TypeScript/speech API types, hub quiz route, WhatsApp settings types, Svelte 5 layout

## Features (v0.32)

- Everything in v0.31, plus:
- **Speak notes** — speech-to-text dictation and text-to-speech read-aloud (Web Speech API in Chrome/Edge)
- **Tablet handwriting pad** — QR/link opens `/notes/pad` on Android/iOS; Apple Pencil/stylus ink syncs back as lecture notes
- **Embedded ARTIZAI labs** — optional iframe embed on student views
- **Lab completion tracking** — students mark AI lab complete; status shown in portal and My Courses

## Features (v0.31)

- Everything in v0.30, plus:
- **ARTIZAI AI Lab on every lecture note** — auto-matched lab (Science, Maths, Arts, etc.) with link to [artizai.uk](https://artizai.uk)
- **OpenStax + AI together** — textbook read/PDF links plus AI lab activities on desktop, Class Hub, and web portal
- **Configurable ARTIZAI base URL** in Settings

## Features (v0.30)

- Everything in v0.29, plus:
- **OpenStax textbook library** — browse 60+ free peer-reviewed school/college books and attach them to courses as lecture notes
- **Student read links** — materials show OpenStax online reader and PDF links in My Courses and Class Hub

## Features (v0.29)

- Everything in v0.28, plus:
- **Admin web console** — dashboard stats, create users/courses, manage enrollments, public URL settings
- **Admin gradebook & announcements** — same teacher tools, scoped to all school courses

## Features (v0.28)

- Everything in v0.27, plus:
- **Teacher web portal** — sign in as teacher/admin, view gradebook, save scores, post announcements
- **Course-scoped access** — teachers see their courses; admins see all courses in the school

## Features (v0.27)

- Everything in v0.26, plus:
- **Web landing** — Codes AI ecosystem intro at [cm.codes-ai.uk](https://cm.codes-ai.uk)
- **Student / parent web portal** — sign in on the web, view courses and grades
- **Broadcast delivery summary** — aggregated sent/delivered/read counts from WhatsApp status webhooks

### Platform (v0.26 and earlier)

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

**Help:** in-app **Help** page, web [cm.codes-ai.uk/help](https://cm.codes-ai.uk/help), or [HELP.md](./HELP.md).

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
