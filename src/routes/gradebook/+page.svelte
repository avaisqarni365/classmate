<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { Course, Gradebook } from "$lib/types";

  let courses = $state<Course[]>([]);
  let selectedCourseId = $state("");
  let gradebook = $state<Gradebook | null>(null);
  let error = $state("");
  let saving = $state("");

  async function loadCourses() {
    courses = await api.listCourses();
    if (!selectedCourseId && courses.length > 0) {
      selectedCourseId = courses[0].id;
    }
  }

  async function loadGradebook() {
    if (!selectedCourseId) return;
    error = "";
    try {
      gradebook = await api.getGradebook(selectedCourseId);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load gradebook";
    }
  }

  async function savePoints(assignmentId: string, studentId: string, raw: string) {
    const key = `${studentId}:${assignmentId}`;
    saving = key;
    try {
      const points = raw.trim() === "" ? null : Number(raw);
      if (points !== null && Number.isNaN(points)) {
        throw new Error("Invalid score");
      }
      await api.saveGrade(assignmentId, studentId, points);
      await loadGradebook();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save grade";
    } finally {
      saving = "";
    }
  }

  async function exportCsv() {
    if (!selectedCourseId || !gradebook) return;
    const csv = await api.exportGradebookCsv(selectedCourseId);
    const blob = new Blob([csv], { type: "text/csv" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${gradebook.course_title.replace(/\s+/g, "-")}-grades.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }

  onMount(async () => {
    await loadCourses();
    await loadGradebook();
  });
</script>

<div class="page-header">
  <div>
    <h2>Gradebook</h2>
    <p>Enter scores for enrolled students by assignment.</p>
  </div>
  <div style="display:flex; gap:0.5rem">
    <select bind:value={selectedCourseId} onchange={loadGradebook}>
      {#each courses as course}
        <option value={course.id}>{course.code} — {course.title}</option>
      {/each}
    </select>
    <button class="btn btn-secondary" onclick={exportCsv} disabled={!gradebook}>Export CSV</button>
  </div>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

{#if !gradebook}
  <p class="empty">Select a course to view the gradebook.</p>
{:else if gradebook.assignments.length === 0}
  <div class="card">
    <p class="empty">No assignments yet. Create assignments from the Courses page.</p>
  </div>
{:else if gradebook.students.length === 0}
  <div class="card">
    <p class="empty">No enrolled students. Enroll students from the Courses page.</p>
  </div>
{:else}
  <div class="card table-wrap">
    <table>
      <thead>
        <tr>
          <th>Student</th>
          {#each gradebook.assignments as assignment}
            <th>
              {assignment.title}
              <div class="sub">/{assignment.max_points}</div>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each gradebook.students as student}
          <tr>
            <td>{student.student_name}</td>
            {#each gradebook.assignments as assignment}
              {@const cell = student.grades[assignment.id]}
              {@const key = `${student.student_id}:${assignment.id}`}
              <td>
                <input
                  class="score-input"
                  type="number"
                  min="0"
                  max={assignment.max_points}
                  step="0.5"
                  value={cell?.points ?? ""}
                  disabled={saving === key}
                  onchange={(e) =>
                    savePoints(
                      assignment.id,
                      student.student_id,
                      (e.currentTarget as HTMLInputElement).value,
                    )}
                />
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
{/if}

<style>
  select {
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 0.55rem 0.75rem;
    background: white;
  }

  .sub {
    font-size: 0.7rem;
    color: var(--muted);
    font-weight: 400;
  }

  .score-input {
    width: 72px;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0.35rem 0.45rem;
  }
</style>
