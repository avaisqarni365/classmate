<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import WhatsAppShare from "$lib/components/WhatsAppShare.svelte";
  import type { Announcement, Course } from "$lib/types";

  let courses = $state<Course[]>([]);
  let announcements = $state<Announcement[]>([]);
  let courseId = $state("");
  let title = $state("");
  let body = $state("");
  let error = $state("");

  async function load() {
    courses = await api.listCourses();
    announcements = await api.listAnnouncements(courseId || undefined);
  }

  async function post(event: Event) {
    event.preventDefault();
    try {
      await api.createAnnouncement({
        course_id: courseId || undefined,
        title,
        body,
      });
      title = "";
      body = "";
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to post";
    }
  }

  onMount(load);
</script>

<div class="page-header">
  <div>
    <h2>Announcements</h2>
    <p>Broadcast updates to a course or the whole school.</p>
  </div>
</div>

<div class="card" style="margin-bottom:1rem">
  <form class="form-grid" onsubmit={post}>
    <label>
      Course (optional)
      <select bind:value={courseId} onchange={load}>
        <option value="">All courses</option>
        {#each courses as c}
          <option value={c.id}>{c.code} — {c.title}</option>
        {/each}
      </select>
    </label>
    <label>Title<input bind:value={title} required /></label>
    <label>Body<textarea bind:value={body} rows="4" required></textarea></label>
    {#if error}<p class="error">{error}</p>{/if}
    <button class="btn btn-primary" type="submit">Post announcement</button>
  </form>
</div>

<div class="card">
  {#each announcements as a}
    <div class="item" style="border-bottom:1px solid var(--border);padding:.75rem 0">
      <div style="display:flex;justify-content:space-between;align-items:flex-start;gap:.5rem">
        <strong>{a.title}</strong>
        <WhatsAppShare
          kind="announcement"
          announcementId={a.id}
          courseId={a.course_id ?? undefined}
        />
      </div>
      <p style="color:var(--muted);margin:.35rem 0">{a.body}</p>
      <small style="color:var(--muted)">{new Date(a.created_at).toLocaleString()}</small>
    </div>
  {:else}
    <p class="empty">No announcements yet.</p>
  {/each}
</div>
