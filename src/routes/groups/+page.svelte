<script lang="ts">

  import { onMount } from "svelte";

  import { api } from "$lib/api";

  import type {

    Course,

    User,

    WhatsAppGroup,

    WhatsAppGroupMember,

    WhatsAppInboundMessage,

    WhatsAppInboundRoutingSettings,

    WhatsAppScheduledBroadcast,

    WhatsAppOutboundMessage,

    WhatsAppBroadcastSummary,

    WhatsAppGroupRosterDiff,

    WhatsAppJoinRequest,

    WhatsAppGroupParticipantEvent,

    WhatsAppGroupSettingsEvent,

  } from "$lib/types";



  let courses = $state<Course[]>([]);

  let users = $state<User[]>([]);

  let groups = $state<WhatsAppGroup[]>([]);

  let members = $state<WhatsAppGroupMember[]>([]);

  let messages = $state<WhatsAppOutboundMessage[]>([]);

  let broadcastSummaries = $state<WhatsAppBroadcastSummary[]>([]);

  let inboundMessages = $state<WhatsAppInboundMessage[]>([]);

  let inboundRouting = $state<WhatsAppInboundRoutingSettings | null>(null);

  let inboundCourseId = $state("");

  let inboundEnabled = $state(false);
  let scheduledBroadcasts = $state<WhatsAppScheduledBroadcast[]>([]);

  let selectedCourseId = $state("");

  let selectedGroupId = $state("");

  let groupName = $state("");

  let groupKind = $state("students");

  let addUserId = $state("");

  let inviteLink = $state("");

  let externalName = $state("");

  let joinApprovalMode = $state("auto_approve");

  let groupInviteConfigured = $state(false);

  let rosterDiff = $state<WhatsAppGroupRosterDiff | null>(null);

  let joinRequests = $state<WhatsAppJoinRequest[]>([]);

  let syncSendInvites = $state(true);

  let syncRemoveOrphans = $state(false);

  let participantEvents = $state<WhatsAppGroupParticipantEvent[]>([]);

  let settingsEvents = $state<WhatsAppGroupSettingsEvent[]>([]);

  let error = $state("");

  let message = $state("");

  let apiConfigured = $state(false);



  let selectedGroup = $derived(groups.find((g) => g.id === selectedGroupId) ?? null);



  async function loadMessages() {

    if (!selectedGroupId) {

      messages = [];

      broadcastSummaries = [];

      return;

    }

    messages = await api.listWhatsAppOutboundMessages(selectedGroupId, 30);

    broadcastSummaries = await api.listWhatsAppBroadcastSummaries(selectedGroupId, 15);

  }

  async function loadInbound() {
    inboundRouting = await api.getWhatsAppInboundRoutingSettings();
    inboundEnabled = inboundRouting.enabled;
    inboundCourseId = inboundRouting.course_id;
    inboundMessages = await api.listWhatsAppInboundMessages(undefined, 30);
    scheduledBroadcasts = await api.listWhatsAppScheduledBroadcasts(undefined, 30);
  }



  async function load() {

    error = "";

    message = "";

    try {

      const waSettings = await api.getWhatsAppBusinessSettings();

      apiConfigured = waSettings.configured;

      const templateSettings = await api.getWhatsAppTemplateSettings();

      groupInviteConfigured = templateSettings.group_invite_configured;

      courses = await api.listCourses();

      users = await api.listUsers();

      if (!selectedCourseId && courses.length > 0) {

        selectedCourseId = courses[0].id;

      }

      groups = await api.listWhatsAppGroups(selectedCourseId || undefined);

      if (selectedGroupId && !groups.some((g) => g.id === selectedGroupId)) {

        selectedGroupId = "";

        members = [];

        messages = [];

      }

      if (!selectedGroupId && groups.length > 0) {

        selectedGroupId = groups[0].id;

      }

      if (selectedGroupId) {

        members = await api.listWhatsAppGroupMembers(selectedGroupId);

        const link = await api.getWhatsAppGroupLink(selectedGroupId);

        inviteLink = link?.invite_link ?? selectedGroup?.invite_link ?? "";

        externalName = link?.external_name ?? selectedGroup?.external_name ?? "";

        await loadMessages();

        if (selectedGroup?.external_group_id) {

          await loadRosterDiff();

          await loadParticipantEvents();

          await loadSettingsEvents();

        } else {

          rosterDiff = null;

          participantEvents = [];

          settingsEvents = [];

        }

      }

      await loadInbound();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to load groups";

    }

  }



  async function createGroup(event: Event) {

    event.preventDefault();

    if (!selectedCourseId || !groupName.trim()) return;

    try {

      await api.createWhatsAppGroup({

        course_id: selectedCourseId,

        name: groupName.trim(),

        kind: groupKind,

      });

      groupName = "";

      message = "Group created.";

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to create group";

    }

  }



  async function deleteGroup() {

    if (!selectedGroupId) return;

    if (!confirm("Delete this contact group?")) return;

    try {

      await api.deleteWhatsAppGroup(selectedGroupId);

      selectedGroupId = "";

      members = [];

      messages = [];

      message = "Group deleted.";

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to delete group";

    }

  }



  async function syncMembers() {

    if (!selectedGroupId) return;

    try {

      const count = await api.syncWhatsAppGroupMembers(selectedGroupId);

      message = `Synced ${count} member(s).`;

      members = await api.listWhatsAppGroupMembers(selectedGroupId);

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to sync members";

    }

  }



  async function addMember() {

    if (!selectedGroupId || !addUserId) return;

    try {

      await api.addWhatsAppGroupMember(selectedGroupId, addUserId);

      addUserId = "";

      members = await api.listWhatsAppGroupMembers(selectedGroupId);

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to add member";

    }

  }



  async function removeMember(userId: string) {

    if (!selectedGroupId) return;

    try {

      await api.removeWhatsAppGroupMember(selectedGroupId, userId);

      members = await api.listWhatsAppGroupMembers(selectedGroupId);

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to remove member";

    }

  }



  async function toggleConsent(userId: string, optedIn: boolean) {

    try {

      await api.setWhatsAppConsent(userId, optedIn);

      members = await api.listWhatsAppGroupMembers(selectedGroupId);

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to update consent";

    }

  }



  async function saveGroupLink() {

    if (!selectedGroupId || !inviteLink.trim()) return;

    try {

      await api.linkWhatsAppGroup({

        group_id: selectedGroupId,

        invite_link: inviteLink.trim(),

        external_name: externalName.trim() || undefined,

      });

      message = "WhatsApp group link saved.";

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to link group";

    }

  }



  async function unlinkGroup() {

    if (!selectedGroupId) return;

    try {

      await api.unlinkWhatsAppGroup(selectedGroupId);

      inviteLink = "";

      externalName = "";

      message = "Group link removed.";

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to unlink group";

    }

  }



  async function createNativeGroup() {

    if (!selectedGroupId) return;

    try {

      const result = await api.createNativeWhatsAppGroup({

        group_id: selectedGroupId,

        join_approval_mode: joinApprovalMode,

      });

      message = result.message;

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to create native group";

    }

  }



  async function refreshNativeGroup() {

    if (!selectedGroupId) return;

    try {

      const result = await api.refreshNativeWhatsAppGroup(selectedGroupId);

      message = result.message;

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to refresh invite link";

    }

  }



  async function sendGroupInvites() {

    if (!selectedGroupId) return;

    try {

      const result = await api.sendWhatsAppGroupInvites({ group_id: selectedGroupId });

      message = `Invites sent: ${result.sent}, failed: ${result.failed}, skipped: ${result.skipped}.`;

      await loadMessages();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to send group invites";

    }

  }



  async function loadRosterDiff() {

    if (!selectedGroupId) {

      rosterDiff = null;

      return;

    }

    try {

      rosterDiff = await api.getWhatsAppGroupRosterDiff(selectedGroupId);

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to compare roster";

    }

  }



  async function reconcileRoster() {

    if (!selectedGroupId) return;

    try {

      await api.syncWhatsAppGroupMembers(selectedGroupId);

      message = "ClassMate roster synced from course.";

      members = await api.listWhatsAppGroupMembers(selectedGroupId);

      await loadRosterDiff();

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to sync roster";

    }

  }



  async function syncNativeRoster() {

    if (!selectedGroupId) return;

    try {

      const result = await api.syncNativeWhatsAppGroupRoster({

        group_id: selectedGroupId,

        send_invites: syncSendInvites,

        remove_orphans: syncRemoveOrphans,

      });

      rosterDiff = result.diff;

      message = result.message;

      await loadMessages();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to sync native roster";

    }

  }



  async function loadJoinRequests() {

    if (!selectedGroupId) return;

    try {

      joinRequests = await api.listWhatsAppGroupJoinRequests(selectedGroupId);

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to load join requests";

    }

  }



  async function approveJoinRequests() {

    if (!selectedGroupId || joinRequests.length === 0) return;

    try {

      await api.approveWhatsAppGroupJoinRequests({

        group_id: selectedGroupId,

        join_request_ids: joinRequests.map((r) => r.join_request_id),

      });

      message = `Approved ${joinRequests.length} join request(s).`;

      joinRequests = [];

      await loadRosterDiff();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to approve join requests";

    }

  }



  async function loadParticipantEvents() {

    if (!selectedGroupId) {

      participantEvents = [];

      return;

    }

    try {

      participantEvents = await api.listWhatsAppGroupParticipantEvents(selectedGroupId, 20);

    } catch {

      participantEvents = [];

    }

  }



  async function loadSettingsEvents() {

    if (!selectedGroupId) {

      settingsEvents = [];

      return;

    }

    try {

      settingsEvents = await api.listWhatsAppGroupSettingsEvents(selectedGroupId, 20);

    } catch {

      settingsEvents = [];

    }

  }



  async function rejectJoinRequests() {

    if (!selectedGroupId || joinRequests.length === 0) return;

    try {

      await api.rejectWhatsAppGroupJoinRequests({

        group_id: selectedGroupId,

        join_request_ids: joinRequests.map((r) => r.join_request_id),

      });

      message = `Rejected ${joinRequests.length} join request(s).`;

      joinRequests = [];

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to reject join requests";

    }

  }

  async function saveInboundRouting() {
    error = "";
    try {
      inboundRouting = await api.setWhatsAppInboundRoutingSettings({
        enabled: inboundEnabled,
        course_id: inboundCourseId,
      });
      message = "Inbound routing saved.";
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save inbound routing";
    }
  }

  async function routeInbound(inboundId: string) {
    try {
      await api.routeWhatsAppInboundMessage(inboundId);
      await loadInbound();
      message = "Message routed to forum.";
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to route message";
    }
  }

  async function ignoreInbound(inboundId: string) {
    try {
      await api.ignoreWhatsAppInboundMessage(inboundId);
      await loadInbound();
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to ignore message";
    }
  }

  async function cancelScheduled(id: string) {
    try {
      await api.cancelWhatsAppScheduledBroadcast(id);
      await loadInbound();
      message = "Scheduled broadcast cancelled.";
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to cancel schedule";
    }
  }

  async function runDueScheduled() {
    try {
      const result = await api.runDueWhatsAppScheduledBroadcasts();
      await loadInbound();
      if (selectedGroupId) await loadMessages();
      message = `Processed ${result.processed} scheduled broadcast(s).`;
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to run scheduled broadcasts";
    }
  }



  onMount(load);

