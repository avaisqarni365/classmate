<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { Course, Quiz, QuizAttempt, QuizAttemptDetail, QuizDetail } from "$lib/types";

  let courses = $state<Course[]>([]);
  let courseId = $state("");
  let quizzes = $state<Quiz[]>([]);
  let selectedQuizId = $state("");
  let detail = $state<QuizDetail | null>(null);
  let attempts = $state<QuizAttempt[]>([]);
  let reviewAttempt = $state<QuizAttemptDetail | null>(null);
  let reviewScore = $state("");
  let reviewFeedback = $state("");
  let error = $state("");

  let quizTitle = $state("");
  let quizDescription = $state("");
  let questionPrompt = $state("");
  let optionA = $state("");
  let optionB = $state("");
  let optionC = $state("");
  let optionD = $state("");
  let correctIndex = $state(0);
  let questionPoints = $state(1);
  let questionKind = $state<"mcq" | "short_answer">("mcq");
  let correctText = $state("");

  async function loadCourses() {
    courses = await api.listCourses();
    if (!courseId && courses.length > 0) {
      courseId = courses[0].id;
    }
  }

  async function loadQuizzes() {
    if (!courseId) return;
    error = "";
    quizzes = await api.listQuizzes(courseId);
    if (!selectedQuizId && quizzes.length > 0) {
      selectedQuizId = quizzes[0].id;
    }
    if (selectedQuizId) {
      await loadDetail();
    } else {
      detail = null;
      attempts = [];
    }
  }

  async function loadDetail() {
    if (!selectedQuizId) return;
    detail = await api.getQuizDetail(selectedQuizId);
    attempts = await api.listQuizAttempts(selectedQuizId);
  }

  async function createQuiz(event: Event) {
    event.preventDefault();
    try {
      const quiz = await api.createQuiz({
        course_id: courseId,
        title: quizTitle,
        description: quizDescription || undefined,
      });
      quizTitle = "";
      quizDescription = "";
      selectedQuizId = quiz.id;
      await loadQuizzes();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create quiz";
    }
  }

  async function addQuestion(event: Event) {
    event.preventDefault();
    if (!selectedQuizId) return;
    const options =
      questionKind === "mcq" ? [optionA, optionB, optionC, optionD].filter((o) => o.trim()) : [];
    try {
      await api.addQuizQuestion({
        quiz_id: selectedQuizId,
        prompt: questionPrompt,
        kind: questionKind,
        options,
        correct_index: correctIndex,
        correct_text: questionKind === "short_answer" ? correctText : undefined,
        points: questionPoints,
      });
      questionPrompt = "";
      optionA = "";
      optionB = "";
      optionC = "";
      optionD = "";
      correctIndex = 0;
      correctText = "";
      questionPoints = 1;
      await loadQuizzes();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to add question";
    }
  }

  async function publish() {
    if (!selectedQuizId) return;
    try {
      await api.publishQuiz(selectedQuizId);
      await loadQuizzes();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to publish quiz";
    }
  }

  async function openReview(attemptId: string) {
    error = "";
    reviewAttempt = await api.getQuizAttemptDetail(attemptId);
    reviewScore = String(reviewAttempt.attempt.score);
    reviewFeedback = reviewAttempt.attempt.feedback ?? "";
  }

  async function saveReview() {
    if (!reviewAttempt) return;
    const score = Number(reviewScore);
    if (Number.isNaN(score)) {
      error = "Enter a valid score";
      return;
    }
    try {
      await api.gradeQuizAttempt({
        attempt_id: reviewAttempt.attempt.id,
        score,
        feedback: reviewFeedback || undefined,
      });
      reviewAttempt = null;
      await loadDetail();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save grade";
    }
  }

  function answerFor(questionId: string) {
    return reviewAttempt?.answers.find((a) => a.question_id === questionId);
  }

  onMount(async () => {
    await loadCourses();
    await loadQuizzes();
  });
</script>

<div class="page-header">
  <div>
    <h2>Quizzes</h2>
    <p>Create multiple-choice and short-answer quizzes for the Class Hub student portal.</p>
  </div>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

