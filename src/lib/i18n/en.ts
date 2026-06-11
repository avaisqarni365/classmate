export const en = {

  appName: "ClassMate",

  signOut: "Sign out",

  loading: "Loading ClassMate...",

  nav: {

    dashboard: "Dashboard",

    myCourses: "My courses",

    courses: "Courses",

    gradebook: "Gradebook",

    quizzes: "Quizzes",

    submissions: "Submissions",

    schedule: "Schedule",

    announcements: "Announcements",

    forums: "Forums",

    hub: "Class Hub",

    certificates: "Certificates",

    parent: "Parent",

    users: "Users",

    settings: "Settings",

    sessions: "Sessions",

    groups: "Groups",

    fees: "Cash book",

    help: "Help",

  },

  tenancy: {

    activeSchool: "Active school",

  },

  login: {

    title: "Sign in to your local workspace",

    email: "Email",

    password: "Password",

    submit: "Sign in",

  },

  student: {

    title: "My courses",

    subtitle: "Your enrolled classes, grades, and materials.",

    average: "Average",

    assignments: "Assignments",

    graded: "Graded",

    upcoming: "Upcoming work",

    materials: "Materials",

    rubricBreakdown: "Rubric breakdown",

    noCourses: "You are not enrolled in any courses yet.",

    teacher: "Teacher",

  },

  appearance: {

    title: "Appearance & language",

    schoolName: "School name",

    theme: "Theme",

    themeDefault: "Default",

    themeHighContrast: "High contrast",

    fontScale: "Text size",

    accent: "Accent color",

    locale: "Language",

    save: "Save appearance",

    saved: "Appearance saved.",

  },

  settings: {

    title: "Settings & sync",

    subtitle: "Video, backups, peer sync, and integrations.",

    saved: "Settings saved.",

    video: {

      title: "Live video",

      galeneTitle: "Galene (lightweight)",

      installed: "Installed",

      running: "Running",

      yes: "Yes",

      no: "No",

      installHint: "Install Galene:",

      refresh: "Refresh",

      start: "Start video",

      stop: "Stop",

      externalUrl: "Fallback external video URL",

      plugnmeetTitle: "plugNmeet (full classroom)",

      plugnmeetHint: "Self-host plugNmeet, then set your server URL below. Used when Galene is unavailable.",

      plugnmeetBase: "plugNmeet server URL",

      plugnmeetRoom: "Default room ID",

      saveVideo: "Save video settings",

    },

    sync: {

      title: "LAN peer sync",

      hint: "Sync databases between computers on the same Wi‑Fi. Use the same sync token on both machines.",

      server: "Server",

      running: "Running",

      stopped: "Stopped",

      port: "Port",

      token: "Sync token",

      saveToken: "Save token",

      start: "Start sync server",

      stopServer: "Stop sync server",

      peerUrl: "Peer URL",

      pull: "Pull from peer",

      push: "Push to peer",

      started: "LAN sync server started.",

      stoppedMsg: "LAN sync server stopped.",

      tokenSaved: "Sync token updated.",

      publicTitle: "Public domain (HTTPS)",

      publicHint: "Point your domain at this PC with a reverse proxy (see DEPLOY.md). Class Hub path defaults to /hub.",

      publicBaseUrl: "Public base URL",

      publicHubPath: "Hub URL path",

      publicSave: "Save public URLs",

      publicSaved: "Public URL settings saved.",

    },

    backup: {

      title: "Backup & restore",

      hint: "Export full JSON backup for migration or off-site sync.",

      export: "Export backup",

      import: "Import backup",

      restored: "Backup restored.",

      autoTitle: "Scheduled auto-backup",

      autoHint: "Saves backups to your app data folder on a schedule.",

      enable: "Enable auto-backup",

      interval: "Interval",

      daily: "Daily",

      weekly: "Weekly",

      off: "Off",

      maxKeep: "Keep newest backups",

      lastRun: "Last backup",

      nextRun: "Next due",

      never: "Never",

      backupNow: "Backup now",

      saveAuto: "Save schedule",

      autoSaved: "Auto-backup settings saved.",

      backupDone: "Backup saved.",

      folder: "Backup folder",

      restore: "Restore",

      confirmRestore: "Restore this backup? Current data will be replaced.",

      cloudTitle: "Cloud backup push",

      cloudHint: "POST full backup JSON to a remote ClassMate sync endpoint.",

      cloudUrl: "Cloud endpoint URL",

      cloudToken: "Cloud sync token",

      pushCloud: "Push backup to cloud",

    },

    whatsapp: {

      title: "WhatsApp sharing",

      hint: "Default country code prepended to local phone numbers (digits only). Use 1 for US/Canada, 52 for Mexico, etc.",

      countryCode: "Default country code",

    },

    whatsappBusiness: {

      title: "WhatsApp Business API",

      hint: "Connect Meta Cloud API for automated outbound messages. Start the LAN sync server so Meta can reach the webhook URL.",

      apiVersion: "Graph API version",

      phoneNumberId: "Phone number ID",

      accessToken: "Access token",

      webhookToken: "Webhook verify token",

      webhookUrl: "Webhook URL",

      save: "Save API settings",

      test: "Test connection",

      configured: "API configured",

      notConfigured: "API not configured",

      templateTitle: "Assignment reminder template",

      templateHint: "Use a Meta-approved template with 4 body variables: student name, course title, assignment title, due date.",

      templateName: "Template name",

      templateLanguage: "Language code",

      templateSave: "Save template settings",

      templateSaved: "Template settings saved.",

      groupInviteTemplateTitle: "Group invite template",

      groupInviteTemplateHint: "Meta-approved template with a group_id body parameter for native group invites.",

      groupInviteTemplateSave: "Save group invite template",

      complianceTitle: "Regional compliance",

      complianceHint: "Auto opt-out when users reply with keywords like STOP. Export per-user WhatsApp data from Users.",

      autoUnsubscribe: "Auto-unsubscribe on keyword replies",

      unsubscribeKeywords: "Unsubscribe keywords (comma-separated)",

      complianceSave: "Save compliance settings",

      complianceSaved: "Compliance settings saved.",

    },

    schools: {

      title: "Multi-school tenancy",

      hint: "Manage campuses and assign users to schools. Data is scoped to the active school in the sidebar.",

      activeTitle: "Active school",

      createTitle: "Create school",

      membersTitle: "School members",

      name: "School name",

      code: "School code",

      saveActive: "Save active school",

      create: "Create school",

      created: "School created.",

      updated: "School updated.",

      memberEmail: "User email to add",

      addMember: "Add member",

      memberAdded: "Member added.",

      memberRemoved: "Member removed.",

      memberName: "User",

      memberRole: "Role",

      removeMember: "Remove",

    },

    push: {

      title: "Mobile push (FCM / APNs)",

      hint: "Send assignment reminders to registered mobile devices. Class Hub students can register via POST /api/student/push/register.",

      enable: "Enable push notifications",

      fcmTitle: "Firebase Cloud Messaging",

      fcmProject: "FCM project ID",

      fcmServiceAccount: "Service account JSON",

      apnsTitle: "Apple Push Notification service",

      apnsKeyId: "APNs key ID",

      apnsTeamId: "Apple team ID",

      apnsBundleId: "Bundle ID",

      apnsPrivateKey: "APNs .p8 private key",

      apnsSandbox: "Use APNs sandbox",

      save: "Save push settings",

      saved: "Push settings saved.",

      testTitle: "Test push",

      testPlatform: "Platform",

      testToken: "Device token",

      testSend: "Send test",

      remindersTitle: "Assignment reminders",

      configured: "Push configured",

      notConfigured: "Push not configured",

      deviceCount: "{count} registered devices",

      lastRun: "Last reminder run",

      remindersEnable: "Daily assignment due reminders",

      remindersSave: "Save reminder schedule",

      remindersSaved: "Reminder settings saved.",

      remindersNow: "Run reminders now",

      remindersRun: "Sent {sent}, failed {failed}",

      logTitle: "Recent push log",

    },

    smtp: {

      title: "SMTP email",

      hint: "Send parent weekly digests by email. Works with Gmail, Outlook, and other SMTP providers.",

      host: "SMTP host",

      port: "Port",

      username: "Username",

      from: "From address",

      password: "Password",

      useTls: "Use STARTTLS",

      save: "Save SMTP settings",

      test: "Send test email",

      testTo: "Test recipient",

      configured: "SMTP configured",

      notConfigured: "SMTP not configured",

      logTitle: "Recent email log",

      digestTitle: "Scheduled parent digest",

      digestHint: "Automatically email weekly grade digests to all linked parents when the app starts.",

      digestEnable: "Enable scheduled digest",

      digestInterval: "Schedule",

      digestDaily: "Daily",

      digestWeekly: "Weekly",

      digestOff: "Off",

      digestSave: "Save digest schedule",

      digestNow: "Send all digests now",

      digestSaved: "Digest schedule saved.",

      digestDone: "Sent {sent}, failed {failed}, skipped {skipped}.",

      digestLastRun: "Last digest run",

      digestNextRun: "Next scheduled run",

      digestNever: "Never",

      parentCount: "{count} linked parents",

    },

    oneroster: {

      title: "OneRoster CSV import",

      import: "Import users",

      result: "Imported {imported}, skipped {skipped}",

    },

    lti: {

      title: "LTI 1.3 tools",

      add: "Add tool",

    },

  },

  fees: {

    title: "Cash book",

    subtitle: "Admin-only income and expense ledger for student fees, teacher salary, and school costs.",

    totalIncome: "Total income",

    totalExpense: "Total expense",

    balance: "Balance",

    entries: "Entries",

    filterTitle: "Date filter",

    fromDate: "From",

    toDate: "To",

    applyFilter: "Apply",

    exportCsv: "Export CSV",

    addEntry: "Record transaction",

    direction: "Type",

    income: "Income",

    expense: "Expense",

    category: "Category",

    categoryStudentFee: "Student fee",

    categoryOtherIncome: "Other income",

    categoryTeacherSalary: "Teacher salary",

    categoryOtherExpense: "Other expense",

    amount: "Amount",

    paymentMethod: "Payment method",

    methodCash: "Cash",

    methodBank: "Bank transfer",

    methodCheque: "Cheque",

    methodOnline: "Online",

    linkedUser: "Linked user",

    linkedCourse: "Linked course",

    none: "None",

    entryDate: "Date",

    reference: "Reference",

    description: "Description",

    saveEntry: "Save entry",

    entrySaved: "Entry saved.",

    invalidAmount: "Enter a valid amount.",

    deleteConfirm: "Delete this entry?",

    delete: "Delete",

    ledgerTitle: "Ledger",

    noEntries: "No entries yet.",

    integrationTitle: "Integration & currency",

    integrationHint: "Optional self-hosted Invoice Ninja (AGPL, open source) for invoicing. ClassMate keeps the local cash book; export CSV or connect Invoice Ninja for formal invoices.",

    currency: "Default currency",

    invoiceNinjaUrl: "Invoice Ninja URL",

    invoiceNinjaToken: "Invoice Ninja API token",

    saveSettings: "Save settings",

    settingsSaved: "Cash book settings saved.",

    testIntegration: "Test Invoice Ninja",

  },

  help: {

    title: "Help & setup guide",

    subtitle: "Install ClassMate locally, connect your domain, sync data, and configure WhatsApp.",

    toc: {

      download: "Download",

      local: "Local setup",

      domain: "Domain",

      sync: "Sync",

      hub: "Class Hub",

      whatsapp: "WhatsApp",

      security: "Security",

    },

    download: {

      title: "Download ClassMate (Windows)",

      body: "Install the desktop app on your admin PC. SQLite is built in — no separate database server.",

      web: "Open web download page",

      direct: "Download installer (.exe)",

      build: "Or build from source (Node.js 20+ and Rust required):",

      output: "Installer output: src-tauri\\target\\release\\bundle\\nsis\\ClassMate_0.26.0_x64-setup.exe",

    },

    local: {

      title: "Local setup (no domain)",

      step1: "Sign in and change all demo passwords under Users.",

      step2: "Create courses, enroll students, add assignments and materials.",

      step3: "Start Class Hub for live sessions — share the join URL with students.",

      step4: "Export backups regularly under Settings → Backup & restore.",

      step5: "All data is stored in a local SQLite file on this computer.",

    },

    demo: {

      role: "Role",

      email: "Email",

      password: "Password",

    },

    domain: {

      title: "Public domain (e.g. cm.codes-ai.uk)",

      body: "Use a VPS + reverse proxy so students, sync, and WhatsApp webhooks reach ClassMate over HTTPS.",

      step1: "GoDaddy DNS: A record cm → your server IP (Ionos: 212.227.54.250).",

      step2: "Settings → LAN peer sync → Public base URL = https://your-domain (hub path /hub).",

      step3: "On the server: nginx + certbot for HTTPS. See DEPLOY.md in the GitHub repo.",

      configured: "Your public URL:",

      notConfigured: "Public URL not set yet — configure in Settings → LAN peer sync.",

    },

    sync: {

      title: "LAN / server sync",

      body: "Keep the same database on your PC and your public server.",

      item1: "Start sync server (desktop or Linux classmate-server service).",

      item2: "Use the same sync token on both machines.",

      item3: "Push or pull via Settings using https://your-domain/api/sync/ and the x-sync-token header.",

      status: "Sync server",

      running: "running",

      stopped: "stopped",

    },

    hub: {

      title: "Class Hub",

      body: "Live student portal for materials, quizzes, polls, and submissions during class.",

      step1: "Open Class Hub → select course → Start hub.",

      step2: "Share the join URL (uses your public domain when configured).",

      step3: "Students enter the session PIN shown on the Hub page.",

    },

    whatsapp: {

      title: "WhatsApp Business API",

      body: "Optional automated messaging to opted-in group members.",

      step1: "Settings → WhatsApp Business API — enter Meta credentials and webhook verify token.",

      step2: "Paste the webhook URL from Settings into the Meta developer dashboard.",

      step3: "Subscribe to group and message webhook fields.",

      step4: "Groups page — manage consent, native groups, and delivery log.",

    },

    security: {

      title: "Before going live",

      item1: "Change all default demo passwords immediately.",

      item2: "Use a strong sync token; it protects backup import/export.",

      item3: "Only expose Hub and API routes via HTTPS — admin UI stays on the desktop app.",

    },

  },

} as const;



export type MessageKey = keyof typeof en | `${keyof typeof en}.${string}`;

