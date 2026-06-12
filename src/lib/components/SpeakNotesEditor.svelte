<script lang="ts">
  import { api } from "$lib/api";
  import { buildSpeakNoteContent } from "$lib/materials";
  import { speakText, startSpeechToText, sttSupported, stopSpeaking, ttsSupported } from "$lib/speech";

  let {
    courseId,
    onSaved,
    onCancel,
  }: {
    courseId: string;
    onSaved?: () => void;
    onCancel?: () => void;
  } = $props();

  let title = $state("");
  let body = $state("");
  let outline = $state("");
  let transcript = $state("");
  let listening = $state(false);
  let error = $state("");
  let saving = $state(false);
  let listener: { stop: () => void } | null = null;

  function toggleListen() {
    error = "";
    if (listening) {
      listener?.stop();
      listener = null;
      listening = false;
      return;
    }
    if (!sttSupported()) {
      error = "Speech-to-text needs Chrome or Edge (Web Speech API).";
      return;
    }
    try {
      listener = startSpeechToText(
        (chunk) => {
          transcript = (transcript + " " + chunk).trim();
          body = (body + " " + chunk).trim();
        },
        (chunk) => {
          transcript = (transcript.split("\n").slice(0, -1).join("\n") + "\n" + chunk).trim();
        },
      );
      listening = true;
    } catch (e) {
      error = e instanceof Error ? e.message : "Could not start microphone";
    }
  }

  async function save(event: Event) {
    event.preventDefault();
    if (!title.trim() || !body.trim()) {
      error = "Title and note body are required.";
      return;
    }
    saving = true;
    error = "";
    try {
      await api.createMaterial({
        course_id: courseId,
        title: title.trim(),
        kind: "speak_note",
        content: buildSpeakNoteContent({ body, outline, transcript }),
      });
      title = "";
      body = "";
      outline = "";
      transcript = "";
      onSaved?.();
    } catch (e) {
      error = e instanceof Error ? e.message : "Could not save speak note";
    } finally {
      saving = false;
    }
  }

  $effect(() => {
    return () => {
      listener?.stop();
      stopSpeaking();
    };
  });
</script>

<form class="speak-notes" onsubmit={save}>
  <p class="hint">
    Dictate lecture notes with your microphone (open-source Web Speech API in Chrome/Edge) or type manually.
    Use read-aloud to review notes with text-to-speech.
  </p>
  {#if error}
    <p class="error">{error}</p>
  {/if}
  <label>
    Title
    <input bind:value={title} required placeholder="Week 3 — Photosynthesis" />
  </label>
  <label>
    Outline (optional)
    <textarea bind:value={outline} rows="2" placeholder="Key topics for this lecture"></textarea>
  </label>
  <label>
    Lecture notes
    <textarea bind:value={body} rows="6" required placeholder="Speak or type your notes here…"></textarea>
  </label>
  {#if transcript}
    <label>
      Live transcript
      <textarea bind:value={transcript} rows={3} readonly class="transcript"></textarea>
    </label>
  {/if}
  <div class="toolbar">
    <button type="button" class="btn btn-secondary btn-sm" onclick={toggleListen}>
      {listening ? "Stop listening" : "Start speech-to-text"}
    </button>
    <button
      type="button"
      class="btn btn-secondary btn-sm"
      disabled={!ttsSupported() || !body.trim()}
      onclick={() => speakText(body)}
    >
      Read aloud
    </button>
    <button type="button" class="btn btn-secondary btn-sm" onclick={() => stopSpeaking()}>
      Stop audio
    </button>
  </div>
  <div class="actions">
    <button class="btn btn-primary" type="submit" disabled={saving}>
      {saving ? "Saving…" : "Save speak notes"}
    </button>
    {#if onCancel}
      <button type="button" class="btn btn-secondary" onclick={onCancel}>Cancel</button>
    {/if}
  </div>
</form>

<style>
  .speak-notes {
    display: grid;
    gap: 0.65rem;
    margin-bottom: 0.75rem;
    padding: 0.75rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: color-mix(in srgb, var(--primary) 4%, var(--card, #fff));
  }

  .hint {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted);
  }

  .transcript {
    font-size: 0.85rem;
    color: var(--muted);
  }

  .toolbar,
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 0.35rem;
  }
</style>
