<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { Course, ScheduleSlot } from "$lib/types";

  const days = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];

  let courses = $state<Course[]>([]);
  let slots = $state<ScheduleSlot[]>([]);
  let courseId = $state("");
  let dayOfWeek = $state(1);
  let startTime = $state("09:00");
  let endTime = $state("10:00");
  let room = $state("");
  let title = $state("");
  let error = $state("");

  async function load() {
    error = "";
    courses = await api.listCourses();
    if (!courseId && courses.length > 0) courseId = courses[0].id;
    slots = await api.listSchedule(courseId || undefined);
  }

  async function addSlot(event: Event) {
    event.preventDefault();
    if (!courseId) return;
    try {
      await api.createScheduleSlot({
        course_id: courseId,
        day_of_week: dayOfWeek,
        start_time: startTime,
        end_time: endTime,
        room: room || undefined,
        title: title || undefined,
      });
      room = "";
      title = "";
      await load();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to add slot";
    }
  }

  async function removeSlot(id: string) {
    await api.deleteScheduleSlot(id);
    await load();
  }

  function slotsForDay(day: number) {
    return slots.filter((s) => s.day_of_week === day);
  }

  onMount(load);
</script>

<div class="page-header">
  <div>
    <h2>Schedule</h2>
    <p>Weekly timetable for courses — rooms, periods, and session titles.</p>
  </div>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

<div class="grid two-col" style="margin-bottom: 1rem">
  <div class="card">
    <h3 style="margin-top: 0">Filter</h3>
    <label>
      Course
      <select
        bind:value={courseId}
        onchange={load}
      >
        <option value="">All courses</option>
        {#each courses as c}
          <option value={c.id}>{c.code} — {c.title}</option>
        {/each}
      </select>
    </label>
  </div>

  <div class="card">
    <h3 style="margin-top: 0">Add slot</h3>
    <form class="form-grid" onsubmit={addSlot}>
      <label>
        Course
        <select bind:value={courseId} required>
          {#each courses as c}
            <option value={c.id}>{c.code} — {c.title}</option>
          {/each}
        </select>
      </label>
      <label>
        Day
        <select bind:value={dayOfWeek}>
          {#each days as day, i}
            <option value={i}>{day}</option>
          {/each}
        </select>
      </label>
      <label>Start<input type="time" bind:value={startTime} required /></label>
      <label>End<input type="time" bind:value={endTime} required /></label>
      <label>Room<input bind:value={room} placeholder="Lab 101" /></label>
      <label>Title<input bind:value={title} placeholder="Morning lab" /></label>
      <button class="btn btn-primary" type="submit">Add to schedule</button>
    </form>
  </div>
</div>

<div class="card schedule-grid">
  {#each days as day, i}
    <div class="day-col">
      <h3>{day}</h3>
      {#each slotsForDay(i) as slot}
        <div class="slot">
          <strong>{slot.course_code}</strong>
          <div>{slot.start_time} – {slot.end_time}</div>
          <div style="color: var(--muted); font-size: 0.9rem">
            {slot.title || slot.course_title}
            {#if slot.room} · {slot.room}{/if}
          </div>
          <button class="btn btn-danger btn-sm" onclick={() => removeSlot(slot.id)}>Remove</button>
        </div>
      {:else}
        <p class="empty" style="font-size: 0.9rem">No sessions</p>
      {/each}
    </div>
  {/each}
</div>

<style>
  .schedule-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 1rem;
  }
  .day-col h3 {
    margin: 0 0 0.75rem;
    font-size: 0.95rem;
  }
  .slot {
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 0.65rem;
    margin-bottom: 0.5rem;
    background: #f8fafc;
  }
  .btn-sm {
    margin-top: 0.5rem;
    padding: 0.25rem 0.5rem;
    font-size: 0.8rem;
  }
</style>
