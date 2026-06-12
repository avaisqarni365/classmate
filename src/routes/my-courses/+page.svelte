<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { materialNotes, materialPdfUrl, materialReadUrl, parseSpeakNoteContent, parseHandwritingContent } from "$lib/materials";
  import { speakText } from "$lib/speech";
  import AiLabPanel from "$lib/components/AiLabPanel.svelte";
  import { t } from "$lib/i18n";
  import type { StudentCourseDetail, StudentDashboard, MaterialWithAiLab } from "$lib/types";

  let dashboard = $state<StudentDashboard | null>(null);
  let selectedCourseId = $state("");
  let detail = $state<StudentCourseDetail | null>(null);
  let lectures = $state<MaterialWithAiLab[]>([]);
  let error = $state("");
  let submitDrafts = $state<Record<string, string>>({});
  let submitFiles = $state<Record<string, File | null>>({});
  let submitting = $state("");

  async function loadDashboard() {
    error = "";
    dashboard = await api.getStudentDashboard();
    if (!selectedCourseId && dashboard.courses.length > 0) {
      selectedCourseId = dashboard.courses[0].course_id;
      await loadCourse();
    }
  }

  async function loadCourse() {
    if (!selectedCourseId) {
      detail = null;
      return;
    }
    detail = await api.getMyCourse(selectedCourseId);
    lectures = await api.listCourseLectures(selectedCourseId);
  }

  function readFileAsBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = String(reader.result ?? "");
        resolve(result.split(",")[1] ?? "");
      };
      reader.onerror = () => reject(new Error("Could not read file"));
      reader.readAsDataURL(file);
    });
  }

  async function submitWork(assignmentId: string) {
    const body = (submitDrafts[assignmentId] ?? "").trim();
    const file = submitFiles[assignmentId] ?? null;
    if (!body && !file) return;
    if (file && file.size > 2 * 1024 * 1024) {
      error = "File exceeds 2 MB limit";
      return;
    }
    submitting = assignmentId;
    error = "";
    try {
      await api.submitMyAssignment({
        assignment_id: assignmentId,
        body,
        file_name: file?.name,
        file_data: file ? await readFileAsBase64(file) : undefined,
      });
      submitDrafts[assignmentId] = "";
      submitFiles[assignmentId] = null;
      await loadDashboard();
      await loadCourse();
    } catch (e) {
      error = e instanceof Error ? e.message : "Submit failed";
    } finally {
      submitting = "";
    }
  }

  onMount(async () => {
    try {
      await loadDashboard();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load courses";
    }
  });
</script>

<div class="page-header">
  <div>
    <h2>{t("student.title")}</h2>
    <p>{t("student.subtitle")}</p>
  </div>
</div>

