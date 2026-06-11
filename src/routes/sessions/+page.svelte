<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { ClassSessionRecord, Course } from "$lib/types";

  let courses = $state<Course[]>([]);
  let courseId = $state("");
  let sessions = $state<ClassSessionRecord[]>([]);
  let error = $state("");

  async function load() {
    error = "";
    courses = await api.listCourses();
    sessions = await api.listClassSessions(courseId || undefined);
  }

  async function exportSession(session: ClassSessionRecord) {
    try {
      const csv = await api.exportAttendanceCsv(session.id);
      const blob = new Blob([csv], { type: "text/csv" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `${session.course_title.replace(/\s+/g, "-")}-attendance.csv`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      error = e instanceof Error ? e.message : "Export failed";
    }
  }

  onMount(load);
</script>

<div class="page-header">
  <div>
    <h2>Session history</h2>
    <p>Past class hub sessions and attendance records.</p>
  </div>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

<div class="card" style="margin-bottom: 1rem">
  <label>
    Filter by course
    <select bind:value={courseId} onchange={load}>
      <option value="">All courses</option>
      {#each courses as c}
        <option value={c.id}>{c.code} — {c.title}</option>
      {/each}
    </select>
  </label>
</div>

<div class="card">
  {#each sessions as s}
    <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.85rem 0">
      <div style="display: flex; justify-content: space-between; gap: 1rem; flex-wrap: wrap">
        <div>
          <strong>{s.title}</strong>
          <span class="tag">{s.status}</span>
          <p style="margin: 0.25rem 0; color: var(--muted); font-size: 0.9rem">
            {s.course_title} · PIN {s.pin} · {s.attendance_count} checked in
          </p>
          {#if s.started_at}
            <small style="color: var(--muted)">
              {new Date(s.started_at).toLocaleString()}
              {#if s.ended_at}
                — {new Date(s.ended_at).toLocaleString()}
              {/if}
            </small>
          {/if}
        </div>
        <button class="btn btn-secondary" onclick={() => exportSession(s)}>Export CSV</button>
      </div>
    </div>
  {:else}
    <p class="empty">No sessions recorded yet. Start a Class Hub session to create one.</p>
  {/each}
</div>

<style>
  .tag {
    display: inline-block;
    margin-left: 0.5rem;
    font-size: 0.75rem;
    padding: 0.1rem 0.45rem;
    border-radius: 999px;
    background: #f1f5f9;
  }
</style>
