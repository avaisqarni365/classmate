<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type {
    AssignmentRubric,
    AssignmentSubmission,
    Course,
    RubricScoreInput,
  } from "$lib/types";

  let courses = $state<Course[]>([]);
  let courseId = $state("");
  let submissions = $state<AssignmentSubmission[]>([]);
  let error = $state("");
  let gradingId = $state("");
  let points = $state("");
  let feedback = $state("");
  let activeRubric = $state<AssignmentRubric | null>(null);
  let rubricScores = $state<Record<string, number>>({});

  async function load() {
    error = "";
    courses = await api.listCourses();
    if (!courseId && courses.length > 0) courseId = courses[0].id;
    if (courseId) {
      submissions = await api.listSubmissions(courseId);
    }
  }

  async function startGrading(sub: AssignmentSubmission) {
    gradingId = sub.id;
    points = "";
    feedback = "";
    activeRubric = await api.getAssignmentRubric(sub.assignment_id);
    rubricScores = {};
    if (activeRubric) {
      for (const c of activeRubric.criteria) {
        rubricScores[c.id] = 0;
      }
      points = String(
        activeRubric.criteria.reduce((sum, c) => sum + (rubricScores[c.id] ?? 0), 0),
      );
    }
  }

  function updateRubricTotal() {
    if (!activeRubric) return;
    points = String(
      activeRubric.criteria.reduce((sum, c) => sum + (Number(rubricScores[c.id]) || 0), 0),
    );
  }

  async function grade(submissionId: string) {
    const pts = Number(points);
    if (Number.isNaN(pts)) {
      error = "Enter a valid score";
      return;
    }
    try {
      const rubric_scores: RubricScoreInput[] | undefined = activeRubric
        ? activeRubric.criteria.map((c) => ({
            criterion_id: c.id,
            points: Number(rubricScores[c.id]) || 0,
          }))
        : undefined;
      await api.gradeSubmission({
        submission_id: submissionId,
        points: pts,
        feedback: feedback || undefined,
        rubric_scores,
      });
      gradingId = "";
      points = "";
      feedback = "";
      activeRubric = null;
      rubricScores = {};
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to grade";
    }
  }

  function downloadFile(sub: AssignmentSubmission) {
    if (!sub.file_name || !sub.file_data) return;
    const link = document.createElement("a");
    link.href = `data:application/octet-stream;base64,${sub.file_data}`;
    link.download = sub.file_name;
    link.click();
  }

  onMount(load);
</script>

<div class="page-header">
  <div>
    <h2>Submissions</h2>
    <p>Review student work submitted via Class Hub and sync grades to the gradebook.</p>
  </div>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

<div class="card" style="margin-bottom: 1rem">
  <label>
    Course
    <select bind:value={courseId} onchange={load}>
      {#each courses as c}
        <option value={c.id}>{c.code} — {c.title}</option>
      {/each}
    </select>
  </label>
</div>

<div class="card">
  {#each submissions as sub}
    <div class="item" style="border-bottom: 1px solid var(--border); padding: 1rem 0">
      <div style="display: flex; justify-content: space-between; gap: 1rem; flex-wrap: wrap">
        <div>
          <strong>{sub.student_name}</strong>
          <span class="tag">{sub.status}</span>
          <div style="color: var(--muted); font-size: 0.9rem">{sub.assignment_title}</div>
        </div>
        <small style="color: var(--muted)">{new Date(sub.submitted_at).toLocaleString()}</small>
      </div>
      {#if sub.body}
        <p style="white-space: pre-wrap; margin: 0.75rem 0">{sub.body}</p>
      {/if}
      {#if sub.file_name && sub.file_data}
        <button class="btn btn-secondary btn-sm" type="button" onclick={() => downloadFile(sub)}>
          Download {sub.file_name}
        </button>
      {/if}
      {#if sub.status === "graded" && sub.points != null}
        <p><strong>Score:</strong> {sub.points} {sub.feedback ? `— ${sub.feedback}` : ""}</p>
      {:else if gradingId === sub.id}
        <form
          class="form-grid"
          style="margin-top: 0.5rem"
          onsubmit={(e) => {
            e.preventDefault();
            grade(sub.id);
          }}
        >
          {#if activeRubric}
            {#each activeRubric.criteria as criterion}
              <label>
                {criterion.name} (max {criterion.max_points})
                <input
                  type="number"
                  min="0"
                  max={criterion.max_points}
                  step="0.5"
                  bind:value={rubricScores[criterion.id]}
                  oninput={updateRubricTotal}
                  required
                />
              </label>
            {/each}
          {/if}
          <label>Total points<input type="number" bind:value={points} required readonly={!!activeRubric} /></label>
          <label>Feedback<input bind:value={feedback} /></label>
          <div style="display: flex; gap: 0.5rem">
            <button class="btn btn-primary" type="submit">Save grade</button>
            <button
              class="btn btn-secondary"
              type="button"
              onclick={() => {
                gradingId = "";
                activeRubric = null;
              }}
            >
              Cancel
            </button>
          </div>
        </form>
      {:else}
        <button class="btn btn-secondary" type="button" onclick={() => startGrading(sub)}>Grade</button>
      {/if}
    </div>
  {:else}
    <p class="empty">No submissions yet. Students submit work from the Class Hub portal.</p>
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
  .btn-sm {
    margin-top: 0.5rem;
    padding: 0.35rem 0.65rem;
    font-size: 0.85rem;
  }
</style>
