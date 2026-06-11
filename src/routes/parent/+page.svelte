<script lang="ts">
  import { onMount } from "svelte";
  import { auth } from "$lib/stores/auth";
  import { api } from "$lib/api";
  import type { ParentGradeEntry, ParentStudentSummary, User } from "$lib/types";

  let dashboard = $state<ParentStudentSummary[]>([]);
  let grades = $state<ParentGradeEntry[]>([]);
  let students = $state<User[]>([]);
  let parentId = $state("");
  let studentId = $state("");
  let error = $state("");
  let message = $state("");
  let smtpConfigured = $state(false);
  let emailOverride = $state("");

  async function load() {
    const user = auth.user;
    if (!user) return;
    parentId = user.role === "parent" ? user.id : parentId;
    if (parentId) {
      dashboard = await api.getParentDashboard(parentId);
      grades = await api.listParentGrades(parentId);
    }
    students = await api.listStudents();
    const smtp = await api.getSmtpSettings();
    smtpConfigured = smtp.configured;
  }

  async function link() {
    if (!parentId || !studentId) return;
    try {
      await api.linkParentStudent({ parent_id: parentId, student_id: studentId });
      studentId = "";
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to link";
    }
  }

  async function exportDigest() {
    if (!parentId) return;
    error = "";
    try {
      const digest = await api.generateParentDigest(parentId);
      const blob = new Blob([digest.html], { type: "text/html" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `classmate-digest-${digest.generated_at.slice(0, 10)}.html`;
      a.click();
      URL.revokeObjectURL(url);
      message = "Weekly digest downloaded.";
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to generate digest";
    }
  }

  async function emailDigest() {
    if (!parentId) return;
    error = "";
    message = "";
    try {
      const result = await api.sendParentDigestEmail({
        parent_id: parentId,
        to_email: emailOverride.trim() || undefined,
      });
      message = `${result.message} → ${result.recipient}`;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to send email";
    }
  }

  onMount(load);
</script>

<div class="page-header">
  <div>
    <h2>Parent portal</h2>
    <p>View linked students, rubric breakdowns, and send the weekly digest by email.</p>
  </div>
  {#if parentId}
    <div style="display:flex;gap:.5rem;flex-wrap:wrap">
      <button class="btn btn-secondary" onclick={exportDigest}>Export HTML digest</button>
      {#if smtpConfigured}
        <button class="btn btn-primary" onclick={emailDigest}>Email digest</button>
      {/if}
    </div>
  {/if}
</div>

{#if auth.user?.role === "admin"}
  <div class="card" style="margin-bottom:1rem">
    <div class="form-grid two-col">
      <label>
        Parent user ID
        <input bind:value={parentId} placeholder="parent user uuid" />
      </label>
      <label>
        Link student
        <select bind:value={studentId}>
          <option value="">Select student</option>
          {#each students as s}
            <option value={s.id}>{s.name}</option>
          {/each}
        </select>
      </label>
      {#if smtpConfigured}
        <label>
          Email override (optional)
          <input type="email" bind:value={emailOverride} placeholder="parent@example.com" />
        </label>
      {/if}
      <button class="btn btn-primary" onclick={link}>Link student</button>
    </div>
  </div>
{/if}

{#if message}<p style="color:var(--success)">{message}</p>{/if}
{#if error}<p class="error">{error}</p>{/if}

{#each dashboard as row}
  <div class="card" style="margin-bottom:1rem">
    <h3 style="margin-top:0">{row.student_name}</h3>
    {#each row.courses as c}
      <div style="padding:.5rem 0;border-top:1px solid var(--border)">
        <strong>{c.course_title}</strong>
        <span style="color:var(--muted);margin-left:.5rem">
          {c.average_percent != null ? `${c.average_percent.toFixed(1)}% avg` : "No grades yet"}
          · {c.assignment_count} assignments
        </span>
      </div>
    {/each}
  </div>
{/each}

{#if grades.length > 0}
  <div class="card">
    <h3 style="margin-top:0">Recent grades</h3>
    <table>
      <thead>
        <tr>
          <th>Student</th>
          <th>Course</th>
          <th>Assignment</th>
          <th>Score</th>
          <th>Rubric</th>
        </tr>
      </thead>
      <tbody>
        {#each grades as g}
          <tr>
            <td>{g.student_name}</td>
            <td>{g.course_title}</td>
            <td>{g.assignment_title}</td>
            <td>{g.points}/{g.max_points}</td>
            <td>
              {#if g.rubric_scores && g.rubric_scores.length > 0}
                {#each g.rubric_scores as s}
                  <div style="font-size:.85rem">{s.name}: {s.points}/{s.max_points}</div>
                {/each}
              {:else}
                —
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

{#if dashboard.length === 0}
  <p class="empty">No linked students yet.</p>
{/if}