</script>



<div class="page-header">

  <div>

    <h2>WhatsApp groups</h2>

    <p>

      Contact lists, consent, linked group chats, and Business API delivery log.

      {#if apiConfigured}

        <span class="badge" style="margin-left:.35rem">API connected</span>

      {/if}

    </p>

  </div>

</div>



{#if error}

  <p class="error">{error}</p>

{/if}

{#if message}

  <p style="color:#15803d;margin-bottom:1rem">{message}</p>

{/if}



<div class="grid two-col">

  <div class="card">

    <h3 style="margin-top:0">Create group</h3>

    <form class="form-grid" onsubmit={createGroup}>

      <label>

        Course

        <select bind:value={selectedCourseId} onchange={load}>

          {#each courses as course}

            <option value={course.id}>{course.code} — {course.title}</option>

          {/each}

        </select>

      </label>

      <label>

        Name

        <input bind:value={groupName} required placeholder="Period 3 students" />

      </label>

      <label>

        Kind

        <select bind:value={groupKind}>

          <option value="students">Students (sync from enrollments)</option>

          <option value="teachers">Teachers (course teacher)</option>

          <option value="custom">Custom (manual members)</option>

        </select>

      </label>

      <button class="btn btn-primary" type="submit">Create group</button>

    </form>

  </div>



  <div class="card">

    <h3 style="margin-top:0">Groups</h3>

    {#if groups.length === 0}

      <p class="empty">No groups for this course yet.</p>

    {:else}

      <label>

        Select group

        <select

          bind:value={selectedGroupId}

          onchange={async () => {

            if (selectedGroupId) {

              members = await api.listWhatsAppGroupMembers(selectedGroupId);

              await loadMessages();

            }

          }}

        >

          {#each groups as g}

            <option value={g.id}>{g.name} · {g.kind} · {g.member_count} members</option>

          {/each}

        </select>

      </label>

      {#if selectedGroup}

        <p style="color:var(--muted);font-size:.9rem;margin:.5rem 0">

          {selectedGroup.course_title} · {selectedGroup.kind}

        </p>

        <div style="display:flex;gap:.5rem;flex-wrap:wrap;margin-top:.75rem">

          {#if selectedGroup.kind !== "custom"}

            <button class="btn btn-secondary btn-sm" type="button" onclick={syncMembers}>

              Sync from course

            </button>

          {/if}

          <button class="btn btn-danger btn-sm" type="button" onclick={deleteGroup}>Delete</button>

        </div>

      {/if}

    {/if}

  </div>

</div>



{#if selectedGroupId}

  {#if apiConfigured}

    <div class="card" style="margin-top:1rem">

      <h3 style="margin-top:0">Native WhatsApp group</h3>

      <p style="color:var(--muted);font-size:.9rem;margin-bottom:.75rem">

        Create a group via Meta Groups API (Official Business Account required, max 8 members).

      </p>

      {#if selectedGroup?.native_status}

        <p style="font-size:.9rem">

          Status: <strong>{selectedGroup.native_status}</strong>

          {#if selectedGroup.external_group_id}

            · ID: {selectedGroup.external_group_id}

          {/if}

        </p>

        {#if selectedGroup.native_status === "suspended"}

          <p style="color:var(--danger);font-size:.9rem;margin-top:.35rem">

            This group is suspended by Meta. Messaging and roster changes may be blocked until the suspension clears.

          </p>

        {/if}

        {#if selectedGroup.group_description}

          <p style="font-size:.85rem;color:var(--muted);margin-top:.35rem">

            Description: {selectedGroup.group_description}

          </p>

        {/if}

      {/if}

      {#if selectedGroup?.creation_error}

        <p style="color:var(--danger);font-size:.9rem">{selectedGroup.creation_error}</p>

      {/if}

      {#if !selectedGroup?.external_group_id}

        <div class="form-grid two-col">

          <label>

            Join approval

            <select bind:value={joinApprovalMode}>

              <option value="auto_approve">Auto approve</option>

              <option value="approval_required">Approval required</option>

            </select>

          </label>

        </div>

        <button

          class="btn btn-primary btn-sm"

          type="button"

          style="margin-top:.75rem"

          onclick={createNativeGroup}

          disabled={!selectedGroup || selectedGroup.member_count > 8}

        >

          Create native group

        </button>

        {#if selectedGroup && selectedGroup.member_count > 8}

          <p style="color:var(--danger);font-size:.85rem;margin-top:.5rem">

            Native groups support at most 8 members. Remove members or sync a smaller group.

          </p>

        {/if}

      {:else}

        <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">

          {#if selectedGroup?.native_status === "pending"}

            <button class="btn btn-secondary btn-sm" type="button" onclick={refreshNativeGroup}>

              Refresh invite link

            </button>

          {/if}

          {#if groupInviteConfigured && selectedGroup?.native_status === "active"}

            <button class="btn btn-primary btn-sm" type="button" onclick={sendGroupInvites}>

              Send group invites

            </button>

          {/if}

          {#if inviteLink}

            <a class="btn btn-secondary btn-sm" href={inviteLink} target="_blank" rel="noopener noreferrer">

              Open group

            </a>

          {/if}

        </div>

        {#if !groupInviteConfigured}

          <p style="color:var(--muted);font-size:.85rem;margin-top:.5rem">

            Configure a group invite template in Settings to send invites to opted-in members.

          </p>

        {/if}

      {/if}

    </div>

    {#if selectedGroup?.external_group_id}

      <div class="card" style="margin-top:1rem">

        <h3 style="margin-top:0">Roster reconciliation</h3>

        <p style="color:var(--muted);font-size:.9rem;margin-bottom:.75rem">

          Compare ClassMate members with native WhatsApp participants.

        </p>

        <div style="display:flex;gap:.5rem;flex-wrap:wrap">

          <button class="btn btn-secondary btn-sm" type="button" onclick={loadRosterDiff}>Compare roster</button>

          {#if selectedGroup?.kind !== "custom"}

            <button class="btn btn-secondary btn-sm" type="button" onclick={reconcileRoster}>Sync ClassMate from course</button>

          {/if}

        </div>

        <div style="margin-top:.75rem;display:flex;flex-direction:column;gap:.35rem;font-size:.9rem">

          <label style="display:flex;align-items:center;gap:.5rem">

            <input type="checkbox" bind:checked={syncSendInvites} />

            Send group invites to members missing from WhatsApp

          </label>

          <label style="display:flex;align-items:center;gap:.5rem">

            <input type="checkbox" bind:checked={syncRemoveOrphans} />

            Remove WhatsApp participants not in ClassMate roster

          </label>

          <button

            class="btn btn-primary btn-sm"

            type="button"

            style="align-self:flex-start;margin-top:.25rem"

            onclick={syncNativeRoster}

            disabled={!groupInviteConfigured && syncSendInvites}

          >

            Sync native roster

          </button>

        </div>

        <div style="margin-top:1rem;padding-top:.75rem;border-top:1px solid var(--border)">

          <h4 style="margin:0 0 .5rem;font-size:.95rem">Join requests</h4>

          <div style="display:flex;gap:.5rem;flex-wrap:wrap">

            <button class="btn btn-secondary btn-sm" type="button" onclick={loadJoinRequests}>Load requests</button>

            {#if joinRequests.length > 0}

              <button class="btn btn-primary btn-sm" type="button" onclick={approveJoinRequests}>Approve all</button>

              <button class="btn btn-danger btn-sm" type="button" onclick={rejectJoinRequests}>Reject all</button>

            {/if}

          </div>

          {#if joinRequests.length > 0}

            <ul style="margin:.75rem 0 0;padding-left:1.25rem;font-size:.9rem">

              {#each joinRequests as req}

                <li>{req.wa_id} · {req.join_request_id.slice(0, 12)}…</li>

              {/each}

            </ul>

          {/if}

        </div>

        {#if rosterDiff}

          <p style="margin-top:.75rem;font-size:.9rem">{rosterDiff.message}</p>

          {#if rosterDiff.native_available}

            <p style="font-size:.85rem;color:var(--muted)">Matched: {rosterDiff.matched_count}</p>

          {/if}

          {#if rosterDiff.only_in_classmate.length > 0}

            <h4 style="margin:1rem 0 .35rem;font-size:.95rem">Only in ClassMate</h4>

            <ul style="margin:0;padding-left:1.25rem;font-size:.9rem">

              {#each rosterDiff.only_in_classmate as m}

                <li>{m.name}{m.phone ? ` (${m.phone})` : ""}</li>

              {/each}

            </ul>

          {/if}

          {#if rosterDiff.only_in_whatsapp.length > 0}

            <h4 style="margin:1rem 0 .35rem;font-size:.95rem">Only in WhatsApp</h4>

            <ul style="margin:0;padding-left:1.25rem;font-size:.9rem">

              {#each rosterDiff.only_in_whatsapp as p}

                <li>{p.wa_id}</li>

              {/each}

            </ul>

          {/if}

        {/if}

        {#if participantEvents.length > 0}

          <h4 style="margin:1rem 0 .35rem;font-size:.95rem">Recent participant activity</h4>

          <ul style="margin:0;padding-left:1.25rem;font-size:.85rem;color:var(--muted)">

            {#each participantEvents as ev}

              <li>

                {new Date(ev.received_at).toLocaleString()} · {ev.event_type}

                {#if ev.direction} ({ev.direction}){/if}

                {#if ev.wa_id} · {ev.wa_id}{/if}

              </li>

            {/each}

          </ul>

          <p style="font-size:.8rem;color:var(--muted);margin-top:.5rem">

            Roster cache refreshes automatically when Meta sends group_participants_update webhooks.

          </p>

        {/if}

        {#if settingsEvents.length > 0}

          <h4 style="margin:1rem 0 .35rem;font-size:.95rem">Recent settings & status</h4>

          <ul style="margin:0;padding-left:1.25rem;font-size:.85rem;color:var(--muted)">

            {#each settingsEvents as ev}

              <li>

                {new Date(ev.received_at).toLocaleString()} · {ev.event_type}

                {#if ev.setting_kind} · {ev.setting_kind}{/if}

                {#if ev.setting_value} · {ev.setting_value}{/if}

                {#if ev.update_successful === false}

                  · failed{#if ev.error_summary}: {ev.error_summary}{/if}

                {:else if ev.update_successful === true}

                  · ok

                {/if}

              </li>

            {/each}

          </ul>

          <p style="font-size:.8rem;color:var(--muted);margin-top:.5rem">

            Subscribe to group_settings_update and group_status_update webhooks in your Meta app.

          </p>

        {/if}

      </div>

    {/if}

  {/if}

  <div class="card" style="margin-top:1rem">

    <h3 style="margin-top:0">Linked WhatsApp group</h3>

    <p style="color:var(--muted);font-size:.9rem;margin-bottom:.75rem">

      Paste the invite link for your real WhatsApp group chat (manual fallback).

    </p>

    <div class="form-grid two-col">

      <label>

        Invite link

        <input bind:value={inviteLink} placeholder="https://chat.whatsapp.com/..." />

      </label>

      <label>

        Group name (optional)

        <input bind:value={externalName} placeholder="SCI-101 Students" />

      </label>

    </div>

    <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">

      <button class="btn btn-primary btn-sm" type="button" onclick={saveGroupLink}>Save link</button>

      {#if inviteLink}

        <a class="btn btn-secondary btn-sm" href={inviteLink} target="_blank" rel="noopener noreferrer">Open group</a>

        <button class="btn btn-secondary btn-sm" type="button" onclick={unlinkGroup}>Remove link</button>

      {/if}

    </div>

  </div>



  <div class="card" style="margin-top:1rem">

    <h3 style="margin-top:0">Members & consent</h3>

    {#if selectedGroup?.kind === "custom"}

      <div class="form-grid two-col" style="margin-bottom:1rem">

        <label>

          Add user

          <select bind:value={addUserId}>

            <option value="">Select user…</option>

            {#each users as u}

              <option value={u.id}>{u.name} ({u.role})</option>

            {/each}

          </select>

        </label>

        <div style="align-self:end">

          <button class="btn btn-secondary" type="button" onclick={addMember} disabled={!addUserId}>

            Add member

          </button>

        </div>

      </div>

    {/if}

    {#if members.length === 0}

      <p class="empty">No members yet. Sync from course or add users manually.</p>

    {:else}

      <table>

        <thead>

          <tr>

            <th>Name</th>

            <th>Role</th>

            <th>Phone</th>

            <th>WhatsApp opt-in</th>

            <th></th>

          </tr>

        </thead>

        <tbody>

          {#each members as m}

            <tr>

              <td>{m.name}</td>

              <td><span class="badge">{m.role}</span></td>

              <td>{m.phone || "—"}</td>

              <td>

                <label style="display:flex;align-items:center;gap:.35rem;font-weight:500">

                  <input

                    type="checkbox"

                    checked={m.opted_in ?? false}

                    onchange={(e) => toggleConsent(m.user_id, (e.target as HTMLInputElement).checked)}

                  />

                  Opted in

                </label>

              </td>

              <td>

                {#if selectedGroup?.kind === "custom"}

                  <button class="btn btn-secondary btn-sm" type="button" onclick={() => removeMember(m.user_id)}>

                    Remove

                  </button>

                {/if}

              </td>

            </tr>

          {/each}

        </tbody>

      </table>

    {/if}

  </div>



  <div class="card" style="margin-top:1rem">

    <h3 style="margin-top:0">Broadcast delivery summary</h3>

    <p style="color:var(--muted);font-size:.9rem;margin-bottom:.75rem">
      Aggregated sent, delivered, and read counts from WhatsApp status webhooks per broadcast batch.
    </p>

    {#if !apiConfigured}

      <p class="empty">Configure WhatsApp Business API in Settings to track delivery.</p>

    {:else if broadcastSummaries.length === 0}

      <p class="empty">No broadcasts yet for this group.</p>

    {:else}

      <div class="table-wrap">

        <table>

          <thead>

            <tr>

              <th>When</th>

              <th>Kind</th>

              <th>Total</th>

              <th>Sent</th>

              <th>Delivered</th>

              <th>Read</th>

              <th>Failed</th>

            </tr>

          </thead>

          <tbody>

            {#each broadcastSummaries as batch}

              <tr>

                <td>{new Date(batch.started_at).toLocaleString()}</td>

                <td><span class="badge">{batch.kind}</span></td>

                <td>{batch.total}</td>

                <td>{batch.sent + batch.pending}</td>

                <td>{batch.delivered}</td>

                <td>{batch.read}</td>

                <td>{batch.failed}</td>

              </tr>

            {/each}

          </tbody>

        </table>

      </div>

    {/if}

  </div>



  <div class="card" style="margin-top:1rem">

    <h3 style="margin-top:0">Delivery log</h3>

    {#if !apiConfigured}

      <p class="empty">Configure WhatsApp Business API in Settings to send and track messages.</p>

    {:else if messages.length === 0}

      <p class="empty">No outbound messages yet. Use Share → Send via Business API on assignments or announcements.</p>

    {:else}

      <div class="table-wrap">

        <table>

          <thead>

            <tr>

              <th>When</th>

              <th>Kind</th>

              <th>Recipient</th>

              <th>Status</th>

              <th>Delivered</th>

              <th>Read</th>

            </tr>

          </thead>

          <tbody>

            {#each messages as msg}

              <tr>

                <td>{new Date(msg.created_at).toLocaleString()}</td>

                <td><span class="badge">{msg.kind}</span></td>

                <td>{msg.user_name || msg.phone}</td>

                <td><span class="badge">{msg.status}</span></td>

                <td>{msg.delivered_at ? new Date(msg.delivered_at).toLocaleString() : "—"}</td>

                <td>{msg.read_at ? new Date(msg.read_at).toLocaleString() : "—"}</td>

              </tr>

            {/each}

          </tbody>

        </table>

      </div>

    {/if}

  </div>

{/if}



<div class="card" style="margin-top:1rem">

  <h3 style="margin-top:0">Inbound replies</h3>

  <p style="color:var(--muted);font-size:.9rem;margin-bottom:.75rem">
    Route student WhatsApp replies into a course forum thread. Requires LAN sync webhook and Meta message subscription.
    Users can reply STOP to opt out when auto-unsubscribe is enabled in Settings.
  </p>

  <label style="display:flex;align-items:center;gap:.5rem;margin-bottom:.75rem">
    <input type="checkbox" bind:checked={inboundEnabled} />
    Auto-route to forum when messages arrive
  </label>

  <label style="display:block;margin-bottom:.75rem">
    Route to course
    <select bind:value={inboundCourseId} style="display:block;margin-top:.25rem;width:100%">
      <option value="">Select course…</option>
      {#each courses as course}
        <option value={course.id}>{course.code} — {course.title}</option>
      {/each}
    </select>
  </label>

  {#if inboundRouting?.topic_title}
    <p style="font-size:.9rem;color:var(--muted)">Forum topic: {inboundRouting.topic_title}</p>
  {/if}

  {#if inboundRouting && inboundRouting.pending_count > 0}
    <p style="font-size:.9rem">{inboundRouting.pending_count} pending message(s)</p>
  {/if}

  <button class="btn btn-primary btn-sm" type="button" onclick={saveInboundRouting}>Save routing</button>

  {#if inboundMessages.length === 0}
    <p class="empty" style="margin-top:1rem">No inbound messages yet.</p>
  {:else}
    <div class="table-wrap" style="margin-top:1rem">
      <table>
        <thead>
          <tr>
            <th>When</th>
            <th>From</th>
            <th>Message</th>
            <th>Status</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each inboundMessages as msg}
            <tr>
              <td>{new Date(msg.received_at).toLocaleString()}</td>
              <td>{msg.from_user_name || msg.from_phone}</td>
              <td style="max-width:240px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{msg.body}</td>
              <td><span class="badge">{msg.status}</span></td>
              <td>
                {#if msg.status === "pending"}
                  <button class="btn btn-secondary btn-sm" type="button" onclick={() => routeInbound(msg.id)}>Route</button>
                  <button class="btn btn-secondary btn-sm" type="button" onclick={() => ignoreInbound(msg.id)}>Ignore</button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

</div>



<div class="card" style="margin-top:1rem">

  <h3 style="margin-top:0">Scheduled broadcasts</h3>

  <p style="color:var(--muted);font-size:.9rem;margin-bottom:.75rem">
    Queued WhatsApp sends run when the app starts or when you click run due now.
  </p>

  <button class="btn btn-secondary btn-sm" type="button" onclick={runDueScheduled}>Run due now</button>

  {#if scheduledBroadcasts.length === 0}
    <p class="empty" style="margin-top:1rem">No scheduled broadcasts. Schedule from assignment or announcement share dialogs.</p>
  {:else}
    <div class="table-wrap" style="margin-top:1rem">
      <table>
        <thead>
          <tr>
            <th>When</th>
            <th>Group</th>
            <th>Type</th>
            <th>Status</th>
            <th>Result</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each scheduledBroadcasts as item}
            <tr>
              <td>{new Date(item.scheduled_at).toLocaleString()}</td>
              <td>{item.group_name || item.group_id}</td>
              <td>{item.broadcast_kind}{item.kind ? ` · ${item.kind}` : ""}</td>
              <td><span class="badge">{item.status}</span></td>
              <td>
                {#if item.status === "sent"}
                  {item.result_sent ?? 0} sent, {item.result_failed ?? 0} failed
                {:else if item.error}
                  {item.error}
                {:else}
                  —
                {/if}
              </td>
              <td>
                {#if item.status === "pending"}
                  <button class="btn btn-secondary btn-sm" type="button" onclick={() => cancelScheduled(item.id)}>Cancel</button>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

</div>


