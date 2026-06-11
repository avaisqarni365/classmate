<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { auth } from "$lib/stores/auth";
  import { t } from "$lib/i18n";
  import type { HelpInfo, User } from "$lib/types";

  let info = $state<HelpInfo | null>(null);
  let user = $state<User | null>(null);

  onMount(() => {
    const unsub = auth.subscribe((u) => {
      user = u;
    });
    api.getHelpInfo().then((h) => {
      info = h;
    }).catch(() => {
      info = null;
    });
    return unsub;
  });
</script>

{#if !user}
  <p style="margin-bottom:1rem;font-size:.9rem">
    <a href="/login">← Sign in</a>
  </p>
{/if}

<div class="help-page">
  <header class="help-header">
    <h1>{t("help.title")}</h1>
    <p>{t("help.subtitle")}</p>
    {#if info}
      <p class="help-meta">ClassMate v{info.app_version}</p>
    {/if}
  </header>

  <nav class="help-toc card">
    <a href="#download">{t("help.toc.download")}</a>
    <a href="#local">{t("help.toc.local")}</a>
    <a href="#domain">{t("help.toc.domain")}</a>
    <a href="#sync">{t("help.toc.sync")}</a>
    <a href="#hub">{t("help.toc.hub")}</a>
    <a href="#whatsapp">{t("help.toc.whatsapp")}</a>
    <a href="#security">{t("help.toc.security")}</a>
  </nav>

  <section id="download" class="card">
    <h2>{t("help.download.title")}</h2>
    <p>{t("help.download.body")}</p>
    <div class="help-actions">
      {#if info?.download_web_url}
        <a class="btn btn-primary" href={info.download_web_url} target="_blank" rel="noopener noreferrer">
          {t("help.download.web")}
        </a>
        {#if info.windows_installer_available}
          <a class="btn btn-secondary" href="{info.download_web_url}/win" target="_blank" rel="noopener noreferrer">
            {t("help.download.direct")}
          </a>
        {/if}
      {/if}
      <a class="btn btn-secondary" href="https://github.com/avaisqarni365/classmate" target="_blank" rel="noopener noreferrer">
        GitHub
      </a>
    </div>
    <p class="help-note">{t("help.download.build")}</p>
    <pre class="mono">npm install{'\n'}npm run tauri:build</pre>
    <p class="help-note">{t("help.download.output")}</p>
  </section>

  <section id="local" class="card">
    <h2>{t("help.local.title")}</h2>
    <ol>
      <li>{t("help.local.step1")}</li>
      <li>{t("help.local.step2")}</li>
      <li>{t("help.local.step3")}</li>
      <li>{t("help.local.step4")}</li>
      <li>{t("help.local.step5")}</li>
    </ol>
    <table class="help-table">
      <thead>
        <tr><th>{t("help.demo.role")}</th><th>{t("help.demo.email")}</th><th>{t("help.demo.password")}</th></tr>
      </thead>
      <tbody>
        <tr><td>Admin</td><td>admin@classmate.local</td><td>admin123</td></tr>
        <tr><td>Teacher</td><td>teacher@classmate.local</td><td>teacher123</td></tr>
        <tr><td>Student</td><td>student@classmate.local</td><td>student123</td></tr>
        <tr><td>Parent</td><td>parent@classmate.local</td><td>parent123</td></tr>
      </tbody>
    </table>
  </section>

  <section id="domain" class="card">
    <h2>{t("help.domain.title")}</h2>
    <p>{t("help.domain.body")}</p>
    <ol>
      <li>{t("help.domain.step1")}</li>
      <li>{t("help.domain.step2")}</li>
      <li>{t("help.domain.step3")}</li>
    </ol>
    {#if info?.public_base_url}
      <p><strong>{t("help.domain.configured")}</strong> <code>{info.public_base_url}</code></p>
    {:else}
      <p class="help-note">{t("help.domain.notConfigured")}</p>
    {/if}
    {#if info?.webhook_url}
      <p>WhatsApp webhook: <code class="break">{info.webhook_url}</code></p>
    {/if}
    {#if info?.hub_join_url}
      <p>Class Hub: <code class="break">{info.hub_join_url}</code></p>
    {/if}
  </section>

  <section id="sync" class="card">
    <h2>{t("help.sync.title")}</h2>
    <p>{t("help.sync.body")}</p>
    <ul>
      <li>{t("help.sync.item1")}</li>
      <li>{t("help.sync.item2")}</li>
      <li>{t("help.sync.item3")}</li>
    </ul>
    {#if info}
      <p>{t("help.sync.status")}: {info.sync_running ? t("help.sync.running") : t("help.sync.stopped")}</p>
    {/if}
  </section>

  <section id="hub" class="card">
    <h2>{t("help.hub.title")}</h2>
    <p>{t("help.hub.body")}</p>
    <ol>
      <li>{t("help.hub.step1")}</li>
      <li>{t("help.hub.step2")}</li>
      <li>{t("help.hub.step3")}</li>
    </ol>
  </section>

  <section id="whatsapp" class="card">
    <h2>{t("help.whatsapp.title")}</h2>
    <p>{t("help.whatsapp.body")}</p>
    <ol>
      <li>{t("help.whatsapp.step1")}</li>
      <li>{t("help.whatsapp.step2")}</li>
      <li>{t("help.whatsapp.step3")}</li>
      <li>{t("help.whatsapp.step4")}</li>
    </ol>
  </section>

  <section id="security" class="card">
    <h2>{t("help.security.title")}</h2>
    <ul>
      <li>{t("help.security.item1")}</li>
      <li>{t("help.security.item2")}</li>
      <li>{t("help.security.item3")}</li>
    </ul>
  </section>
</div>

<style>
  .help-page { max-width: 820px; }
  .help-header h1 { margin: 0 0 0.35rem; }
  .help-header p { color: var(--muted); margin: 0; }
  .help-meta { font-size: 0.85rem; margin-top: 0.5rem !important; }
  .help-toc { display: flex; flex-wrap: wrap; gap: 0.75rem 1rem; margin: 1.25rem 0; }
  .help-toc a { font-size: 0.9rem; }
  section.card h2 { margin-top: 0; font-size: 1.1rem; }
  .help-actions { display: flex; flex-wrap: wrap; gap: 0.5rem; margin: 1rem 0; }
  .help-note { color: var(--muted); font-size: 0.9rem; }
  pre.mono {
    background: #0f172a;
    color: #e2e8f0;
    padding: 0.75rem 1rem;
    border-radius: 8px;
    font-size: 0.85rem;
    overflow-x: auto;
  }
  .help-table { width: 100%; border-collapse: collapse; font-size: 0.9rem; margin-top: 1rem; }
  .help-table th, .help-table td { border: 1px solid var(--border); padding: 0.45rem 0.6rem; text-align: left; }
  code.break { word-break: break-all; }
  ol, ul { padding-left: 1.25rem; color: var(--muted); }
  li { margin: 0.35rem 0; }
</style>