<div class="grid two-col" style="margin-bottom: 1rem">
  <div class="card">
    <h3 style="margin-top: 0">Course</h3>
    <label>
      Select course
      <select
        bind:value={courseId}
        onchange={async () => {
          selectedQuizId = "";
          await loadQuizzes();
        }}
      >
        {#each courses as c}
          <option value={c.id}>{c.code} — {c.title}</option>
        {/each}
      </select>
    </label>
  </div>

  <div class="card">
    <h3 style="margin-top: 0">New quiz</h3>
    <form class="form-grid" onsubmit={createQuiz}>
      <label>Title<input bind:value={quizTitle} required /></label>
      <label>Description<textarea bind:value={quizDescription} rows="2"></textarea></label>
      <button class="btn btn-primary" type="submit">Create draft quiz</button>
    </form>
  </div>
</div>

<div class="grid two-col">
  <div class="card">
    <h3 style="margin-top: 0">Quizzes</h3>
    {#each quizzes as q}
      <button
        type="button"
        class="quiz-row"
        class:selected={q.id === selectedQuizId}
        onclick={async () => {
          selectedQuizId = q.id;
          await loadDetail();
        }}
      >
        <strong>{q.title}</strong>
        <span class="tag">{q.status}</span>
        <small>{q.question_count} questions · {q.attempt_count} attempts</small>
      </button>
    {:else}
      <p class="empty">No quizzes for this course yet.</p>
    {/each}
  </div>

  {#if detail}
    <div class="card">
      <h3 style="margin-top: 0">{detail.quiz.title}</h3>
      <p style="color: var(--muted)">{detail.quiz.description || "No description"}</p>
      <p>
        Status: <strong>{detail.quiz.status}</strong> · Max points: {detail.quiz.max_points}
      </p>

      {#if detail.quiz.status === "draft"}
        <form class="form-grid" style="margin-top: 1rem" onsubmit={addQuestion}>
          <h4 style="margin: 0">Add question</h4>
          <label>
            Type
            <select bind:value={questionKind}>
              <option value="mcq">Multiple choice</option>
              <option value="short_answer">Short answer</option>
            </select>
          </label>
          <label>Prompt<textarea bind:value={questionPrompt} rows="2" required></textarea></label>
          {#if questionKind === "mcq"}
            <label>Option A<input bind:value={optionA} required /></label>
            <label>Option B<input bind:value={optionB} required /></label>
            <label>Option C<input bind:value={optionC} /></label>
            <label>Option D<input bind:value={optionD} /></label>
            <label>
              Correct answer
              <select bind:value={correctIndex}>
                <option value={0}>A</option>
                <option value={1}>B</option>
                <option value={2}>C</option>
                <option value={3}>D</option>
              </select>
            </label>
          {:else}
            <label>
              Expected answer (optional, for auto-grade)
              <input bind:value={correctText} placeholder="Observation" />
            </label>
          {/if}
          <label>Points<input type="number" min="0.5" step="0.5" bind:value={questionPoints} /></label>
          <button class="btn btn-secondary" type="submit">Add question</button>
        </form>
        {#if detail.questions.length > 0}
          <button class="btn btn-primary" style="margin-top: 1rem" onclick={publish}>
            Publish to Class Hub
          </button>
        {/if}
      {/if}

      <h4>Questions</h4>
      {#each detail.questions as q, i}
        <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.5rem 0">
          <strong>{i + 1}. {q.prompt}</strong>
          <span class="tag">{q.kind}</span>
          {#if q.kind === "short_answer"}
            <p style="color: var(--muted); margin: 0.35rem 0">
              Expected: {q.correct_text || "Manual review"}
            </p>
          {:else}
            <ul style="margin: 0.35rem 0; color: var(--muted)">
              {#each q.options as opt, j}
                <li class:correct={j === q.correct_index}>{opt}</li>
              {/each}
            </ul>
          {/if}
        </div>
      {:else}
        <p class="empty">No questions yet.</p>
      {/each}

      <h4>Attempts</h4>
      {#each attempts as a}
        <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.5rem 0">
          <div style="display: flex; justify-content: space-between; gap: 0.5rem; flex-wrap: wrap">
            <div>
              <strong>{a.student_name}</strong>
              {#if a.review_status === "pending"}
                <span class="tag pending">Needs review</span>
              {/if}
              <span>{a.score}/{a.max_score}</span>
            </div>
            <button class="btn btn-secondary btn-sm" type="button" onclick={() => openReview(a.id)}>
              Review
            </button>
          </div>
          <small style="color: var(--muted); display: block">
            {new Date(a.submitted_at).toLocaleString()}
          </small>
          {#if a.feedback}
            <p style="margin: 0.35rem 0 0; color: var(--muted); font-size: 0.9rem">{a.feedback}</p>
          {/if}
        </div>
      {:else}
        <p class="empty">No student attempts yet.</p>
      {/each}
    </div>
  {/if}
</div>

{#if reviewAttempt}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-backdrop" role="presentation" onclick={() => (reviewAttempt = null)}>
    <div class="card modal" role="dialog" aria-modal="true" tabindex="-1" onclick={(e) => e.stopPropagation()}>
      <h3 style="margin-top: 0">Review: {reviewAttempt.attempt.student_name}</h3>
      <p style="color: var(--muted)">
        Auto score {reviewAttempt.attempt.score}/{reviewAttempt.attempt.max_score}
      </p>
      {#each reviewAttempt.questions as q, i}
        {@const ans = answerFor(q.id)}
        <div class="item" style="border-bottom: 1px solid var(--border); padding: 0.6rem 0">
          <strong>{i + 1}. {q.prompt}</strong>
          <span class="tag">{q.kind}</span>
          {#if q.kind === "short_answer"}
            <p style="margin: 0.35rem 0">
              Answer: <strong>{ans?.text_answer || "—"}</strong>
            </p>
            <p style="color: var(--muted); margin: 0">
              Expected: {q.correct_text || "Manual grading"}
            </p>
          {:else if ans?.selected_index != null && q.options[ans.selected_index] != null}
            <p style="margin: 0.35rem 0">
              Selected: {q.options[ans.selected_index]}
              {#if ans.selected_index === q.correct_index}
                <span class="correct">Correct</span>
              {/if}
            </p>
          {/if}
        </div>
      {/each}
      <form class="form-grid" style="margin-top: 1rem" onsubmit={(e) => { e.preventDefault(); saveReview(); }}>
        <label>
          Final score (max {reviewAttempt.attempt.max_score})
          <input type="number" min="0" max={reviewAttempt.attempt.max_score} step="0.5" bind:value={reviewScore} required />
        </label>
        <label>
          Feedback
          <textarea bind:value={reviewFeedback} rows="2"></textarea>
        </label>
        <div style="display: flex; gap: 0.5rem">
          <button class="btn btn-primary" type="submit">Save grade</button>
          <button class="btn btn-secondary" type="button" onclick={() => (reviewAttempt = null)}>Cancel</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .quiz-row {
    display: block;
    width: 100%;
    text-align: left;
    background: transparent;
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 0.75rem;
    margin-bottom: 0.5rem;
    cursor: pointer;
  }
  .quiz-row.selected {
    border-color: var(--primary);
    background: #eff6ff;
  }
  .tag {
    display: inline-block;
    margin-left: 0.5rem;
    font-size: 0.75rem;
    padding: 0.1rem 0.45rem;
    border-radius: 999px;
    background: #f1f5f9;
  }
  .tag.pending {
    background: #fef3c7;
    color: #92400e;
  }
  .btn-sm {
    padding: 0.25rem 0.55rem;
    font-size: 0.8rem;
  }
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    z-index: 50;
  }
  .modal {
    width: min(640px, 100%);
    max-height: 90vh;
    overflow: auto;
  }
  .correct {
    color: #059669;
    font-weight: 600;
  }
</style>
