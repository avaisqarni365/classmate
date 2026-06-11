<script lang="ts">
  import { api } from "$lib/api";
  import type { WhatsAppGroup, WhatsAppSharePlan, WhatsAppTemplatePreview } from "$lib/types";
  import { copyText, openWhatsApp } from "$lib/whatsapp";

  interface Props {
    kind: "assignment" | "announcement" | "hub" | "task";
    assignmentId?: string;
    announcementId?: string;
    courseId?: string;
    label?: string;
  }

  let {
    kind,
    assignmentId,
    announcementId,
    courseId,
    label = "WhatsApp",
  }: Props = $props();

  let open = $state(false);
  let loading = $state(false);
  let error = $state("");
  let plan = $state<WhatsAppSharePlan | null>(null);
  let groups = $state<WhatsAppGroup[]>([]);
  let groupId = $state("");
  let copied = $state(false);
  let apiConfigured = $state(false);
  let templateConfigured = $state(false);
  let templatePreview = $state<WhatsAppTemplatePreview | null>(null);
  let broadcastResult = $state("");
  let sending = $state(false);
  let sendingTemplate = $state(false);
  let scheduling = $state(false);
  let scheduleAt = $state("");

  function defaultScheduleAt() {
    const d = new Date(Date.now() + 60 * 60 * 1000);
    d.setMinutes(d.getMinutes() - d.getTimezoneOffset());
    return d.toISOString().slice(0, 16);
  }

  const canSendTemplate = $derived(
    (kind === "assignment" || kind === "task") &&
      !!assignmentId &&
      apiConfigured &&
      templateConfigured &&
      !!groupId,
  );

  async function openShare() {
    open = true;
    loading = true;
    error = "";
    plan = null;
    copied = false;
    broadcastResult = "";
    templatePreview = null;
    scheduleAt = defaultScheduleAt();
    try {
      const settings = await api.getWhatsAppBusinessSettings();
      apiConfigured = settings.configured;
      const templateSettings = await api.getWhatsAppTemplateSettings();
      templateConfigured = templateSettings.assignment_configured;
      groups = await api.listWhatsAppGroups(courseId || undefined);
      if (!groupId && groups.length > 0) {
        groupId = groups[0].id;
      }
      await loadPlan();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to prepare share";
    } finally {
      loading = false;
    }
  }

  async function loadPlan() {
    loading = true;
    error = "";
    templatePreview = null;
    try {
      plan = await api.buildWhatsAppShare({
        kind,
        assignment_id: assignmentId,
        announcement_id: announcementId,
        group_id: groupId || undefined,
      });
      if (templateConfigured && assignmentId) {
        templatePreview = await api.previewWhatsAppAssignmentTemplate(assignmentId);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to build message";
    } finally {
      loading = false;
    }
  }

  async function copyGroupMessage() {
    if (!plan) return;
    await copyText(plan.group_paste);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function close() {
    open = false;
    plan = null;
    error = "";
    broadcastResult = "";
    templatePreview = null;
  }

  async function sendViaApi() {
    if (!groupId || !apiConfigured) return;
    sending = true;
    error = "";
    broadcastResult = "";
    try {
      const result = await api.sendWhatsAppBroadcast({
        kind,
        group_id: groupId,
        assignment_id: assignmentId,
        announcement_id: announcementId,
      });
      broadcastResult = `Sent ${result.sent}, failed ${result.failed}, skipped ${result.skipped}.`;
      if (result.errors.length > 0) {
        broadcastResult += ` ${result.errors.slice(0, 2).join("; ")}`;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Broadcast failed";
    } finally {
      sending = false;
    }
  }

  async function sendTemplateViaApi() {
    if (!canSendTemplate || !assignmentId) return;
    sendingTemplate = true;
    error = "";
    broadcastResult = "";
    try {
      const result = await api.sendWhatsAppTemplateBroadcast({
        group_id: groupId,
        assignment_id: assignmentId,
      });
      broadcastResult = `Template sent ${result.sent}, failed ${result.failed}, skipped ${result.skipped}.`;
      if (result.errors.length > 0) {
        broadcastResult += ` ${result.errors.slice(0, 2).join("; ")}`;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Template broadcast failed";
    } finally {
      sendingTemplate = false;
    }
  }

  async function scheduleBroadcast(useTemplate: boolean) {
    if (!groupId || !scheduleAt || !apiConfigured) return;
    if (useTemplate && !canSendTemplate) return;
    scheduling = true;
    error = "";
    broadcastResult = "";
    try {
      await api.createWhatsAppScheduledBroadcast({
        broadcast_kind: useTemplate ? "template" : "text",
        group_id: groupId,
        scheduled_at: new Date(scheduleAt).toISOString(),
        kind: useTemplate ? undefined : kind,
        assignment_id: assignmentId,
        announcement_id: announcementId,
      });
      broadcastResult = `Scheduled for ${new Date(scheduleAt).toLocaleString()}. View on Groups page.`;
    } catch (e) {
      error = e instanceof Error ? e.message : "Schedule failed";
    } finally {
      scheduling = false;
    }
  }
</script>

<button class="btn btn-secondary btn-sm" type="button" onclick={openShare}>
  {label}
</button>

{#if open}
  <div class="wa-overlay" role="dialog" aria-modal="true">
    <div class="wa-modal card">
      <div class="wa-header">
        <h3 style="margin:0">Share via WhatsApp</h3>
        <button class="btn btn-secondary btn-sm" type="button" onclick={close}>Close</button>
      </div>

      {#if groups.length > 0}
        <label style="margin-top:1rem;display:block">
          Contact group
          <select bind:value={groupId} onchange={loadPlan}>
            {#each groups as g}
              <option value={g.id}>{g.course_title} — {g.name} ({g.member_count})</option>
            {/each}
          </select>
        </label>
      {:else}
        <p class="empty" style="margin-top:1rem">
          No contact groups yet. Create student or teacher groups on the Groups page.
        </p>
      {/if}

      {#if error}
        <p class="error">{error}</p>
      {/if}

      {#if loading}
        <p class="empty">Preparing message…</p>
      {:else if plan}
        <label style="display:block;margin-top:1rem">
          Message preview
          <textarea readonly rows="6" value={plan.message}></textarea>
        </label>

        <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-top:.75rem">
          <button class="btn btn-primary btn-sm" type="button" onclick={copyGroupMessage}>
            {copied ? "Copied!" : "Copy for group chat"}
          </button>
          {#if apiConfigured && groupId}
            <button class="btn btn-primary btn-sm" type="button" onclick={sendViaApi} disabled={sending}>
              {sending ? "Sending…" : "Send via Business API"}
            </button>
          {/if}
          {#if canSendTemplate}
            <button
              class="btn btn-secondary btn-sm"
              type="button"
              onclick={sendTemplateViaApi}
              disabled={sendingTemplate}
            >
              {sendingTemplate ? "Sending…" : "Send template reminder"}
            </button>
          {/if}
        </div>

        {#if apiConfigured && groupId}
          <div style="margin-top:1rem;padding-top:.75rem;border-top:1px solid var(--border)">
            <label style="display:block">
              Schedule broadcast
              <input type="datetime-local" bind:value={scheduleAt} style="display:block;margin-top:.25rem;width:100%" />
            </label>
            <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-top:.5rem">
              <button class="btn btn-secondary btn-sm" type="button" onclick={() => scheduleBroadcast(false)} disabled={scheduling}>
                {scheduling ? "Scheduling…" : "Schedule text send"}
              </button>
              {#if canSendTemplate}
                <button class="btn btn-secondary btn-sm" type="button" onclick={() => scheduleBroadcast(true)} disabled={scheduling}>
                  Schedule template send
                </button>
              {/if}
            </div>
          </div>
        {/if}

        {#if templatePreview}
          <p style="color:var(--muted);font-size:.85rem;margin-top:.75rem">
            Template <span class="mono">{templatePreview.template_name}</span> ({templatePreview.language})
          </p>
          <ul class="plain-list" style="font-size:.85rem;color:var(--muted)">
            {#each templatePreview.parameters as param, i}
              <li>{templatePreview.parameter_labels[i] ?? `Param ${i + 1}`}: {param}</li>
            {/each}
          </ul>
        {/if}

        {#if broadcastResult}
          <p style="color:#15803d;font-size:.9rem;margin-top:.5rem">{broadcastResult}</p>
        {/if}

        {#if plan.missing_phones.length > 0}
          <p class="error" style="font-size:.9rem;margin-top:.75rem">
            Missing phone: {plan.missing_phones.join(", ")} — add numbers on Users.
          </p>
        {/if}

        {#if plan.recipients.length > 0}
          <h4 style="margin:1rem 0 .5rem">Send to members</h4>
          <ul class="plain-list">
            {#each plan.recipients as r}
              <li style="display:flex;justify-content:space-between;align-items:center;gap:.5rem;margin-bottom:.35rem">
                <span>{r.name}{r.phone ? ` (${r.phone})` : ""}</span>
                {#if r.wa_url}
                  <button class="btn btn-secondary btn-sm" type="button" onclick={() => openWhatsApp(r.wa_url!)}>
                    Open
                  </button>
                {:else}
                  <span style="color:var(--muted);font-size:.85rem">No phone</span>
                {/if}
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </div>
  </div>
{/if}

<style>
  .wa-overlay {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    z-index: 1000;
  }
  .wa-modal {
    width: min(520px, 100%);
    max-height: 90vh;
    overflow: auto;
  }
  .wa-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 0.5rem;
  }
</style>