{#if error}
  <p class="error">{error}</p>
{:else if dashboard}
  <div class="grid stats-grid" style="margin-bottom: 1rem">
    {#each dashboard.courses as course}
      <button
        type="button"
        class="card course-card"
        class:selected={course.course_id === selectedCourseId}
        onclick={async () => {
          selectedCourseId = course.course_id;
          await loadCourse();
        }}
      >
        <strong>{course.code}</strong>
        <div>{course.title}</div>
        <p style="color: var(--muted); font-size: 0.9rem; margin: 0.5rem 0 0">
          {#if course.average_percent != null}
            {t("student.average")}: {course.average_percent.toFixed(0)}%
          {/if}
          · {course.graded_count}/{course.assignment_count} {t("student.graded").toLowerCase()}
        </p>
      </button>
    {:else}
      <p class="empty">{t("student.noCourses")}</p>
    {/each}
  </div>

  {#if detail}
    <div class="grid two-col">
      <div class="card">
        <h3 style="margin-top: 0">{detail.course.title}</h3>
        {#if detail.course.teacher_name}
          <p style="color: var(--muted)">{t("student.teacher")}: {detail.course.teacher_name}</p>
        {/if}
        <h4>{t("student.assignments")}</h4>
        {#each detail.assignments as a}
          <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.6rem 0">
            <strong>{a.title}</strong>
            <p style="margin: 0.25rem 0; color: var(--muted); font-size: 0.9rem">
              {#if a.points != null}
                {a.points}/{a.max_points}
                {#if a.feedback} — {a.feedback}{/if}
              {:else}
                Not graded · max {a.max_points}
              {/if}
            </p>
            {#if a.rubric_scores && a.rubric_scores.length > 0}
              <ul style="margin:.35rem 0 0;padding-left:1.2rem;font-size:.85rem;color:var(--muted)">
                {#each a.rubric_scores as score}
                  <li>{score.name}: {score.points}/{score.max_points}</li>
                {/each}
              </ul>
            {/if}
            {#if a.points == null}
              <textarea
                rows="2"
                placeholder="Submit your work..."
                bind:value={submitDrafts[a.assignment_id]}
              ></textarea>
              <label style="display: block; margin-top: 0.5rem; font-size: 0.9rem">
                Attach file (max 2 MB)
                <input
                  type="file"
                  onchange={(e) => {
                    const input = e.currentTarget as HTMLInputElement;
                    submitFiles[a.assignment_id] = input.files?.[0] ?? null;
                  }}
                />
              </label>
              <button
                class="btn btn-secondary btn-sm"
                disabled={submitting === a.assignment_id}
                onclick={() => submitWork(a.assignment_id)}
              >
                {submitting === a.assignment_id ? "Submitting..." : "Submit"}
              </button>
            {/if}
          </div>
        {:else}
          <p class="empty">No assignments yet.</p>
        {/each}
      </div>

      <div class="card">
        <h3 style="margin-top: 0">{t("student.materials")}</h3>
        {#each lectures as lecture}
          {@const m = lecture.material}
          <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.6rem 0">
            <strong>{m.title}</strong>
            {#if m.kind === "textbook"}
              <p style="margin: 0.25rem 0; color: var(--muted); font-size: 0.9rem">
                OpenStax textbook
                {#if materialNotes(m)}
                  — {materialNotes(m)}
                {/if}
              </p>
              <p style="margin: 0.25rem 0 0; font-size: 0.9rem">
                {#if materialReadUrl(m)}
                  <a href={materialReadUrl(m)} target="_blank" rel="noopener">Read online</a>
                {/if}
                {#if materialPdfUrl(m)}
                  · <a href={materialPdfUrl(m)} target="_blank" rel="noopener">Download PDF</a>
                {/if}
              </p>
            {:else if m.kind === "speak_note"}
              {@const speak = parseSpeakNoteContent(m.content)}
              <p style="margin: 0.25rem 0; color: var(--muted); font-size: 0.9rem; white-space: pre-wrap">
                {speak?.body ?? m.content}
              </p>
              <button type="button" class="btn btn-secondary btn-sm" onclick={() => speakText(speak?.body ?? "")}>
                Read aloud
              </button>
            {:else if m.kind === "handwriting"}
              {@const ink = parseHandwritingContent(m.content)}
              {#if ink?.preview_data_url}
                <img src={ink.preview_data_url} alt="Handwritten notes" style="max-width:100%;margin-top:.35rem;border:1px solid var(--border);border-radius:6px" />
              {/if}
            {:else if m.kind === "link"}
              <p style="margin: 0.25rem 0; font-size: 0.9rem">
                <a href={m.content} target="_blank" rel="noopener">{m.content}</a>
              </p>
            {:else}
              <p style="margin: 0.25rem 0; color: var(--muted); font-size: 0.9rem; white-space: pre-wrap">{m.content}</p>
            {/if}
            <AiLabPanel
              aiLab={lecture.ai_lab}
              compact
              embed
              studentMode
              materialId={m.id}
              labCompleted={lecture.lab_completed ?? false}
              onLabComplete={loadCourse}
            />
          </div>
        {:else}
          <p class="empty">No materials yet.</p>
        {/each}

        <h4 style="margin-top: 1.25rem">{t("student.upcoming")}</h4>
        {#each dashboard.upcoming as u}
          <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.5rem 0">
            <strong>{u.title}</strong>
            <small style="color: var(--muted); display: block">{u.course_title}</small>
          </div>
        {:else}
          <p class="empty">All caught up.</p>
        {/each}
      </div>
    </div>
  {/if}
{:else}
  <p class="empty">Loading...</p>
{/if}

<style>
  .course-card {
    text-align: left;
    cursor: pointer;
    width: 100%;
    border: 1px solid var(--border);
  }
  .course-card.selected {
    border-color: var(--primary);
    box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 25%, transparent);
  }
  .btn-sm {
    margin-top: 0.5rem;
    padding: 0.35rem 0.65rem;
    font-size: 0.85rem;
  }
</style>
