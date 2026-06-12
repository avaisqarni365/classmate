<script lang="ts">
  import { onDestroy } from "svelte";
  import { api } from "$lib/api";
  import type { CaptureSession } from "$lib/types";

  let {
    courseId,
    onAttached,
    onCancel,
  }: {
    courseId: string;
    onAttached?: () => void;
    onCancel?: () => void;
  } = $props();

  let title = $state("");
  let session = $state<CaptureSession | null>(null);
  let error = $state("");
  let loading = $state(false);
  let attachTitle = $state("");
  let pollTimer: ReturnType<typeof setInterval> | null = null;

  async function startSession() {
    if (!title.trim()) {
      error = "Enter a title for this handwriting session.";
      return;
    }
    loading = true;
    error = "";
    try {
      session = await api.createCaptureSession({ course_id: courseId, title: title.trim() });
      attachTitle = session.title;
      startPolling();
    } catch (e) {
      error = e instanceof Error ? e.message : "Could not start capture session";
    } finally {
      loading = false;
    }
  }

  async function refreshSession() {
    if (!session) return;
    try {
      session = await api.getCaptureSession(session.id);
    } catch {
      /* ignore transient poll errors */
    }
  }

  function startPolling() {
    stopPolling();
    pollTimer = setInterval(refreshSession, 3000);
  }

  function stopPolling() {
    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  async function attachNotes() {
    if (!session) return;
    loading = true;
    error = "";
    try {
      await api.attachCaptureSession(session.id, attachTitle.trim() || undefined);
      stopPolling();
      session = null;
      title = "";
      onAttached?.();
    } catch (e) {
      error = e instanceof Error ? e.message : "Could not attach handwriting";
    } finally {
      loading = false;
    }
  }

  const padUrl = $derived(session?.pad_url ?? "");
  const qrUrl = $derived(
    padUrl ? `https://api.qrserver.com/v1/create-qr-code/?size=180x180&data=${encodeURIComponent(padUrl)}` : "",
  );
  const hasInk = $derived(Boolean(session?.preview_data_url));

  onDestroy(stopPolling);
</script>

<div class="handwriting-capture">
  <p class="hint">
    Open the pad link on your Android or iOS tablet, then write with Apple Pencil or stylus.
    Ink syncs back here automatically — attach when ready.
  </p>
  {#if error}
    <p class="error">{error}</p>
  {/if}

  {#if !session}
    <label>
      Session title
      <input bind:value={title} placeholder="Lecture 4 — diagrams" />
    </label>
    <div class="actions">
      <button class="btn btn-primary btn-sm" type="button" disabled={loading} onclick={startSession}>
        {loading ? "Starting…" : "Open tablet pad"}
      </button>
      {#if onCancel}
        <button class="btn btn-secondary btn-sm" type="button" onclick={onCancel}>Cancel</button>
      {/if}
    </div>
  {:else}
    <div class="pad-panel">
      <strong>{session.title}</strong>
      <p class="muted" style="font-size:.85rem;margin:.35rem 0">
        Status: {session.status} · expires {session.expires_at.slice(0, 16).replace("T", " ")}
      </p>
      {#if padUrl}
        <div class="pad-link">
          {#if qrUrl}
            <img src={qrUrl} alt="QR code for tablet pad" width="180" height="180" />
          {/if}
          <div>
            <a href={padUrl} target="_blank" rel="noopener">{padUrl}</a>
            <p class="muted" style="font-size:.82rem;margin-top:.35rem">
              Scan the QR code or open this link on your phone/tablet.
            </p>
          </div>
        </div>
      {/if}
      {#if session.preview_data_url}
        <img class="preview" src={session.preview_data_url} alt="Handwriting preview" />
      {:else}
        <p class="muted">Waiting for ink from your tablet…</p>
      {/if}
      <label>
        Material title when attached
        <input bind:value={attachTitle} />
      </label>
      <div class="actions">
        <button class="btn btn-secondary btn-sm" type="button" onclick={refreshSession}>Refresh</button>
        <button
          class="btn btn-primary btn-sm"
          type="button"
          disabled={loading || !hasInk}
          onclick={attachNotes}
        >
          {loading ? "Attaching…" : "Attach as lecture note"}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .handwriting-capture {
    margin-bottom: 0.75rem;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: color-mix(in srgb, #0ea5e9 6%, var(--card, #fff));
  }

  .hint {
    margin: 0 0 0.65rem;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .pad-link {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    align-items: flex-start;
    margin: 0.5rem 0;
  }

  .preview {
    max-width: 100%;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: #fff;
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
    margin-top: 0.5rem;
  }
</style>
