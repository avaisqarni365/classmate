<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { buildOpenStaxMaterialContent, materialReadUrl, materialPdfUrl, materialNotes, parseSpeakNoteContent, parseHandwritingContent } from "$lib/materials";
  import AiLabPanel from "$lib/components/AiLabPanel.svelte";
  import SpeakNotesEditor from "$lib/components/SpeakNotesEditor.svelte";
  import HandwritingCapture from "$lib/components/HandwritingCapture.svelte";
  import { speakText } from "$lib/speech";
  import WhatsAppShare from "$lib/components/WhatsAppShare.svelte";
  import type {
    Assignment,
    AssignmentRubric,
    Course,
    CourseMaterial,
    Enrollment,
    MaterialWithAiLab,
    OpenStaxBook,
    RubricCriterionInput,
    User,
  } from "$lib/types";

  let courses = $state<Course[]>([]);
  let teachers = $state<User[]>([]);
  let students = $state<User[]>([]);
  let selectedCourseId = $state<string | null>(null);
  let assignments = $state<Assignment[]>([]);
  let enrollments = $state<Enrollment[]>([]);
  let materials = $state<CourseMaterial[]>([]);
  let lectures = $state<MaterialWithAiLab[]>([]);
  let error = $state("");
  let showCourseForm = $state(false);
  let showAssignmentForm = $state(false);
  let showMaterialForm = $state(false);
  let showOpenStaxForm = $state(false);
  let showSpeakNotesForm = $state(false);
  let showHandwritingForm = $state(false);
  let openStaxBooks = $state<OpenStaxBook[]>([]);
  let openStaxSubject = $state("");
  let openStaxSearch = $state("");
  let openStaxNotes = $state("");
  let openStaxLoading = $state(false);

  let title = $state("");
  let code = $state("");
  let description = $state("");
  let teacherId = $state("");
  let term = $state("2026 Spring");

  let assignmentTitle = $state("");
  let assignmentDesc = $state("");
  let assignmentPoints = $state(100);

  let materialTitle = $state("");
  let materialKind = $state("note");
  let materialContent = $state("");
  let enrollStudentId = $state("");
  let rubricAssignmentId = $state("");
  let rubricCriteria = $state<RubricCriterionInput[]>([]);
  let rubricMessage = $state("");

  const rubricTotal = $derived(
    rubricCriteria.reduce((sum, c) => sum + (Number(c.max_points) || 0), 0),
  );

  const selectedCourse = $derived(
    courses.find((c) => c.id === selectedCourseId) ?? null,
  );

  async function load() {
    error = "";
    try {
      const [courseList, userList, studentList] = await Promise.all([
        api.listCourses(),
        api.listUsers(),
        api.listStudents(),
      ]);
      courses = courseList;
      teachers = userList.filter((u) => u.role === "teacher" || u.role === "admin");
      students = studentList;
      if (!selectedCourseId && courses.length > 0) {
        selectedCourseId = courses[0].id;
      }
      await loadCourseDetails();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to load courses";
    }
  }

  async function loadCourseDetails() {
    if (!selectedCourseId) return;
    const [assignmentList, enrollmentList, materialList] = await Promise.all([
      api.listAssignments(selectedCourseId),
      api.listEnrollments(selectedCourseId),
      api.listMaterials(selectedCourseId),
    ]);
    assignments = assignmentList;
    enrollments = enrollmentList.filter((e) => e.status === "active");
    materials = materialList;
    lectures = await api.listCourseLectures(selectedCourseId);
  }

  async function createCourse(event: Event) {
    event.preventDefault();
    error = "";
    try {
      await api.createCourse({
        title,
        code,
        description: description || undefined,
        teacher_id: teacherId || undefined,
        term: term || undefined,
      });
      title = "";
      code = "";
      description = "";
      showCourseForm = false;
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create course";
    }
  }

  async function createAssignment(event: Event) {
    event.preventDefault();
    if (!selectedCourseId) return;
    try {
      await api.createAssignment({
        course_id: selectedCourseId,
        title: assignmentTitle,
        description: assignmentDesc || undefined,
        max_points: assignmentPoints,
      });
      assignmentTitle = "";
      assignmentDesc = "";
      assignmentPoints = 100;
      showAssignmentForm = false;
      await loadCourseDetails();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create assignment";
    }
  }

  async function createMaterial(event: Event) {
    event.preventDefault();
    if (!selectedCourseId) return;
    try {
      await api.createMaterial({
        course_id: selectedCourseId,
        title: materialTitle,
        kind: materialKind,
        content: materialContent,
      });
      materialTitle = "";
      materialContent = "";
      showMaterialForm = false;
      await loadCourseDetails();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to create material";
    }
  }

  async function enrollStudent() {
    if (!selectedCourseId || !enrollStudentId) return;
    try {
      await api.enrollStudent(selectedCourseId, enrollStudentId);
      enrollStudentId = "";
      await loadCourseDetails();
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to enroll student";
    }
  }

  async function removeEnrollment(enrollmentId: string) {
    try {
      await api.unenrollStudent(enrollmentId);
      await loadCourseDetails();
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to remove enrollment";
    }
  }

  async function removeMaterial(materialId: string) {
    try {
      await api.deleteMaterial(materialId);
      await loadCourseDetails();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to delete material";
    }
  }

  async function loadOpenStaxBooks() {
    openStaxLoading = true;
    error = "";
    try {
      openStaxBooks = await api.listOpenStaxBooks(openStaxSubject || undefined);
    } catch (e) {
      error = e instanceof Error ? e.message : "Could not load OpenStax catalog";
      openStaxBooks = [];
    } finally {
      openStaxLoading = false;
    }
  }

  async function attachOpenStaxBook(book: OpenStaxBook) {
    if (!selectedCourseId) return;
    error = "";
    try {
      await api.createMaterial({
        course_id: selectedCourseId,
        title: book.title,
        kind: "textbook",
        content: buildOpenStaxMaterialContent(book, openStaxNotes),
      });
      showOpenStaxForm = false;
      openStaxNotes = "";
      await loadCourseDetails();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to attach textbook";
    }
  }

  const filteredOpenStaxBooks = $derived(
    openStaxBooks.filter((book) => {
      const q = openStaxSearch.trim().toLowerCase();
      if (!q) return true;
      return (
        book.title.toLowerCase().includes(q) ||
        book.subjects.some((s) => s.toLowerCase().includes(q))
      );
    }),
  );

  async function openRubric(assignmentId: string) {
    rubricAssignmentId = assignmentId;
    rubricMessage = "";
    const existing = await api.getAssignmentRubric(assignmentId);
    rubricCriteria = existing?.criteria.map((c) => ({
      id: c.id,
      name: c.name,
      description: c.description ?? undefined,
      max_points: c.max_points,
    })) ?? [{ name: "", description: "", max_points: 25 }];
  }

  function addRubricRow() {
    rubricCriteria = [...rubricCriteria, { name: "", description: "", max_points: 10 }];
  }

  function removeRubricRow(index: number) {
    rubricCriteria = rubricCriteria.filter((_, i) => i !== index);
  }

  async function saveRubric() {
    if (!rubricAssignmentId) return;
    try {
      await api.saveAssignmentRubric({
        assignment_id: rubricAssignmentId,
        criteria: rubricCriteria,
      });
      rubricMessage = "Rubric saved.";
      rubricAssignmentId = "";
      await loadCourseDetails();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save rubric";
    }
  }

  async function selectCourse(id: string) {
    selectedCourseId = id;
    await loadCourseDetails();
  }

  onMount(load);
