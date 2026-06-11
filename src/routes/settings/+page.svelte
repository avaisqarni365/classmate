<script lang="ts">

  import { onMount } from "svelte";

  import { api } from "$lib/api";

  import { preferences } from "$lib/stores/preferences";

  import { t } from "$lib/i18n";

  import type {

    AutoBackupEntry,

    AutoBackupStatus,

    BackupPayload,

    DigestEmailStatus,

    ImportResult,

    LtiTool,

    SyncServerStatus,

    UiPreferences,

    TenancyContext,

    SchoolMember,

    PushReminderStatus,

    PushLogEntry,

    VideoStatus,

  } from "$lib/types";



  let video = $state<VideoStatus | null>(null);

  let sync = $state<SyncServerStatus | null>(null);

  let autoBackup = $state<AutoBackupStatus | null>(null);

  let backupFiles = $state<AutoBackupEntry[]>([]);

  let peerUrl = $state("");

  let syncTokenEdit = $state("");

  let publicBaseUrl = $state("");

  let publicHubPath = $state("/hub");

  let ltiTools = $state<LtiTool[]>([]);

  let externalVideoUrl = $state("");

  let plugnmeetBaseUrl = $state("");

  let plugnmeetRoom = $state("classmate");
  let cloudBackupUrl = $state("");
  let cloudBackupToken = $state("");
  let whatsappCountryCode = $state("1");
  let waApiVersion = $state("v21.0");
  let waPhoneNumberId = $state("");
  let waAccessToken = $state("");
  let waWebhookToken = $state("");
  let waBusiness = $state<{ configured: boolean; webhook_url?: string | null } | null>(null);
  let waTemplateName = $state("");
  let waTemplateLanguage = $state("en");
  let waGroupInviteTemplateName = $state("");
  let waGroupInviteTemplateLanguage = $state("en");
  let waAutoUnsubscribe = $state(true);
  let waUnsubscribeKeywords = $state("STOP,UNSUBSCRIBE,CANCEL,OPT OUT");
  let waTestResult = $state("");
  let smtpHost = $state("");
  let smtpPort = $state(587);
  let smtpUsername = $state("");
  let smtpFrom = $state("");
  let smtpPassword = $state("");
  let smtpUseTls = $state(true);
  let smtpConfigured = $state(false);
  let smtpTestTo = $state("");
  let smtpTestResult = $state("");
  let emailLog = $state<{ recipient: string; subject: string; status: string; created_at: string }[]>([]);
  let digestStatus = $state<DigestEmailStatus | null>(null);
  let digestEnabled = $state(false);
  let digestInterval = $state("weekly");

  let pushEnabled = $state(false);

  let fcmProjectId = $state("");

  let fcmServiceAccountJson = $state("");

  let apnsKeyId = $state("");

  let apnsTeamId = $state("");

  let apnsBundleId = $state("");

  let apnsPrivateKey = $state("");

  let apnsUseSandbox = $state(false);

  let pushTestPlatform = $state("fcm");

  let pushTestToken = $state("");

  let pushTestResult = $state("");

  let pushReminderStatus = $state<PushReminderStatus | null>(null);

  let pushRemindersEnabled = $state(false);

  let pushLog = $state<PushLogEntry[]>([]);

  let tenancyContext = $state<TenancyContext | null>(null);

  let schoolMembers = $state<SchoolMember[]>([]);

  let newSchoolName = $state("");

  let newSchoolCode = $state("");

  let editSchoolName = $state("");

  let editSchoolCode = $state("");

  let memberEmail = $state("");

  let ui = $state<UiPreferences>({

    school_name: "ClassMate",

    theme: "default",

    font_scale: "100",

    accent_color: "#2563eb",

    locale: "en",

  });

  let autoEnabled = $state(false);

  let autoInterval = $state("daily");

  let autoMaxKeep = $state(7);

  let csvText = $state("");

  let importResult = $state<ImportResult | null>(null);

  let ltiName = $state("");

  let ltiUrl = $state("");

  let ltiKey = $state("");

  let ltiSecret = $state("");

  let message = $state("");

  let error = $state("");



  async function load() {

    video = await api.getVideoStatus();

    ltiTools = await api.listLtiTools();

    externalVideoUrl = (await api.getSetting("external_video_url")) ?? "";

    plugnmeetBaseUrl = (await api.getSetting("plugnmeet_base_url")) ?? "";

    plugnmeetRoom = (await api.getSetting("plugnmeet_room")) ?? "classmate";
    cloudBackupUrl = (await api.getSetting("cloud_backup_url")) ?? "";
    cloudBackupToken = (await api.getSetting("cloud_backup_token")) ?? "";
    whatsappCountryCode = (await api.getSetting("whatsapp_country_code")) ?? "1";
    waBusiness = await api.getWhatsAppBusinessSettings();
    waApiVersion = waBusiness.api_version;
    waPhoneNumberId = waBusiness.phone_number_id;
    waWebhookToken = waBusiness.webhook_verify_token_set ? "••••••••" : "";
    const waTemplate = await api.getWhatsAppTemplateSettings();
    waTemplateName = waTemplate.assignment_name;
    waTemplateLanguage = waTemplate.assignment_language || "en";
    waGroupInviteTemplateName = waTemplate.group_invite_name;
    waGroupInviteTemplateLanguage = waTemplate.group_invite_language || "en";
    const waCompliance = await api.getWhatsAppComplianceSettings();
    waAutoUnsubscribe = waCompliance.auto_unsubscribe;
    waUnsubscribeKeywords = waCompliance.unsubscribe_keywords;
    const smtp = await api.getSmtpSettings();
    smtpConfigured = smtp.configured;
    smtpHost = smtp.host;
    smtpPort = smtp.port;
    smtpUsername = smtp.username;
    smtpFrom = smtp.from;
    smtpUseTls = smtp.use_tls;
    emailLog = await api.listEmailLog(10);
    digestStatus = await api.getDigestEmailStatus();
    digestEnabled = digestStatus.settings.enabled;
    digestInterval = digestStatus.settings.interval;

    const push = await api.getPushSettings();

    pushEnabled = push.enabled;

    fcmProjectId = push.fcm_project_id;

    apnsKeyId = push.apns_key_id;

    apnsTeamId = push.apns_team_id;

    apnsBundleId = push.apns_bundle_id;

    apnsUseSandbox = push.apns_use_sandbox;

    pushReminderStatus = await api.getPushReminderStatus();

    pushRemindersEnabled = pushReminderStatus.enabled;

    pushLog = await api.listPushLog(10);

    ui = await api.getUiPreferences();

    sync = await api.getSyncStatus();

    syncTokenEdit = sync.sync_token;

    publicBaseUrl = (await api.getSetting("public_base_url")) ?? "";

    publicHubPath = (await api.getSetting("public_hub_path")) ?? "/hub";

    autoBackup = await api.getAutoBackupStatus();

    autoEnabled = autoBackup.settings.enabled;

    autoInterval = autoBackup.settings.interval;

    autoMaxKeep = autoBackup.settings.max_keep;

    backupFiles = await api.listAutoBackups();

    tenancyContext = await api.getTenancyContext();

    if (tenancyContext) {

      editSchoolName = tenancyContext.active_school_name;

      const active = tenancyContext.schools.find((s) => s.id === tenancyContext!.active_school_id);

      editSchoolCode = active?.code ?? "";

      if (tenancyContext.is_org_admin) {

        schoolMembers = await api.listSchoolMembers(tenancyContext.active_school_id);

      } else {

        schoolMembers = [];

      }

    }

  }

  async function reloadSchoolAdmin() {

    tenancyContext = await api.getTenancyContext();

    if (tenancyContext?.is_org_admin) {

      schoolMembers = await api.listSchoolMembers(tenancyContext.active_school_id);

      editSchoolName = tenancyContext.active_school_name;

      const active = tenancyContext.schools.find((s) => s.id === tenancyContext!.active_school_id);

      editSchoolCode = active?.code ?? "";

    }

  }

  async function createSchool() {

    error = "";

    message = "";

    try {

      await api.createSchool({ name: newSchoolName.trim(), code: newSchoolCode.trim() });

      newSchoolName = "";

      newSchoolCode = "";

      message = t("settings.schools.created");

      await reloadSchoolAdmin();

      const { tenancy } = await import("$lib/stores/tenancy");

      await tenancy.init();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to create school";

    }

  }

  async function updateActiveSchool() {

    error = "";

    message = "";

    if (!tenancyContext) return;

    try {

      await api.updateSchool({

        id: tenancyContext.active_school_id,

        name: editSchoolName.trim(),

        code: editSchoolCode.trim(),

      });

      message = t("settings.schools.updated");

      await reloadSchoolAdmin();

      const { tenancy } = await import("$lib/stores/tenancy");

      await tenancy.init();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to update school";

    }

  }

  async function addSchoolMember() {

    error = "";

    message = "";

    if (!tenancyContext) return;

    try {

      await api.addSchoolMember({

        school_id: tenancyContext.active_school_id,

        user_email: memberEmail.trim(),

      });

      memberEmail = "";

      message = t("settings.schools.memberAdded");

      schoolMembers = await api.listSchoolMembers(tenancyContext.active_school_id);

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to add member";

    }

  }

  async function removeSchoolMember(membershipId: string) {

    error = "";

    message = "";

    try {

      await api.removeSchoolMember(membershipId);

      message = t("settings.schools.memberRemoved");

      await reloadSchoolAdmin();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to remove member";

    }

  }

  async function savePushSettings() {

    error = "";

    message = "";

    try {

      await api.savePushSettings({

        enabled: pushEnabled,

        fcm_project_id: fcmProjectId.trim(),

        fcm_service_account_json: fcmServiceAccountJson.trim() || undefined,

        apns_key_id: apnsKeyId.trim(),

        apns_team_id: apnsTeamId.trim(),

        apns_bundle_id: apnsBundleId.trim(),

        apns_private_key: apnsPrivateKey.trim() || undefined,

        apns_use_sandbox: apnsUseSandbox,

      });

      message = t("settings.push.saved");

      pushReminderStatus = await api.getPushReminderStatus();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to save push settings";

    }

  }

  async function testPushSettings() {

    error = "";

    pushTestResult = "";

    try {

      const result = await api.testPushNotification({

        platform: pushTestPlatform,

        token: pushTestToken.trim(),

        title: "ClassMate test",

        body: "Push notifications are working.",

      });

      pushTestResult = result;

      pushLog = await api.listPushLog(10);

    } catch (e) {

      error = e instanceof Error ? e.message : "Push test failed";

    }

  }

  async function savePushReminders() {

    error = "";

    message = "";

    try {

      await api.setPushReminderSettings(pushRemindersEnabled);

      message = t("settings.push.remindersSaved");

      pushReminderStatus = await api.getPushReminderStatus();

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to save reminder settings";

    }

  }

  async function runPushRemindersNow() {

    error = "";

    message = "";

    try {

      const result = await api.runPushRemindersNow();

      message = t("settings.push.remindersRun")

        .replace("{sent}", String(result.sent))

        .replace("{failed}", String(result.failed));

      pushReminderStatus = await api.getPushReminderStatus();

      pushLog = await api.listPushLog(10);

    } catch (e) {

      error = e instanceof Error ? e.message : "Reminder run failed";

    }

  }



  async function saveVideoSettings() {

    await api.setSetting("external_video_url", externalVideoUrl);

    await api.setSetting("plugnmeet_base_url", plugnmeetBaseUrl);

    await api.setSetting("plugnmeet_room", plugnmeetRoom);

    await api.setSetting("cloud_backup_url", cloudBackupUrl);

    await api.setSetting("cloud_backup_token", cloudBackupToken);

    await api.setSetting("whatsapp_country_code", whatsappCountryCode.trim() || "1");

    message = t("settings.saved");

  }

  async function saveWhatsAppBusiness() {
    error = "";
    message = "";
    try {
      waBusiness = await api.saveWhatsAppBusinessSettings({
        api_version: waApiVersion,
        phone_number_id: waPhoneNumberId,
        access_token: waAccessToken.trim() || undefined,
        webhook_verify_token: waWebhookToken.startsWith("•") ? undefined : waWebhookToken.trim() || undefined,
      });
      waAccessToken = "";
      if (waBusiness.webhook_verify_token_set) waWebhookToken = "••••••••";
      message = t("settings.saved");
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save WhatsApp API settings";
    }
  }

  async function testWhatsAppBusiness() {
    waTestResult = "";
    try {
      const result = await api.testWhatsAppBusinessConnection();
      waTestResult = result.ok
        ? `${result.message}${result.display_phone_number ? ` (${result.display_phone_number})` : ""}`
        : result.message;
    } catch (e) {
      waTestResult = e instanceof Error ? e.message : "Connection test failed";
    }
  }

  async function saveWhatsAppTemplateSettings() {
    error = "";
    message = "";
    try {
      await api.saveWhatsAppTemplateSettings({
        assignment_name: waTemplateName.trim(),
        assignment_language: waTemplateLanguage.trim() || "en",
        group_invite_name: waGroupInviteTemplateName.trim() || undefined,
        group_invite_language: waGroupInviteTemplateLanguage.trim() || "en",
      });
      message = t("settings.whatsappBusiness.templateSaved");
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save template settings";
    }
  }

  async function saveWhatsAppComplianceSettings() {
    error = "";
    message = "";
    try {
      await api.saveWhatsAppComplianceSettings({
        auto_unsubscribe: waAutoUnsubscribe,
        unsubscribe_keywords: waUnsubscribeKeywords.trim(),
      });
      message = t("settings.whatsappBusiness.complianceSaved");
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save compliance settings";
    }
  }

  async function saveSmtpSettings() {
    error = "";
    message = "";
    try {
      const smtp = await api.saveSmtpSettings({
        host: smtpHost,
        port: smtpPort,
        username: smtpUsername,
        from: smtpFrom,
        use_tls: smtpUseTls,
        password: smtpPassword.trim() || undefined,
      });
      smtpConfigured = smtp.configured;
      smtpPassword = "";
      message = t("settings.saved");
      emailLog = await api.listEmailLog(10);
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save SMTP settings";
    }
  }

  async function testSmtpSettings() {
    smtpTestResult = "";
    if (!smtpTestTo.trim()) return;
    try {
      const result = await api.testSmtpConnection(smtpTestTo.trim());
      smtpTestResult = result.message;
      emailLog = await api.listEmailLog(10);
    } catch (e) {
      smtpTestResult = e instanceof Error ? e.message : "SMTP test failed";
    }
  }

  async function saveDigestSchedule() {
    error = "";
    try {
      await api.setDigestEmailSettings({
        enabled: digestEnabled,
        interval: digestInterval,
      });
      digestStatus = await api.getDigestEmailStatus();
      message = t("settings.smtp.digestSaved");
    } catch (e) {
      error = e instanceof Error ? e.message : "Failed to save digest schedule";
    }
  }

  async function runDigestNow() {
    error = "";
    try {
      const result = await api.runScheduledDigestNow();
      digestStatus = await api.getDigestEmailStatus();
      emailLog = await api.listEmailLog(10);
      message = t("settings.smtp.digestDone")
        .replace("{sent}", String(result.sent))
        .replace("{failed}", String(result.failed))
        .replace("{skipped}", String(result.skipped));
    } catch (e) {
      error = e instanceof Error ? e.message : "Digest run failed";
    }
  }



  async function pushCloudBackup() {

    try {

      const result = await api.pushBackupToCloud();

      message = `${result.message} (${result.exported_at})`;

    } catch (e) {

      error = e instanceof Error ? e.message : "Cloud push failed";

    }

  }



  async function saveAppearance() {

    await preferences.save(ui);

    message = t("appearance.saved");

  }



  async function exportData() {

    const backup = await api.exportBackup();

    const blob = new Blob([JSON.stringify(backup, null, 2)], { type: "application/json" });

    const url = URL.createObjectURL(blob);

    const a = document.createElement("a");

    a.href = url;

    a.download = `classmate-backup-${backup.exported_at.slice(0, 10)}.json`;

    a.click();

    URL.revokeObjectURL(url);

  }



  async function importData(event: Event) {

    const input = event.target as HTMLInputElement;

    const file = input.files?.[0];

    if (!file) return;

    const text = await file.text();

    const payload = JSON.parse(text) as BackupPayload;

    await api.importBackup(payload);

    message = t("settings.backup.restored");

    input.value = "";

    await load();

  }



  async function importCsv() {

    importResult = await api.importOnerosterCsv(csvText);

  }



  async function startVideo() {

    video = await api.startVideo();

  }



  async function stopVideo() {

    video = await api.stopVideo();

  }



  async function addLti() {

    await api.createLtiTool({

      name: ltiName,

      launch_url: ltiUrl,

      consumer_key: ltiKey,

      shared_secret: ltiSecret,

    });

    ltiName = ltiUrl = ltiKey = ltiSecret = "";

    ltiTools = await api.listLtiTools();

  }



  async function startSyncServer() {

    sync = await api.startSyncServer();

    syncTokenEdit = sync.sync_token;

    message = t("settings.sync.started");

  }



  async function stopSyncServer() {

    sync = await api.stopSyncServer();

    message = t("settings.sync.stoppedMsg");

  }



  async function saveSyncToken() {

    await api.setSyncToken(syncTokenEdit);

    sync = await api.getSyncStatus();

    message = t("settings.sync.tokenSaved");

  }



  async function savePublicUrlSettings() {

    error = "";

    try {

      await api.setSetting("public_base_url", publicBaseUrl.trim());

      await api.setSetting("public_hub_path", publicHubPath.trim() || "/hub");

      sync = await api.getSyncStatus();

      waBusiness = await api.getWhatsAppBusinessSettings();

      message = t("settings.sync.publicSaved");

    } catch (e) {

      error = e instanceof Error ? e.message : "Failed to save public URL settings";

    }

  }



  async function pullPeer() {

    if (!peerUrl.trim()) return;

    const result = await api.pullFromPeer(peerUrl.trim());

    message = `${result.message} (${result.exported_at})`;

  }



  async function pushPeer() {

    if (!peerUrl.trim()) return;

    const result = await api.pushToPeer(peerUrl.trim());

    message = `${result.message} (${result.exported_at})`;

  }



  async function saveAutoBackup() {

    await api.setAutoBackupSettings({

      enabled: autoEnabled,

      interval: autoInterval,

      max_keep: autoMaxKeep,

    });

    autoBackup = await api.getAutoBackupStatus();

    message = t("settings.backup.autoSaved");

  }



  async function backupNow() {

    await api.runAutoBackupNow();

    autoBackup = await api.getAutoBackupStatus();

    backupFiles = await api.listAutoBackups();

    message = t("settings.backup.backupDone");

  }



  async function restoreBackup(filename: string) {

    if (!confirm(t("settings.backup.confirmRestore"))) return;

    error = "";

    try {

      await api.restoreAutoBackup(filename);

      message = t("settings.backup.restored");

      await load();

    } catch (e) {

      error = e instanceof Error ? e.message : "Restore failed";

    }

  }



  function formatBytes(size: number) {

    if (size < 1024) return `${size} B`;

    if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;

    return `${(size / (1024 * 1024)).toFixed(1)} MB`;

  }



  onMount(load);

