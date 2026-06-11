<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { Course, ForumPost, ForumTopic } from "$lib/types";

  let courses = $state<Course[]>([]);
  let courseId = $state("");
  let topics = $state<ForumTopic[]>([]);
  let selectedTopicId = $state("");
  let posts = $state<ForumPost[]>([]);
  let topicTitle = $state("");
  let topicBody = $state("");
  let authorName = $state("Teacher");
  let replyBody = $state("");
  let error = $state("");

  async function loadTopics() {
    if (!courseId) return;
    topics = await api.listForumTopics(courseId);
  }

  async function loadPosts() {
    if (!selectedTopicId) return;
    posts = await api.listForumPosts(selectedTopicId);
  }

  async function createTopic(event: Event) {
    event.preventDefault();
    try {
      await api.createForumTopic({
        course_id: courseId,
        title: topicTitle,
        author_name: authorName,
        body: topicBody,
      });
      topicTitle = "";
      topicBody = "";
      await loadTopics();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed";
    }
  }

  async function reply(event: Event) {
    event.preventDefault();
    try {
      await api.createForumPost({
        topic_id: selectedTopicId,
        author_name: authorName,
        body: replyBody,
      });
      replyBody = "";
      await loadPosts();
      await loadTopics();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed";
    }
  }

  onMount(async () => {
    courses = await api.listCourses();
    if (courses.length) {
      courseId = courses[0].id;
      await loadTopics();
    }
  });
</script>

<div class="page-header">
  <div>
    <h2>Forums</h2>
    <p>Course discussions visible to students on the class hub.</p>
  </div>
</div>

{#if error}<p class="error">{error}</p>{/if}

<div class="grid two-col">
  <div class="card">
    <label>
      Course
      <select bind:value={courseId} onchange={loadTopics}>
        {#each courses as c}
          <option value={c.id}>{c.code}</option>
        {/each}
      </select>
    </label>
    <form class="form-grid" style="margin-top:1rem" onsubmit={createTopic}>
      <label>Topic title<input bind:value={topicTitle} required /></label>
      <label>First post<textarea bind:value={topicBody} rows="3" required /></label>
      <button class="btn btn-primary" type="submit">Create topic</button>
    </form>
    <ul style="list-style:none;padding:0;margin-top:1rem">
      {#each topics as t}
        <li style="padding:.5rem 0;border-top:1px solid var(--border)">
          <button class="btn btn-secondary btn-sm" onclick={() => { selectedTopicId = t.id; loadPosts(); }}>
            {t.title} ({t.post_count})
          </button>
        </li>
      {/each}
    </ul>
  </div>
  <div class="card">
    <h3 style="margin-top:0">Thread</h3>
    {#if !selectedTopicId}
      <p class="empty">Select a topic.</p>
    {:else}
      {#each posts as p}
        <div style="padding:.6rem 0;border-bottom:1px solid var(--border)">
          <strong>{p.author_name}</strong>
          <p>{p.body}</p>
        </div>
      {/each}
      <form class="form-grid" style="margin-top:1rem" onsubmit={reply}>
        <label>Reply<textarea bind:value={replyBody} rows="2" required /></label>
        <button class="btn btn-primary" type="submit">Post reply</button>
      </form>
    {/if}
  </div>
</div>

<style>
  .btn-sm { padding: 0.35rem 0.6rem; font-size: 0.85rem; }
</style>
