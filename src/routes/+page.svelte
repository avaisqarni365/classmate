<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { api } from "$lib/api";
  import { auth } from "$lib/stores/auth";
  import type { AnalyticsReport, DashboardStats } from "$lib/types";

  let stats = $state<DashboardStats | null>(null);
  let analytics = $state<AnalyticsReport | null>(null);
  let error = $state("");

  onMount(async () => {
    if (auth.user?.role === "student") {
      goto("/my-courses");
      return;
    }
    try {      [stats, analytics] = await Promise.all([
        api.getDashboardStats(),
        api.getAnalytics(),
      ]);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load dashboard";
    }
  });
</script>

<div class="page-header">
  <div>
    <h2>Dashboard</h2>
    <p>Overview of your local ClassMate workspace.</p>
  </div>
</div>

{#if error}
  <p class="error">{error}</p>
{:else if stats && analytics}
  <div class="grid stats-grid">
    <div class="card">
      <div class="stat-label">Users</div>
      <div class="stat-value">{stats.user_count}</div>
    </div>
    <div class="card">
      <div class="stat-label">Courses</div>
      <div class="stat-value">{stats.course_count}</div>
    </div>
    <div class="card">
      <div class="stat-label">Students</div>
      <div class="stat-value">{stats.student_count}</div>
    </div>
    <div class="card">
      <div class="stat-label">Assignments</div>
      <div class="stat-value">{stats.assignment_count}</div>
    </div>
    <div class="card">
      <div class="stat-label">Live sessions</div>
      <div class="stat-value">{stats.active_sessions}</div>
    </div>
    <div class="card">
      <div class="stat-label">Quiz attempts</div>
      <div class="stat-value">{analytics.total_quiz_attempts}</div>
    </div>
  </div>

  <div class="grid two-col" style="margin-top: 1rem">
    <div class="card">
      <h3 style="margin-top: 0">Course performance</h3>
      {#each analytics.course_summaries as c}
        <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.6rem 0">
          <strong>{c.course_title}</strong>
          <p style="margin: 0.25rem 0; color: var(--muted); font-size: 0.9rem">
            {c.student_count} students
            {#if c.average_percent != null}
              · Avg grade {c.average_percent.toFixed(0)}%
            {/if}
            {#if c.attendance_rate != null}
              · Attendance {c.attendance_rate.toFixed(0)}%
            {/if}
          </p>
        </div>
      {:else}
        <p class="empty">No course data yet.</p>
      {/each}
    </div>

    <div class="card">
      <h3 style="margin-top: 0">At-risk students</h3>
      <p style="color: var(--muted); font-size: 0.9rem; margin-top: 0">
        Students averaging below 60% on graded assignments.
      </p>
      {#each analytics.at_risk_students as s}
        <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.6rem 0">
          <strong>{s.student_name}</strong>
          <p style="margin: 0.25rem 0; color: var(--muted); font-size: 0.9rem">
            {s.course_title} · {s.average_percent.toFixed(0)}%
          </p>
        </div>
      {:else}
        <p class="empty">No at-risk students flagged.</p>
      {/each}
    </div>
  </div>

  <div class="card" style="margin-top: 1rem">
    <h3 style="margin-top: 0">Recent class sessions</h3>
    {#each analytics.recent_sessions as s}
      <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.6rem 0">
        <strong>{s.title}</strong>
        <p style="margin: 0.25rem 0; color: var(--muted); font-size: 0.9rem">
          {s.course_title} · {s.attendance_count} checked in
          {#if s.started_at}
            · {new Date(s.started_at).toLocaleString()}
          {/if}
        </p>
      </div>
    {:else}
      <p class="empty">No sessions recorded yet.</p>
    {/each}
  </div>
{:else}
  <p class="empty">Loading dashboard...</p>
{/if}