</script>



<div class="page-header">

  <div>

    <h2>{t("settings.title")}</h2>

    <p>{t("settings.subtitle")}</p>

  </div>

</div>



{#if message}<p style="color:var(--success)">{message}</p>{/if}

{#if error}<p class="error">{error}</p>{/if}



<div class="grid two-col">

  <div class="card">

    <h3 style="margin-top:0">{t("settings.video.galeneTitle")}</h3>

    {#if video}

      <p>{video.message}</p>

      <p>

        {t("settings.video.installed")}: {video.galene_installed ? t("settings.video.yes") : t("settings.video.no")} ·

        {t("settings.video.running")}: {video.running ? t("settings.video.yes") : t("settings.video.no")}

      </p>

      {#if video.url}<p class="mono">{video.url}</p>{/if}

    {/if}

    <p style="color:var(--muted);font-size:.9rem">

      {t("settings.video.installHint")} <code>powershell -File scripts/install-galene.ps1</code>

    </p>

    <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">

      <button class="btn btn-secondary" onclick={() => api.getVideoStatus().then((v) => (video = v))}>

        {t("settings.video.refresh")}

      </button>

      <button class="btn btn-primary" onclick={startVideo}>{t("settings.video.start")}</button>

      <button class="btn btn-danger" onclick={stopVideo}>{t("settings.video.stop")}</button>

    </div>



    <h4 style="margin-top:1.25rem">{t("settings.video.plugnmeetTitle")}</h4>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.video.plugnmeetHint")}</p>

    <p style="color:var(--muted);font-size:.85rem">

      <code>powershell -File scripts/install-plugnmeet.ps1</code>

    </p>

    <label style="display:block;margin-top:.75rem">

      {t("settings.video.plugnmeetBase")}

      <input bind:value={plugnmeetBaseUrl} placeholder="http://192.168.1.10:8080" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.video.plugnmeetRoom")}

      <input bind:value={plugnmeetRoom} placeholder="classmate" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.video.externalUrl")}

      <input bind:value={externalVideoUrl} placeholder="https://meet.example.com/room" />

    </label>

    <button class="btn btn-primary" style="margin-top:.75rem" onclick={saveVideoSettings}>

      {t("settings.video.saveVideo")}

    </button>

  </div>



  {#if tenancyContext?.is_org_admin}

  <div class="card">

    <h3 style="margin-top:0">{t("settings.schools.title")}</h3>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.schools.hint")}</p>

    <h4 style="margin:1rem 0 .5rem">{t("settings.schools.activeTitle")}</h4>

    <label style="display:block;margin-top:.5rem">

      {t("settings.schools.name")}

      <input bind:value={editSchoolName} />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.schools.code")}

      <input bind:value={editSchoolCode} placeholder="MAIN" />

    </label>

    <button class="btn btn-primary" type="button" style="margin-top:.75rem" onclick={updateActiveSchool}>

      {t("settings.schools.saveActive")}

    </button>

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.schools.createTitle")}</h4>

    <label style="display:block;margin-top:.5rem">

      {t("settings.schools.name")}

      <input bind:value={newSchoolName} placeholder="East Campus" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.schools.code")}

      <input bind:value={newSchoolCode} placeholder="EAST" />

    </label>

    <button class="btn btn-secondary" type="button" style="margin-top:.75rem" onclick={createSchool}>

      {t("settings.schools.create")}

    </button>

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.schools.membersTitle")}</h4>

    <label style="display:block;margin-top:.5rem">

      {t("settings.schools.memberEmail")}

      <input bind:value={memberEmail} placeholder="teacher@classmate.local" />

    </label>

    <button class="btn btn-secondary" type="button" style="margin-top:.75rem" onclick={addSchoolMember}>

      {t("settings.schools.addMember")}

    </button>

    {#if schoolMembers.length > 0}

      <table class="table" style="margin-top:1rem">

        <thead>

          <tr>

            <th>{t("settings.schools.memberName")}</th>

            <th>{t("settings.schools.memberRole")}</th>

            <th></th>

          </tr>

        </thead>

        <tbody>

          {#each schoolMembers as member}

            <tr>

              <td>{member.user_name}<br /><small style="color:var(--muted)">{member.user_email}</small></td>

              <td>{member.user_role}</td>

              <td>

                <button class="btn btn-secondary btn-sm" type="button" onclick={() => removeSchoolMember(member.id)}>

                  {t("settings.schools.removeMember")}

                </button>

              </td>

            </tr>

          {/each}

        </tbody>

      </table>

    {/if}

  </div>

  {/if}



  <div class="card">

    <h3 style="margin-top:0">{t("appearance.title")}</h3>

    <div class="form-grid">

      <label>

        {t("appearance.schoolName")}

        <input bind:value={ui.school_name} />

      </label>

      <label>

        {t("appearance.theme")}

        <select bind:value={ui.theme}>

          <option value="default">{t("appearance.themeDefault")}</option>

          <option value="high-contrast">{t("appearance.themeHighContrast")}</option>

        </select>

      </label>

      <label>

        {t("appearance.fontScale")}

        <select bind:value={ui.font_scale}>

          <option value="100">100%</option>

          <option value="110">110%</option>

          <option value="125">125%</option>

        </select>

      </label>

      <label>

        {t("appearance.accent")}

        <input type="color" bind:value={ui.accent_color} />

      </label>

      <label>

        {t("appearance.locale")}

        <select bind:value={ui.locale}>

          <option value="en">English</option>

          <option value="es">Español</option>

        </select>

      </label>

      <button class="btn btn-primary" onclick={saveAppearance}>{t("appearance.save")}</button>

    </div>

  </div>



  <div class="card">

    <h3 style="margin-top:0">{t("settings.sync.title")}</h3>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.sync.hint")}</p>

    {#if sync}

      <p>

        {t("settings.sync.server")}:

        {sync.running ? t("settings.sync.running") : t("settings.sync.stopped")} ·

        {t("settings.sync.port")} {sync.port}

      </p>

      {#if sync.sync_url}

        <p class="mono">{sync.sync_url}</p>

      {/if}

      {#if sync.webhook_url}

        <p style="font-size:.85rem;margin-top:.5rem">

          WhatsApp webhook: <span class="mono">{sync.webhook_url}</span>

        </p>

      {/if}

      {#if sync.hub_join_url}

        <p style="font-size:.85rem;margin-top:.35rem">

          Class Hub (public): <span class="mono">{sync.hub_join_url}</span>

        </p>

      {/if}

      <h4 style="margin:1rem 0 .35rem;font-size:.95rem">{t("settings.sync.publicTitle")}</h4>

      <p style="color:var(--muted);font-size:.85rem;margin-bottom:.75rem">{t("settings.sync.publicHint")}</p>

      <label style="display:block">

        {t("settings.sync.publicBaseUrl")}

        <input bind:value={publicBaseUrl} placeholder="https://classmate.yourschool.com" />

      </label>

      <label style="display:block;margin-top:.75rem">

        {t("settings.sync.publicHubPath")}

        <input bind:value={publicHubPath} placeholder="/hub" />

      </label>

      <button class="btn btn-secondary btn-sm" type="button" style="margin-top:.75rem" onclick={savePublicUrlSettings}>

        {t("settings.sync.publicSave")}

      </button>

      <label style="display:block;margin-top:.75rem">

        {t("settings.sync.token")}

        <input bind:value={syncTokenEdit} />

      </label>

      <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">

        <button class="btn btn-secondary" onclick={saveSyncToken}>{t("settings.sync.saveToken")}</button>

        {#if !sync.running}

          <button class="btn btn-primary" onclick={startSyncServer}>{t("settings.sync.start")}</button>

        {:else}

          <button class="btn btn-danger" onclick={stopSyncServer}>{t("settings.sync.stopServer")}</button>

        {/if}

      </div>

      <label style="display:block;margin-top:1rem">

        {t("settings.sync.peerUrl")}

        <input bind:value={peerUrl} placeholder="http://192.168.1.10:8766" />

      </label>

      <div style="display:flex;gap:.5rem;margin-top:.75rem">

        <button class="btn btn-primary" onclick={pullPeer}>{t("settings.sync.pull")}</button>

        <button class="btn btn-secondary" onclick={pushPeer}>{t("settings.sync.push")}</button>

      </div>

    {/if}

  </div>



  <div class="card">

    <h3 style="margin-top:0">{t("settings.backup.title")}</h3>

    <p style="color:var(--muted)">{t("settings.backup.hint")}</p>

    <div style="display:flex;gap:.5rem;margin:.75rem 0;flex-wrap:wrap">

      <button class="btn btn-primary" onclick={exportData}>{t("settings.backup.export")}</button>

      <label class="btn btn-secondary" style="cursor:pointer">

        {t("settings.backup.import")}

        <input type="file" accept="application/json" hidden onchange={importData} />

      </label>

    </div>



    <h4>{t("settings.backup.autoTitle")}</h4>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.backup.autoHint")}</p>

    {#if autoBackup}

      <p style="font-size:.9rem;color:var(--muted)">

        {t("settings.backup.folder")}: <span class="mono">{autoBackup.backup_dir}</span>

      </p>

      <p style="font-size:.9rem">

        {t("settings.backup.lastRun")}:

        {autoBackup.last_backup_at ?? t("settings.backup.never")}

      </p>

      {#if autoBackup.next_due_at}

        <p style="font-size:.9rem">{t("settings.backup.nextRun")}: {autoBackup.next_due_at}</p>

      {/if}

    {/if}

    <div class="form-grid" style="margin-top:.75rem">

      <label style="display:flex;align-items:center;gap:.5rem">

        <input type="checkbox" bind:checked={autoEnabled} />

        {t("settings.backup.enable")}

      </label>

      <label>

        {t("settings.backup.interval")}

        <select bind:value={autoInterval}>

          <option value="daily">{t("settings.backup.daily")}</option>

          <option value="weekly">{t("settings.backup.weekly")}</option>

          <option value="off">{t("settings.backup.off")}</option>

        </select>

      </label>

      <label>

        {t("settings.backup.maxKeep")}

        <input type="number" min="1" max="30" bind:value={autoMaxKeep} />

      </label>

      <div style="display:flex;gap:.5rem;flex-wrap:wrap">

        <button class="btn btn-primary" onclick={saveAutoBackup}>{t("settings.backup.saveAuto")}</button>

        <button class="btn btn-secondary" onclick={backupNow}>{t("settings.backup.backupNow")}</button>

      </div>

    </div>



    {#if backupFiles.length > 0}

      <ul style="padding-left:1.1rem;margin-top:1rem">

        {#each backupFiles as file}

          <li style="margin-bottom:.5rem">

            <strong>{file.filename}</strong>

            <span style="color:var(--muted);font-size:.85rem"> · {formatBytes(file.size_bytes)}</span>

            <button class="btn btn-secondary btn-sm" onclick={() => restoreBackup(file.filename)}>

              {t("settings.backup.restore")}

            </button>

          </li>

        {/each}

      </ul>

    {/if}



    <h4 style="margin-top:1.25rem">{t("settings.backup.cloudTitle")}</h4>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.backup.cloudHint")}</p>

    <label style="display:block;margin-top:.75rem">

      {t("settings.backup.cloudUrl")}

      <input bind:value={cloudBackupUrl} placeholder="https://backup.example.com" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.backup.cloudToken")}

      <input bind:value={cloudBackupToken} />

    </label>

    <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">

      <button class="btn btn-secondary" onclick={saveVideoSettings}>{t("settings.saved")}</button>

      <button class="btn btn-primary" onclick={pushCloudBackup}>{t("settings.backup.pushCloud")}</button>

    </div>

  </div>



  <div class="card">

    <h3 style="margin-top:0">{t("settings.whatsapp.title")}</h3>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.whatsapp.hint")}</p>

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsapp.countryCode")}

      <input bind:value={whatsappCountryCode} placeholder="1" inputmode="numeric" />

    </label>

    <button class="btn btn-secondary" style="margin-top:.75rem" onclick={saveVideoSettings}>{t("settings.saved")}</button>

  </div>



  <div class="card">

    <h3 style="margin-top:0">{t("settings.whatsappBusiness.title")}</h3>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.whatsappBusiness.hint")}</p>

    <p style="font-size:.9rem;margin:.5rem 0">

      {waBusiness?.configured ? t("settings.whatsappBusiness.configured") : t("settings.whatsappBusiness.notConfigured")}

    </p>

    {#if waBusiness?.webhook_url}

      <p style="font-size:.85rem;color:var(--muted);word-break:break-all">

        {t("settings.whatsappBusiness.webhookUrl")}: {waBusiness.webhook_url}

      </p>

    {/if}

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsappBusiness.apiVersion")}

      <input bind:value={waApiVersion} placeholder="v21.0" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsappBusiness.phoneNumberId")}

      <input bind:value={waPhoneNumberId} placeholder="1234567890" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsappBusiness.accessToken")}

      <input type="password" bind:value={waAccessToken} placeholder={waBusiness?.access_token_set ? "Leave blank to keep current" : "EAA..."} />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsappBusiness.webhookToken")}

      <input bind:value={waWebhookToken} placeholder="your-verify-token" />

    </label>

    <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">

      <button class="btn btn-primary" type="button" onclick={saveWhatsAppBusiness}>{t("settings.whatsappBusiness.save")}</button>

      <button class="btn btn-secondary" type="button" onclick={testWhatsAppBusiness}>{t("settings.whatsappBusiness.test")}</button>

    </div>

    {#if waTestResult}

      <p style="margin-top:.75rem;font-size:.9rem;color:var(--muted)">{waTestResult}</p>

    {/if}

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.whatsappBusiness.templateTitle")}</h4>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.whatsappBusiness.templateHint")}</p>

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsappBusiness.templateName")}

      <input bind:value={waTemplateName} placeholder="assignment_reminder" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsappBusiness.templateLanguage")}

      <input bind:value={waTemplateLanguage} placeholder="en" />

    </label>

    <button class="btn btn-primary" type="button" style="margin-top:.75rem" onclick={saveWhatsAppTemplateSettings}>{t("settings.whatsappBusiness.templateSave")}</button>

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.whatsappBusiness.groupInviteTemplateTitle")}</h4>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.whatsappBusiness.groupInviteTemplateHint")}</p>

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsappBusiness.templateName")}

      <input bind:value={waGroupInviteTemplateName} placeholder="group_invite" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.whatsappBusiness.templateLanguage")}

      <input bind:value={waGroupInviteTemplateLanguage} placeholder="en" />

    </label>

    <button class="btn btn-primary" type="button" style="margin-top:.75rem" onclick={saveWhatsAppTemplateSettings}>{t("settings.whatsappBusiness.groupInviteTemplateSave")}</button>

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.whatsappBusiness.complianceTitle")}</h4>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.whatsappBusiness.complianceHint")}</p>

    <label style="display:flex;align-items:center;gap:.5rem;margin-top:.75rem">
      <input type="checkbox" bind:checked={waAutoUnsubscribe} />
      {t("settings.whatsappBusiness.autoUnsubscribe")}
    </label>

    <label style="display:block;margin-top:.75rem">
      {t("settings.whatsappBusiness.unsubscribeKeywords")}
      <input bind:value={waUnsubscribeKeywords} placeholder="STOP,UNSUBSCRIBE,CANCEL" />
    </label>

    <button class="btn btn-primary" type="button" style="margin-top:.75rem" onclick={saveWhatsAppComplianceSettings}>{t("settings.whatsappBusiness.complianceSave")}</button>

  </div>



  <div class="card">

    <h3 style="margin-top:0">{t("settings.push.title")}</h3>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.push.hint")}</p>

    <label style="display:flex;align-items:center;gap:.5rem;margin-top:.75rem">

      <input type="checkbox" bind:checked={pushEnabled} />

      {t("settings.push.enable")}

    </label>

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.push.fcmTitle")}</h4>

    <label style="display:block;margin-top:.5rem">

      {t("settings.push.fcmProject")}

      <input bind:value={fcmProjectId} placeholder="classmate-app" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.push.fcmServiceAccount")}

      <textarea bind:value={fcmServiceAccountJson} rows="4" placeholder="Paste Firebase service account JSON"></textarea>

    </label>

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.push.apnsTitle")}</h4>

    <label style="display:block;margin-top:.5rem">

      {t("settings.push.apnsKeyId")}

      <input bind:value={apnsKeyId} placeholder="ABC123DEFG" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.push.apnsTeamId")}

      <input bind:value={apnsTeamId} placeholder="TEAMID1234" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.push.apnsBundleId")}

      <input bind:value={apnsBundleId} placeholder="com.example.classmate" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.push.apnsPrivateKey")}

      <textarea bind:value={apnsPrivateKey} rows="4" placeholder="-----BEGIN PRIVATE KEY-----"></textarea>

    </label>

    <label style="display:flex;align-items:center;gap:.5rem;margin-top:.75rem">

      <input type="checkbox" bind:checked={apnsUseSandbox} />

      {t("settings.push.apnsSandbox")}

    </label>

    <button class="btn btn-primary" type="button" style="margin-top:.75rem" onclick={savePushSettings}>{t("settings.push.save")}</button>

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.push.testTitle")}</h4>

    <label style="display:block;margin-top:.5rem">

      {t("settings.push.testPlatform")}

      <select bind:value={pushTestPlatform} style="display:block;margin-top:.25rem;width:100%">

        <option value="fcm">FCM (Android)</option>

        <option value="apns">APNs (iOS)</option>

      </select>

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.push.testToken")}

      <input bind:value={pushTestToken} placeholder="device token" />

    </label>

    <button class="btn btn-secondary" type="button" style="margin-top:.75rem" onclick={testPushSettings}>{t("settings.push.testSend")}</button>

    {#if pushTestResult}

      <p style="margin-top:.75rem;font-size:.9rem;color:var(--muted)">{pushTestResult}</p>

    {/if}

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.push.remindersTitle")}</h4>

    {#if pushReminderStatus}

      <p style="font-size:.9rem;color:var(--muted)">

        {pushReminderStatus.push_configured ? t("settings.push.configured") : t("settings.push.notConfigured")}

        · {t("settings.push.deviceCount").replace("{count}", String(pushReminderStatus.device_count))}

      </p>

      {#if pushReminderStatus.last_run_at}

        <p style="font-size:.9rem">{t("settings.push.lastRun")}: {pushReminderStatus.last_run_at}</p>

      {/if}

    {/if}

    <label style="display:flex;align-items:center;gap:.5rem;margin-top:.75rem">

      <input type="checkbox" bind:checked={pushRemindersEnabled} />

      {t("settings.push.remindersEnable")}

    </label>

    <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">

      <button class="btn btn-primary" type="button" onclick={savePushReminders}>{t("settings.push.remindersSave")}</button>

      <button class="btn btn-secondary" type="button" onclick={runPushRemindersNow}>{t("settings.push.remindersNow")}</button>

    </div>

    {#if pushLog.length > 0}

      <h4 style="margin:1rem 0 .5rem">{t("settings.push.logTitle")}</h4>

      <ul class="plain-list" style="font-size:.85rem;color:var(--muted)">

        {#each pushLog as entry}

          <li>{entry.status} · {entry.platform} · {entry.title}</li>

        {/each}

      </ul>

    {/if}

  </div>



  <div class="card">

    <h3 style="margin-top:0">{t("settings.smtp.title")}</h3>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.smtp.hint")}</p>

    <p style="font-size:.9rem;margin:.5rem 0">

      {smtpConfigured ? t("settings.smtp.configured") : t("settings.smtp.notConfigured")}

    </p>

    <label style="display:block;margin-top:.75rem">

      {t("settings.smtp.host")}

      <input bind:value={smtpHost} placeholder="smtp.gmail.com" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.smtp.port")}

      <input type="number" bind:value={smtpPort} />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.smtp.username")}

      <input bind:value={smtpUsername} />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.smtp.from")}

      <input type="email" bind:value={smtpFrom} placeholder="classmate@school.edu" />

    </label>

    <label style="display:block;margin-top:.75rem">

      {t("settings.smtp.password")}

      <input type="password" bind:value={smtpPassword} placeholder="Leave blank to keep current" />

    </label>

    <label style="display:flex;align-items:center;gap:.5rem;margin-top:.75rem;font-weight:600">

      <input type="checkbox" bind:checked={smtpUseTls} />

      {t("settings.smtp.useTls")}

    </label>

    <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">

      <button class="btn btn-primary" type="button" onclick={saveSmtpSettings}>{t("settings.smtp.save")}</button>

    </div>

    <label style="display:block;margin-top:1rem">

      {t("settings.smtp.testTo")}

      <input type="email" bind:value={smtpTestTo} placeholder="you@example.com" />

    </label>

    <button class="btn btn-secondary" type="button" style="margin-top:.5rem" onclick={testSmtpSettings}>{t("settings.smtp.test")}</button>

    {#if smtpTestResult}

      <p style="margin-top:.75rem;font-size:.9rem;color:var(--muted)">{smtpTestResult}</p>

    {/if}

    <h4 style="margin:1.25rem 0 .5rem">{t("settings.smtp.digestTitle")}</h4>

    <p style="color:var(--muted);font-size:.9rem">{t("settings.smtp.digestHint")}</p>

    {#if digestStatus}
      <p style="font-size:.9rem">
        {digestStatus.smtp_configured ? t("settings.smtp.configured") : t("settings.smtp.notConfigured")}
        · {t("settings.smtp.parentCount").replace("{count}", String(digestStatus.parent_count))}
      </p>
      <p style="font-size:.9rem">
        {t("settings.smtp.digestLastRun")}:
        {digestStatus.last_run_at ?? t("settings.smtp.digestNever")}
      </p>
      {#if digestStatus.next_due_at}
        <p style="font-size:.9rem">{t("settings.smtp.digestNextRun")}: {digestStatus.next_due_at}</p>
      {/if}
    {/if}

    <label style="display:flex;align-items:center;gap:.5rem;margin-top:.75rem">
      <input type="checkbox" bind:checked={digestEnabled} />
      {t("settings.smtp.digestEnable")}
    </label>

    <label style="display:block;margin-top:.75rem">
      {t("settings.smtp.digestInterval")}
      <select bind:value={digestInterval} style="display:block;margin-top:.25rem;width:100%">
        <option value="daily">{t("settings.smtp.digestDaily")}</option>
        <option value="weekly">{t("settings.smtp.digestWeekly")}</option>
        <option value="off">{t("settings.smtp.digestOff")}</option>
      </select>
    </label>

    <div style="display:flex;gap:.5rem;margin-top:.75rem;flex-wrap:wrap">
      <button class="btn btn-primary" type="button" onclick={saveDigestSchedule}>{t("settings.smtp.digestSave")}</button>
      <button class="btn btn-secondary" type="button" onclick={runDigestNow}>{t("settings.smtp.digestNow")}</button>
    </div>

    {#if emailLog.length > 0}

      <h4 style="margin:1rem 0 .5rem">{t("settings.smtp.logTitle")}</h4>

      <ul class="plain-list" style="font-size:.85rem;color:var(--muted)">

        {#each emailLog as entry}

          <li>{entry.status} · {entry.recipient} · {entry.subject}</li>

        {/each}

      </ul>

    {/if}

  </div>



  <div class="card">

    <h3 style="margin-top:0">{t("settings.oneroster.title")}</h3>

    <textarea bind:value={csvText} rows="6" placeholder="email,givenName,familyName,role"></textarea>

    <button class="btn btn-primary" style="margin-top:.5rem" onclick={importCsv}>

      {t("settings.oneroster.import")}

    </button>

    {#if importResult}

      <p>{t("settings.oneroster.result").replace("{imported}", String(importResult.imported)).replace("{skipped}", String(importResult.skipped))}</p>

    {/if}

  </div>



  <div class="card">

    <h3 style="margin-top:0">{t("settings.lti.title")}</h3>

    <div class="form-grid">

      <input bind:value={ltiName} placeholder="Tool name" />

      <input bind:value={ltiUrl} placeholder="Launch URL" />

      <input bind:value={ltiKey} placeholder="Consumer key" />

      <input bind:value={ltiSecret} placeholder="Shared secret" />

      <button class="btn btn-primary" onclick={addLti}>{t("settings.lti.add")}</button>

    </div>

    <ul style="padding-left:1.1rem;margin-top:1rem">

      {#each ltiTools as tool}

        <li>{tool.name} — {tool.launch_url}</li>

      {/each}

    </ul>

  </div>

</div>



<style>

  code {

    background: #f1f5f9;

    padding: 0.15rem 0.35rem;

    border-radius: 4px;

  }

  .mono {

    font-family: ui-monospace, monospace;

    word-break: break-all;

  }

  .btn-sm {

    margin-left: 0.5rem;

    padding: 0.25rem 0.55rem;

    font-size: 0.8rem;

  }

</style>

