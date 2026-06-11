<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { CertificateInfo, Course, User } from "$lib/types";

  let courses = $state<Course[]>([]);
  let students = $state<User[]>([]);
  let certificates = $state<CertificateInfo[]>([]);
  let courseId = $state("");
  let studentId = $state("");
  let lastHtml = $state("");
  let error = $state("");

  async function load() {
    courses = await api.listCourses();
    students = await api.listStudents();
    certificates = await api.listCertificates();
    if (!courseId && courses.length) courseId = courses[0].id;
  }

  async function issue() {
    if (!courseId || !studentId) return;
    try {
      const cert = await api.issueCertificate(courseId, studentId);
      lastHtml = cert.html;
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to issue certificate";
    }
  }

  function openHtml(html: string) {
    const w = window.open("", "_blank");
    if (w) {
      w.document.write(html);
      w.document.close();
    }
  }

  onMount(load);
</script>

<div class="page-header">
  <div>
    <h2>Certificates</h2>
    <p>Issue completion certificates for enrolled students.</p>
  </div>
</div>

<div class="card" style="margin-bottom:1rem">
  <div class="form-grid two-col">
    <label>
      Course
      <select bind:value={courseId}>
        {#each courses as c}
          <option value={c.id}>{c.code} — {c.title}</option>
        {/each}
      </select>
    </label>
    <label>
      Student
      <select bind:value={studentId}>
        <option value="">Select student</option>
        {#each students as s}
          <option value={s.id}>{s.name}</option>
        {/each}
      </select>
    </label>
    <button class="btn btn-primary" onclick={issue}>Issue certificate</button>
  </div>
  {#if error}<p class="error">{error}</p>{/if}
</div>

<div class="card table-wrap">
  <table>
    <thead>
      <tr><th>Student</th><th>Course</th><th>Issued</th><th></th></tr>
    </thead>
    <tbody>
      {#each certificates as c}
        <tr>
          <td>{c.student_name}</td>
          <td>{c.course_title}</td>
          <td>{new Date(c.issued_at).toLocaleDateString()}</td>
          <td><button class="btn btn-secondary btn-sm" onclick={() => openHtml(c.html)}>View</button></td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .btn-sm { padding: 0.35rem 0.6rem; font-size: 0.85rem; }
</style>