</script>

<div class="page-header">
  <div>
    <h2>Courses</h2>
    <p>Manage classes, enrollments, materials, and assignments.</p>
  </div>
  <button class="btn btn-primary" onclick={() => (showCourseForm = !showCourseForm)}>
    {showCourseForm ? "Cancel" : "New course"}
  </button>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

{#if showCourseForm}
  <div class="card" style="margin-bottom: 1rem">
    <form class="form-grid two-col" onsubmit={createCourse}>
      <label>
        Title
        <input bind:value={title} required placeholder="Introduction to Biology" />
      </label>
      <label>
        Code
        <input bind:value={code} required placeholder="BIO-101" />
      </label>
      <label>
        Term
        <input bind:value={term} placeholder="2026 Spring" />
      </label>
      <label>
        Teacher
        <select bind:value={teacherId}>
          <option value="">Unassigned</option>
          {#each teachers as teacher}
            <option value={teacher.id}>{teacher.name}</option>
          {/each}
        </select>
      </label>
      <label style="grid-column: 1 / -1">
        Description
        <textarea bind:value={description} rows="3"></textarea>
      </label>
      <div>
        <button class="btn btn-primary" type="submit">Create course</button>
      </div>
    </form>
  </div>
{/if}

<div class="grid two-col">
  <div class="card table-wrap">
    <h3 style="margin-top:0">All courses</h3>
    {#if courses.length === 0}
      <p class="empty">No courses yet.</p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Code</th>
            <th>Title</th>
            <th>Students</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each courses as course}
            <tr class:selected={course.id === selectedCourseId}>
              <td><span class="badge">{course.code}</span></td>
              <td>{course.title}</td>
              <td>{course.student_count}</td>
              <td>
                <button class="btn btn-secondary" onclick={() => selectCourse(course.id)}>
                  Manage
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

  <div class="card">
    {#if !selectedCourse}
      <p class="empty">Select a course to manage content.</p>
    {:else}
      <h3 style="margin-top:0">{selectedCourse.code} — {selectedCourse.title}</h3>

      <section class="section">
        <div class="section-head">
          <h4>Enrolled students</h4>
          <div class="inline-form">
            <select bind:value={enrollStudentId}>
              <option value="">Add student...</option>
              {#each students as student}
                <option value={student.id}>{student.name}</option>
              {/each}
            </select>
            <button class="btn btn-secondary" onclick={enrollStudent} disabled={!enrollStudentId}>
              Enroll
            </button>
          </div>
        </div>
        {#if enrollments.length === 0}
          <p class="empty">No students enrolled.</p>
        {:else}
          <ul class="plain-list">
            {#each enrollments as enrollment}
              <li>
                <span>{enrollment.student_name}</span>
                <button
                  class="btn btn-danger btn-sm"
                  onclick={() => removeEnrollment(enrollment.id)}
                >
                  Remove
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="section">
        <div class="section-head">
          <h4>Lecture notes &amp; materials</h4>
          <div style="display:flex;gap:.35rem;flex-wrap:wrap">
            <button
              class="btn btn-secondary btn-sm"
              onclick={async () => {
                showOpenStaxForm = !showOpenStaxForm;
                showMaterialForm = false;
                showSpeakNotesForm = false;
                showHandwritingForm = false;
                if (showOpenStaxForm) await loadOpenStaxBooks();
              }}
            >
              {showOpenStaxForm ? "Cancel" : "Add OpenStax book"}
            </button>
            <button
              class="btn btn-secondary btn-sm"
              onclick={() => {
                showSpeakNotesForm = !showSpeakNotesForm;
                showMaterialForm = false;
                showOpenStaxForm = false;
                showHandwritingForm = false;
              }}
            >
              {showSpeakNotesForm ? "Cancel" : "Speak notes"}
            </button>
            <button
              class="btn btn-secondary btn-sm"
              onclick={() => {
                showHandwritingForm = !showHandwritingForm;
                showMaterialForm = false;
                showOpenStaxForm = false;
                showSpeakNotesForm = false;
              }}
            >
              {showHandwritingForm ? "Cancel" : "Tablet handwriting"}
            </button>
            <button class="btn btn-secondary btn-sm" onclick={() => {
              showMaterialForm = !showMaterialForm;
              showOpenStaxForm = false;
              showSpeakNotesForm = false;
              showHandwritingForm = false;
            }}>
              {showMaterialForm ? "Cancel" : "Add note/link"}
            </button>
          </div>
        </div>
        {#if showOpenStaxForm}
          <div class="openstax-panel">
            <p class="muted" style="margin-top:0;font-size:.9rem">
              Attach free peer-reviewed textbooks from <a href="https://openstax.org" target="_blank" rel="noopener">OpenStax</a>
              plus an <a href="https://artizai.uk" target="_blank" rel="noopener">ARTIZAI Academy</a> lab on every lecture note.
            </p>
            <div class="form-grid" style="margin-bottom:.75rem">
              <label>
                Filter by subject
                <select bind:value={openStaxSubject} onchange={loadOpenStaxBooks}>
                  <option value="">All subjects</option>
                  <option value="Math">Math</option>
                  <option value="Science">Science</option>
                  <option value="Business">Business</option>
                  <option value="Humanities">Humanities</option>
                  <option value="Social">Social Sciences</option>
                  <option value="Nursing">Nursing</option>
                  <option value="Computer">Computer Science</option>
                </select>
              </label>
              <label>
                Search
                <input bind:value={openStaxSearch} placeholder="Biology, Algebra, Physics…" />
              </label>
              <label style="grid-column:1/-1">
                Teacher notes (optional)
                <textarea bind:value={openStaxNotes} rows="2" placeholder="Read chapters 1–3 before next class"></textarea>
              </label>
            </div>
            {#if openStaxLoading}
              <p class="empty">Loading OpenStax catalog…</p>
            {:else if filteredOpenStaxBooks.length === 0}
              <p class="empty">No books match your filter.</p>
            {:else}
              <ul class="openstax-list">
                {#each filteredOpenStaxBooks as book}
                  <li>
                    <div>
                      <strong>{book.title}</strong>
                      <div class="muted" style="font-size:.85rem;margin-top:.2rem">
                        {book.subjects.join(" · ")}
                      </div>
                    </div>
                    <button class="btn btn-primary btn-sm" type="button" onclick={() => attachOpenStaxBook(book)}>
                      Attach
                    </button>
                  </li>
                {/each}
              </ul>
            {/if}
          </div>
        {/if}
        {#if showSpeakNotesForm && selectedCourseId}
          <SpeakNotesEditor
            courseId={selectedCourseId}
            onSaved={async () => {
              showSpeakNotesForm = false;
              await loadCourseDetails();
            }}
            onCancel={() => (showSpeakNotesForm = false)}
          />
        {/if}
        {#if showHandwritingForm && selectedCourseId}
          <HandwritingCapture
            courseId={selectedCourseId}
            onAttached={async () => {
              showHandwritingForm = false;
              await loadCourseDetails();
            }}
            onCancel={() => (showHandwritingForm = false)}
          />
        {/if}
        {#if showMaterialForm}
          <form class="form-grid" onsubmit={createMaterial}>
            <label>
              Title
              <input bind:value={materialTitle} required />
            </label>
            <label>
              Type
              <select bind:value={materialKind}>
                <option value="note">Note</option>
                <option value="link">Link</option>
                <option value="file">File reference</option>
              </select>
            </label>
            <label>
              Content
              <textarea bind:value={materialContent} rows="3" required></textarea>
            </label>
            <button class="btn btn-primary" type="submit">Save material</button>
          </form>
        {/if}
        {#if lectures.length === 0}
          <p class="empty">No materials yet. Attach an OpenStax textbook or add your own notes — each includes an ARTIZAI lab.</p>
        {:else}
          <ul class="plain-list lecture-list">
            {#each lectures as lecture}
              <li>
                <div style="flex:1">
                  <span>
                    <span class="badge">{lecture.material.kind === "textbook" ? "OpenStax" : lecture.material.kind === "speak_note" ? "Speak notes" : lecture.material.kind === "handwriting" ? "Handwriting" : lecture.material.kind}</span>
                    {lecture.material.title}
                  </span>
                  {#if lecture.material.kind === "textbook"}
                    {@const readUrl = materialReadUrl(lecture.material)}
                    {@const pdfUrl = materialPdfUrl(lecture.material)}
                    {@const notes = materialNotes(lecture.material)}
                    {#if notes}
                      <span class="muted" style="display:block;font-size:.85rem;margin-top:.2rem">{notes}</span>
                    {/if}
                    <span style="display:block;font-size:.85rem;margin-top:.25rem">
                      {#if readUrl}
                        <a href={readUrl} target="_blank" rel="noopener">Read online</a>
                      {/if}
                      {#if pdfUrl}
                        · <a href={pdfUrl} target="_blank" rel="noopener">PDF</a>
                      {/if}
                    </span>
                  {:else if lecture.material.kind === "speak_note"}
                    {@const speak = parseSpeakNoteContent(lecture.material.content)}
                    {#if speak?.outline}
                      <span class="muted" style="display:block;font-size:.85rem;margin-top:.2rem"><strong>Outline:</strong> {speak.outline}</span>
                    {/if}
                    <span class="muted" style="display:block;font-size:.85rem;margin-top:.2rem;white-space:pre-wrap">{speak?.body ?? lecture.material.content}</span>
                    <button type="button" class="btn btn-secondary btn-sm" style="margin-top:.35rem" onclick={() => speakText(speak?.body ?? "")}>
                      Read aloud
                    </button>
                  {:else if lecture.material.kind === "handwriting"}
                    {@const ink = parseHandwritingContent(lecture.material.content)}
                    {#if ink?.preview_data_url}
                      <img src={ink.preview_data_url} alt="Handwritten notes" style="max-width:100%;margin-top:.35rem;border:1px solid var(--border);border-radius:6px" />
                    {:else}
                      <span class="muted" style="display:block;font-size:.85rem;margin-top:.2rem">Handwritten ink attached</span>
                    {/if}
                  {:else if lecture.material.kind === "link"}
                    <a href={lecture.material.content} target="_blank" rel="noopener" style="display:block;font-size:.85rem;margin-top:.25rem">{lecture.material.content}</a>
                  {:else}
                    <span class="muted" style="display:block;font-size:.85rem;margin-top:.2rem;white-space:pre-wrap">{lecture.material.content}</span>
                  {/if}
                  <AiLabPanel aiLab={lecture.ai_lab} compact />
                </div>
                <button class="btn btn-danger btn-sm" onclick={() => removeMaterial(lecture.material.id)}>
                  Delete
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </section>

      <section class="section">
        <div class="section-head">
          <h4>Assignments</h4>
          <button
            class="btn btn-secondary btn-sm"
            onclick={() => (showAssignmentForm = !showAssignmentForm)}
          >
            {showAssignmentForm ? "Cancel" : "Add"}
          </button>
        </div>
        {#if showAssignmentForm}
          <form class="form-grid" onsubmit={createAssignment}>
            <label>
              Title
              <input bind:value={assignmentTitle} required />
            </label>
            <label>
              Max points
              <input type="number" bind:value={assignmentPoints} min="1" />
            </label>
            <label>
              Description
              <textarea bind:value={assignmentDesc} rows="2"></textarea>
            </label>
            <button class="btn btn-primary" type="submit">Save assignment</button>
          </form>
        {/if}
        {#if assignments.length === 0}
          <p class="empty">No assignments yet.</p>
        {:else}
          <ul class="plain-list">
            {#each assignments as assignment}
              <li style="display:flex;justify-content:space-between;gap:.5rem;align-items:center">
                <span>{assignment.title} ({assignment.max_points} pts)</span>
                <div style="display:flex;gap:.35rem">
                  <WhatsAppShare
                    kind="task"
                    assignmentId={assignment.id}
                    courseId={selectedCourseId ?? undefined}
                    label="Share"
                  />
                  <button class="btn btn-secondary btn-sm" type="button" onclick={() => openRubric(assignment.id)}>
                    Rubric
                  </button>
                </div>
              </li>
            {/each}
          </ul>
        {/if}
        {#if rubricAssignmentId}
          <div class="card" style="margin-top:1rem;background:#f8fafc">
            <h4 style="margin-top:0">Edit rubric</h4>
            {#each rubricCriteria as criterion, index}
              <div class="form-grid" style="margin-bottom:.75rem">
                <label>
                  Criterion
                  <input bind:value={criterion.name} required />
                </label>
                <label>
                  Points
                  <input type="number" min="1" step="0.5" bind:value={criterion.max_points} />
                </label>
                <label>
                  Description
                  <input bind:value={criterion.description} />
                </label>
                <button class="btn btn-danger btn-sm" type="button" onclick={() => removeRubricRow(index)}>
                  Remove
                </button>
              </div>
            {/each}
            <p style="color:var(--muted);font-size:.9rem">Total: {rubricTotal} points</p>
            <div style="display:flex;gap:.5rem;flex-wrap:wrap">
              <button class="btn btn-secondary" type="button" onclick={addRubricRow}>Add criterion</button>
              <button class="btn btn-primary" type="button" onclick={saveRubric}>Save rubric</button>
              <button class="btn btn-secondary" type="button" onclick={() => (rubricAssignmentId = "")}>Cancel</button>
            </div>
            {#if rubricMessage}<p style="color:var(--success)">{rubricMessage}</p>{/if}
          </div>
        {/if}
      </section>
    {/if}
  </div>
</div>

<style>
  tr.selected {
    background: #eff6ff;
  }

  .section {
    margin-top: 1.25rem;
    padding-top: 1rem;
    border-top: 1px solid var(--border);
  }

  .section-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .section-head h4 {
    margin: 0;
  }

  .inline-form {
    display: flex;
    gap: 0.35rem;
  }

  .plain-list {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .plain-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
    padding: 0.45rem 0;
    border-bottom: 1px solid var(--border);
  }

  .btn-sm {
    padding: 0.35rem 0.6rem;
    font-size: 0.8rem;
  }

  .openstax-panel {
    background: #f8fafc;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0.85rem;
    margin-bottom: 0.75rem;
  }

  .openstax-list {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 280px;
    overflow-y: auto;
  }

  .openstax-list li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.75rem;
    padding: 0.55rem 0;
    border-bottom: 1px solid var(--border);
  }

  .lecture-list li {
    align-items: flex-start;
    flex-wrap: wrap;
  }

  @media (max-width: 900px) {
    .openstax-list li {
      flex-direction: column;
      align-items: flex-start;
    }

    .inline-form select {
      min-width: 0;
      flex: 1 1 100%;
    }
  }
</style>
