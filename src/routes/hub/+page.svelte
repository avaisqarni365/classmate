<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { api } from "$lib/api";
  import WhatsAppShare from "$lib/components/WhatsAppShare.svelte";
  import type { Course, HubStatus, PollResults } from "$lib/types";

  let courses = $state<Course[]>([]);
  let hub = $state<HubStatus | null>(null);
  let selectedCourseId = $state("");
  let sessionTitle = $state("");
  let enableVideo = $state(true);
  let attendance = $state(0);
  let error = $state("");
  let pollResults = $state<PollResults | null>(null);
  let pollQuestion = $state("");
  let pollOptionA = $state("");
  let pollOptionB = $state("");
  let pollOptionC = $state("");
  let refreshTimer: ReturnType<typeof setInterval> | undefined;
  let pollTimer: ReturnType<typeof setInterval> | undefined;

  async function refreshHub() {
    hub = await api.getHubStatus();
    if (hub?.session_id) {
      attendance = await api.getAttendanceCount(hub.session_id);
      await refreshPoll();
    } else {
      attendance = 0;
      pollResults = null;
    }
  }

  async function refreshPoll() {
    if (!hub?.session_id) return;
    pollResults = await api.getActiveSessionPoll(hub.session_id);
  }

  async function load() {
    error = "";
    try {
      courses = await api.listCourses();
      if (!selectedCourseId && courses.length > 0) {
        selectedCourseId = courses[0].id;
      }
      await refreshHub();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load class hub";
    }
  }

  async function startHub() {
    if (!selectedCourseId) return;
    error = "";
    try {
      hub = await api.startClassHub(
        selectedCourseId,
        sessionTitle || undefined,
        enableVideo,
      );
      attendance = hub.session_id
        ? await api.getAttendanceCount(hub.session_id)
        : 0;
      pollResults = null;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to start class hub";
    }
  }

  async function stopHub() {
    error = "";
    try {
      hub = await api.stopClassHub();
      attendance = 0;
      pollResults = null;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to stop class hub";
    }
  }

  async function exportAttendance() {
    if (!hub?.session_id || !hub.course_title) return;
    try {
      const csv = await api.exportAttendanceCsv(hub.session_id);
      const blob = new Blob([csv], { type: "text/csv" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${hub.course_title.replace(/\s+/g, "-")}-attendance.csv`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to export attendance";
    }
  }

  async function launchPoll(event: Event) {
    event.preventDefault();
    if (!hub?.session_id) return;
    const options = [pollOptionA, pollOptionB, pollOptionC].filter((o) => o.trim());
    try {
      pollResults = await api.createSessionPoll({
        session_id: hub.session_id,
        question: pollQuestion,
        options,
      });
      pollQuestion = "";
      pollOptionA = "";
      pollOptionB = "";
      pollOptionC = "";
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to launch poll";
    }
  }

  async function closePoll() {
    if (!pollResults) return;
    pollResults = await api.closeSessionPoll(pollResults.poll.id);
  }

  onMount(() => {
    load();
    refreshTimer = setInterval(refreshHub, 3000);
    pollTimer = setInterval(refreshPoll, 2000);
  });

  onDestroy(() => {
    if (refreshTimer) clearInterval(refreshTimer);
    if (pollTimer) clearInterval(pollTimer);
  });
</script>

<div class="page-header">
  <div>
    <h2>Class Hub</h2>
    <p>Broadcast a local classroom on your Wi‑Fi. Students join from any browser.</p>
  </div>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

<div class="grid two-col hub-panel">
  <div class="card">
    <h3 style="margin-top:0">Start session</h3>
    <form
      class="form-grid"
      onsubmit={(e) => {
        e.preventDefault();
        startHub();
      }}
    >
      <label>
        Course
        <select bind:value={selectedCourseId} required>
          {#each courses as course}
            <option value={course.id}>{course.code} — {course.title}</option>
          {/each}
        </select>
      </label>
      <label>
        Session title (optional)
        <input bind:value={sessionTitle} placeholder="Period 3 — Lab day" />
      </label>
      <label style="display:flex;align-items:center;gap:.5rem;font-weight:600">
        <input type="checkbox" bind:checked={enableVideo} />
        Enable live video (Galene)
      </label>
      <div style="display:flex; gap:0.5rem">
        <button class="btn btn-primary" type="submit" disabled={hub?.running}>
          Start class hub
        </button>
        {#if hub?.running}
          <button class="btn btn-danger" type="button" onclick={stopHub}>
            Stop
          </button>
        {/if}
      </div>
    </form>
  </div>

  <div class="card" class:hub-live={hub?.running}>
    <h3 style="margin-top:0">Live status</h3>
    {#if !hub}
      <p class="empty">Loading...</p>
    {:else if !hub.running}
      <p class="empty">No active class hub. Start a session to generate a PIN and join URL.</p>
    {:else}
      <p><strong>{hub.course_title}</strong></p>
      <p>PIN: <span class="mono">{hub.pin}</span></p>
      <p>Students joined: <strong>{attendance}</strong></p>
      {#if hub.join_url}
        <p>Join URL:</p>
        <p class="mono" style="word-break: break-all">{hub.join_url}</p>
        <WhatsAppShare kind="hub" label="Share join link" />
      {/if}
      {#if hub.video_url}
        <p>Video URL:</p>
        <p class="mono" style="word-break: break-all">{hub.video_url}</p>
      {/if}
      {#if hub.local_ip}
        <p style="color: var(--muted); font-size: 0.9rem">
          Make sure student devices are on the same Wi‑Fi network as this computer ({hub.local_ip}).
        </p>
      {/if}
      <button class="btn btn-secondary" type="button" onclick={exportAttendance} style="margin-top: 0.75rem">
        Export attendance CSV
      </button>
    {/if}
  </div>
</div>

{#if hub?.running}
  <div class="grid two-col" style="margin-top: 1rem">
    <div class="card">
      <h3 style="margin-top: 0">Live poll</h3>
      <form class="form-grid" onsubmit={launchPoll}>
        <label>Question<input bind:value={pollQuestion} required placeholder="Do you understand today's topic?" /></label>
        <label>Option A<input bind:value={pollOptionA} required /></label>
        <label>Option B<input bind:value={pollOptionB} required /></label>
        <label>Option C<input bind:value={pollOptionC} /></label>
        <button class="btn btn-primary" type="submit">Launch poll</button>
      </form>
    </div>

    <div class="card">
      <h3 style="margin-top: 0">Poll results</h3>
      {#if pollResults}
        <p><strong>{pollResults.poll.question}</strong></p>
        <p style="color: var(--muted); font-size: 0.9rem">
          {pollResults.total_votes} votes · {pollResults.poll.status}
        </p>
        <ul style="margin: 0.75rem 0; padding-left: 1.2rem">
          {#each pollResults.poll.options as opt, i}
            <li>{opt}: <strong>{pollResults.vote_counts[i] ?? 0}</strong></li>
          {/each}
        </ul>
        {#if pollResults.poll.status === "open"}
          <button class="btn btn-secondary" onclick={closePoll}>Close poll</button>
        {/if}
      {:else}
        <p class="empty">No active poll. Launch one to gather live responses from students.</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .hub-live {
    border-color: #86efac;
    background: #f0fdf4;
  }
  .mono { font-family: ui-monospace, monospace; }
</style>
