import { invoke } from "@tauri-apps/api/core";
import type {
  Announcement,
  Assignment,
  BackupPayload,
  CertificateInfo,
  Course,
  CourseMaterial,
  CreateAnnouncementInput,
  CreateAssignmentInput,
  CreateCourseInput,
  CreateForumPostInput,
  CreateForumTopicInput,
  CreateLtiToolInput,
  CreateMaterialInput,
  CaptureSession,
  CreateCaptureSessionInput,
  MaterialAiLab,
  MaterialWithAiLab,
  OpenStaxBook,
  ArtizAiConfig,
  CreateUserInput,
  CreateQuizInput,
  CreateQuizQuestionInput,
  CreateScheduleSlotInput,
  CreateSessionPollInput,
  DashboardStats,
  Enrollment,
  ForumPost,
  ForumTopic,
  Gradebook,
  HubStatus,
  ImportResult,
  LinkParentInput,
  LoginInput,
  LtiTool,
  ParentStudentSummary,
  Quiz,
  QuizAttempt,
  QuizAttemptDetail,
  GradeQuizAttemptInput,
  QuizDetail,
  AnalyticsReport,
  ScheduleSlot,
  PollResults,
  AssignmentSubmission,
  StudentDashboard,
  StudentCourseDetail,
  UiPreferences,
  SyncServerStatus,
  SyncPeerResult,
  ClassSessionRecord,
  AutoBackupSettings,
  AutoBackupStatus,
  AutoBackupEntry,
  AssignmentRubric,
  SaveAssignmentRubricInput,
  RubricScoreInput,
  ParentDigest,
  User,
  VideoStatus,
  WhatsAppGroup,
  WhatsAppGroupMember,
  CreateWhatsAppGroupInput,
  WhatsAppShareInput,
  WhatsAppSharePlan,
  WhatsAppBusinessSettings,
  SaveWhatsAppBusinessSettingsInput,
  WhatsAppConnectionTest,
  LinkWhatsAppGroupInput,
  WhatsAppGroupLink,
  CreateNativeWhatsAppGroupInput,
  NativeWhatsAppGroupResult,
  SendWhatsAppGroupInvitesInput,
  SendWhatsAppBroadcastInput,
  WhatsAppBroadcastResult,
  WhatsAppOutboundMessage,
  WhatsAppBroadcastSummary,
  WhatsAppMessageStatusEvent,
  WhatsAppTemplateSettings,
  SaveWhatsAppTemplateSettingsInput,
  SendWhatsAppTemplateBroadcastInput,
  WhatsAppTemplatePreview,
  WhatsAppInboundMessage,
  WhatsAppInboundRoutingSettings,
  SaveWhatsAppInboundRoutingSettingsInput,
  WhatsAppScheduledBroadcast,
  CreateWhatsAppScheduledBroadcastInput,
  WhatsAppScheduledRunResult,
  WhatsAppComplianceSettings,
  SaveWhatsAppComplianceSettingsInput,
  WhatsAppGdprExport,
  WhatsAppConsentLogEntry,
  TenancyContext,
  School,
  SchoolMember,
  CreateSchoolInput,
  UpdateSchoolInput,
  AddSchoolMemberInput,
  PushSettings,
  SavePushSettingsInput,
  PushLogEntry,
  PushReminderStatus,
  PushReminderRunResult,
  TestPushInput,
  ParentGradeEntry,
  RubricScoreDisplay,
  SmtpSettings,
  SaveSmtpSettingsInput,
  SmtpConnectionTest,
  SendParentDigestEmailInput,
  SendParentDigestEmailResult,
  EmailLogEntry,
  DigestEmailSettings,
  DigestEmailStatus,
  DigestEmailRunResult,
  CashbookEntry,
  CreateCashbookEntryInput,
  CashbookSummary,
  CashbookSettings,
  SaveCashbookSettingsInput,
  CashbookIntegrationTest,
  WhatsAppGroupRosterDiff,
  WhatsAppJoinRequest,
  SyncNativeWhatsAppGroupRosterInput,
  SyncNativeWhatsAppGroupRosterResult,
  ManageWhatsAppJoinRequestsInput,
  WhatsAppGroupParticipantEvent,
  WhatsAppGroupSettingsEvent,
  HelpInfo,
} from "./types";

export const api = {
  login: (input: LoginInput) => invoke<User>("login", { input }),
  logout: () => invoke<void>("logout"),
  getSession: () => invoke<User | null>("get_session"),
  getDashboardStats: () => invoke<DashboardStats>("get_dashboard_stats"),
  listUsers: () => invoke<User[]>("list_users"),
  listStudents: () => invoke<User[]>("list_students"),
  createUser: (input: CreateUserInput) => invoke<User>("create_user", { input }),
  listCourses: () => invoke<Course[]>("list_courses"),
  createCourse: (input: CreateCourseInput) => invoke<Course>("create_course", { input }),
  listAssignments: (courseId: string) =>
    invoke<Assignment[]>("list_assignments", { courseId }),
  createAssignment: (input: CreateAssignmentInput) =>
    invoke<Assignment>("create_assignment", { input }),
  getGradebook: (courseId: string) => invoke<Gradebook>("get_gradebook", { courseId }),
  saveGrade: (
    assignmentId: string,
    studentId: string,
    points?: number | null,
    feedback?: string,
  ) =>
    invoke<void>("save_grade", {
      input: {
        assignment_id: assignmentId,
        student_id: studentId,
        points: points ?? null,
        feedback: feedback ?? null,
      },
    }),
  exportGradebookCsv: (courseId: string) =>
    invoke<string>("export_gradebook_csv", { courseId }),
  listEnrollments: (courseId: string) =>
    invoke<Enrollment[]>("list_enrollments", { courseId }),
  enrollStudent: (courseId: string, studentId: string) =>
    invoke<Enrollment>("enroll_student", {
      input: { course_id: courseId, student_id: studentId },
    }),
  unenrollStudent: (enrollmentId: string) =>
    invoke<void>("unenroll_student", { enrollmentId }),
  listMaterials: (courseId: string) =>
    invoke<CourseMaterial[]>("list_materials", { courseId }),
  createMaterial: (input: CreateMaterialInput) =>
    invoke<CourseMaterial>("create_material", { input }),
  deleteMaterial: (materialId: string) =>
    invoke<void>("delete_material", { materialId }),
  listOpenStaxBooks: (subject?: string) =>
    invoke<OpenStaxBook[]>("list_openstax_books", { subject: subject ?? null }),
  listCourseLectures: (courseId: string) =>
    invoke<MaterialWithAiLab[]>("list_course_lectures", { courseId }),
  createCaptureSession: (input: CreateCaptureSessionInput) =>
    invoke<CaptureSession>("create_capture_session", { input }),
  getCaptureSession: (sessionId: string) =>
    invoke<CaptureSession>("get_capture_session", { sessionId }),
  attachCaptureSession: (sessionId: string, title?: string) =>
    invoke<CourseMaterial>("attach_capture_session", {
      sessionId,
      title: title ?? null,
    }),
  markMaterialLabComplete: (materialId: string) =>
    invoke<{ material_id: string; completed_at: string }>("mark_material_lab_complete", {
      materialId,
    }),
  getArtizAiConfig: () => invoke<ArtizAiConfig>("get_artizai_config"),
  saveArtizAiBaseUrl: (baseUrl: string) =>
    invoke<ArtizAiConfig>("save_artizai_base_url", { baseUrl }),
  resolveMaterialAiLab: (
    courseCode: string,
    courseTitle: string,
    materialTitle: string,
    subjects?: string[],
  ) =>
    invoke<MaterialAiLab>("resolve_material_ai_lab", {
      courseCode,
      courseTitle,
      materialTitle,
      subjects: subjects ?? null,
    }),
  listAnnouncements: (courseId?: string) =>
    invoke<Announcement[]>("list_announcements", { courseId: courseId ?? null }),
  createAnnouncement: (input: CreateAnnouncementInput) =>
    invoke<Announcement>("create_announcement", { input }),
  deleteAnnouncement: (id: string) => invoke<void>("delete_announcement", { id }),
  listForumTopics: (courseId: string) =>
    invoke<ForumTopic[]>("list_forum_topics", { courseId }),
  listForumPosts: (topicId: string) =>
    invoke<ForumPost[]>("list_forum_posts", { topicId }),
  createForumTopic: (input: CreateForumTopicInput) =>
    invoke<ForumTopic>("create_forum_topic", { input }),
  createForumPost: (input: CreateForumPostInput) =>
    invoke<ForumPost>("create_forum_post", { input }),
  exportBackup: () => invoke<BackupPayload>("export_backup"),
  importBackup: (payload: BackupPayload) => invoke<void>("import_backup", { payload }),
  getSetting: (key: string) => invoke<string | null>("get_setting", { key }),
  setSetting: (key: string, value: string) => invoke<void>("set_setting", { key, value }),
  importOnerosterCsv: (csv: string) => invoke<ImportResult>("import_oneroster_csv", { csv }),
  listLtiTools: () => invoke<LtiTool[]>("list_lti_tools"),
  createLtiTool: (input: CreateLtiToolInput) => invoke<LtiTool>("create_lti_tool", { input }),
  deleteLtiTool: (id: string) => invoke<void>("delete_lti_tool", { id }),
  getLtiLaunchUrl: (toolId: string, courseId: string) =>
    invoke<string>("get_lti_launch_url", { toolId, courseId }),
  linkParentStudent: (input: LinkParentInput) =>
    invoke<void>("link_parent_student", { input }),
  getParentDashboard: (parentId: string) =>
    invoke<ParentStudentSummary[]>("get_parent_dashboard", { parentId }),
  issueCertificate: (courseId: string, studentId: string) =>
    invoke<CertificateInfo>("issue_certificate", {
      input: { course_id: courseId, student_id: studentId },
    }),
  listCertificates: (courseId?: string) =>
    invoke<CertificateInfo[]>("list_certificates", { courseId: courseId ?? null }),
  getVideoStatus: () => invoke<VideoStatus>("get_video_status"),
  startVideo: () => invoke<VideoStatus>("start_video"),
  stopVideo: () => invoke<VideoStatus>("stop_video"),
  getHubStatus: () => invoke<HubStatus>("get_hub_status"),
  startClassHub: (courseId: string, title?: string, enableVideo = true) =>
    invoke<HubStatus>("start_class_hub", {
      input: { course_id: courseId, title, enable_video: enableVideo },
    }),
  stopClassHub: () => invoke<HubStatus>("stop_class_hub"),
  getAttendanceCount: (sessionId: string) =>
    invoke<number>("get_attendance_count", { sessionId }),
  listQuizzes: (courseId: string) => invoke<Quiz[]>("list_quizzes", { courseId }),
  createQuiz: (input: CreateQuizInput) => invoke<Quiz>("create_quiz", { input }),
  addQuizQuestion: (input: CreateQuizQuestionInput) =>
    invoke("add_quiz_question", { input }),
  publishQuiz: (quizId: string) => invoke<void>("publish_quiz", { quizId }),
  getQuizDetail: (quizId: string) => invoke<QuizDetail>("get_quiz_detail", { quizId }),
  listQuizAttempts: (quizId: string) =>
    invoke<QuizAttempt[]>("list_quiz_attempts", { quizId }),
  getQuizAttemptDetail: (attemptId: string) =>
    invoke<QuizAttemptDetail>("get_quiz_attempt_detail", { attemptId }),
  gradeQuizAttempt: (input: GradeQuizAttemptInput) =>
    invoke<QuizAttempt>("grade_quiz_attempt", { input }),
  getAnalytics: () => invoke<AnalyticsReport>("get_analytics"),
  exportAttendanceCsv: (sessionId: string) =>
    invoke<string>("export_attendance_csv", { sessionId }),
  listSchedule: (courseId?: string) =>
    invoke<ScheduleSlot[]>("list_schedule", { courseId: courseId ?? null }),
  createScheduleSlot: (input: CreateScheduleSlotInput) =>
    invoke<ScheduleSlot>("create_schedule_slot", { input }),
  deleteScheduleSlot: (slotId: string) =>
    invoke<void>("delete_schedule_slot", { slotId }),
  createSessionPoll: (input: CreateSessionPollInput) =>
    invoke<PollResults>("create_session_poll", { input }),
  closeSessionPoll: (pollId: string) => invoke<PollResults>("close_session_poll", { pollId }),
  getActiveSessionPoll: (sessionId: string) =>
    invoke<PollResults | null>("get_active_session_poll", { sessionId }),
  getSessionPollResults: (pollId: string) =>
    invoke<PollResults>("get_session_poll_results", { pollId }),
  listSubmissions: (courseId: string) =>
    invoke<AssignmentSubmission[]>("list_submissions", { courseId }),
  gradeSubmission: (input: {
    submission_id: string;
    points: number;
    feedback?: string;
    rubric_scores?: RubricScoreInput[];
  }) => invoke<AssignmentSubmission>("grade_submission", { input }),
  getAssignmentRubric: (assignmentId: string) =>
    invoke<AssignmentRubric | null>("get_assignment_rubric", { assignmentId }),
  saveAssignmentRubric: (input: SaveAssignmentRubricInput) =>
    invoke<AssignmentRubric>("save_assignment_rubric", { input }),
  generateParentDigest: (parentId: string) =>
    invoke<ParentDigest>("generate_parent_digest", { parentId }),
  listParentGrades: (parentId: string) =>
    invoke<ParentGradeEntry[]>("list_parent_grades", { parentId }),
  getSmtpSettings: () => invoke<SmtpSettings>("get_smtp_settings"),
  saveSmtpSettings: (input: SaveSmtpSettingsInput) =>
    invoke<SmtpSettings>("save_smtp_settings", { input }),
  testSmtpConnection: (testRecipient: string) =>
    invoke<SmtpConnectionTest>("test_smtp_connection", { testRecipient }),
  sendParentDigestEmail: (input: SendParentDigestEmailInput) =>
    invoke<SendParentDigestEmailResult>("send_parent_digest_email", { input }),
  listEmailLog: (limit?: number) =>
    invoke<EmailLogEntry[]>("list_email_log", { limit: limit ?? null }),
  getDigestEmailStatus: () => invoke<DigestEmailStatus>("get_digest_email_status"),
  setDigestEmailSettings: (input: DigestEmailSettings) =>
    invoke<void>("set_digest_email_settings", { input }),
  runScheduledDigestNow: () =>
    invoke<DigestEmailRunResult>("run_scheduled_digest_now"),
  pushBackupToCloud: () => invoke<SyncPeerResult>("push_backup_to_cloud"),
  getStudentDashboard: () => invoke<StudentDashboard>("get_student_dashboard"),
  getMyCourse: (courseId: string) => invoke<StudentCourseDetail>("get_my_course", { courseId }),
  getUiPreferences: () => invoke<UiPreferences>("get_ui_preferences"),
  setUiPreferences: (input: UiPreferences) => invoke<void>("set_ui_preferences", { input }),
  getSyncStatus: () => invoke<SyncServerStatus>("get_sync_status"),
  getHelpInfo: () => invoke<HelpInfo>("get_help_info"),
  startSyncServer: () => invoke<SyncServerStatus>("start_sync_server"),
  stopSyncServer: () => invoke<SyncServerStatus>("stop_sync_server"),
  setSyncToken: (token: string) => invoke<void>("set_sync_token", { token }),
  pullFromPeer: (peerUrl: string) => invoke<SyncPeerResult>("pull_from_peer", { peerUrl }),
  pushToPeer: (peerUrl: string) => invoke<SyncPeerResult>("push_to_peer", { peerUrl }),
  listClassSessions: (courseId?: string) =>
    invoke<ClassSessionRecord[]>("list_class_sessions", { courseId: courseId ?? null }),
  submitMyAssignment: (input: {
    assignment_id: string;
    body: string;
    file_name?: string;
    file_data?: string;
  }) => invoke("submit_my_assignment", { input }),
  getAutoBackupStatus: () => invoke<AutoBackupStatus>("get_auto_backup_status"),
  setAutoBackupSettings: (input: AutoBackupSettings) =>
    invoke<void>("set_auto_backup_settings", { input }),
  runAutoBackupNow: () => invoke<AutoBackupEntry>("run_auto_backup_now"),
  listAutoBackups: () => invoke<AutoBackupEntry[]>("list_auto_backups"),
  restoreAutoBackup: (filename: string) =>
    invoke<void>("restore_auto_backup", { filename }),
  listWhatsAppGroups: (courseId?: string) =>
    invoke<WhatsAppGroup[]>("list_whatsapp_groups", { courseId: courseId ?? null }),
  createWhatsAppGroup: (input: CreateWhatsAppGroupInput) =>
    invoke<WhatsAppGroup>("create_whatsapp_group", { input }),
  deleteWhatsAppGroup: (groupId: string) =>
    invoke<void>("delete_whatsapp_group", { groupId }),
  listWhatsAppGroupMembers: (groupId: string) =>
    invoke<WhatsAppGroupMember[]>("list_whatsapp_group_members", { groupId }),
  addWhatsAppGroupMember: (groupId: string, userId: string) =>
    invoke<void>("add_whatsapp_group_member", { groupId, userId }),
  removeWhatsAppGroupMember: (groupId: string, userId: string) =>
    invoke<void>("remove_whatsapp_group_member", { groupId, userId }),
  syncWhatsAppGroupMembers: (groupId: string) =>
    invoke<number>("sync_whatsapp_group_members", { groupId }),
  updateUserPhone: (userId: string, phone?: string | null) =>
    invoke<void>("update_user_phone", { input: { user_id: userId, phone: phone ?? null } }),
  buildWhatsAppShare: (input: WhatsAppShareInput) =>
    invoke<WhatsAppSharePlan>("build_whatsapp_share", { input }),
  getWhatsAppBusinessSettings: () =>
    invoke<WhatsAppBusinessSettings>("get_whatsapp_business_settings"),
  saveWhatsAppBusinessSettings: (input: SaveWhatsAppBusinessSettingsInput) =>
    invoke<WhatsAppBusinessSettings>("save_whatsapp_business_settings", { input }),
  testWhatsAppBusinessConnection: () =>
    invoke<WhatsAppConnectionTest>("test_whatsapp_business_connection"),
  linkWhatsAppGroup: (input: LinkWhatsAppGroupInput) =>
    invoke<WhatsAppGroupLink>("link_whatsapp_group", { input }),
  unlinkWhatsAppGroup: (groupId: string) =>
    invoke<void>("unlink_whatsapp_group", { groupId }),
  getWhatsAppGroupLink: (groupId: string) =>
    invoke<WhatsAppGroupLink | null>("get_whatsapp_group_link", { groupId }),
  createNativeWhatsAppGroup: (input: CreateNativeWhatsAppGroupInput) =>
    invoke<NativeWhatsAppGroupResult>("create_native_whatsapp_group", { input }),
  refreshNativeWhatsAppGroup: (groupId: string) =>
    invoke<NativeWhatsAppGroupResult>("refresh_native_whatsapp_group", { groupId }),
  sendWhatsAppGroupInvites: (input: SendWhatsAppGroupInvitesInput) =>
    invoke<WhatsAppBroadcastResult>("send_whatsapp_group_invites", { input }),
  setWhatsAppConsent: (userId: string, optedIn: boolean) =>
    invoke<void>("set_whatsapp_consent", { userId, optedIn }),
  sendWhatsAppBroadcast: (input: SendWhatsAppBroadcastInput) =>
    invoke<WhatsAppBroadcastResult>("send_whatsapp_broadcast", { input }),
  listWhatsAppOutboundMessages: (groupId?: string, limit?: number) =>
    invoke<WhatsAppOutboundMessage[]>("list_whatsapp_outbound_messages", {
      groupId: groupId ?? null,
      limit: limit ?? null,
    }),
  listWhatsAppBroadcastSummaries: (groupId: string, limit?: number) =>
    invoke<WhatsAppBroadcastSummary[]>("list_whatsapp_broadcast_summaries", {
      groupId,
      limit: limit ?? null,
    }),
  listWhatsAppMessageStatusEvents: (groupId: string, batchKey?: string, limit?: number) =>
    invoke<WhatsAppMessageStatusEvent[]>("list_whatsapp_message_status_events", {
      groupId,
      batchKey: batchKey ?? null,
      limit: limit ?? null,
    }),
  getWhatsAppTemplateSettings: () =>
    invoke<WhatsAppTemplateSettings>("get_whatsapp_template_settings"),
  saveWhatsAppTemplateSettings: (input: SaveWhatsAppTemplateSettingsInput) =>
    invoke<WhatsAppTemplateSettings>("save_whatsapp_template_settings", { input }),
  previewWhatsAppAssignmentTemplate: (assignmentId: string) =>
    invoke<WhatsAppTemplatePreview>("preview_whatsapp_assignment_template", { assignmentId }),
  sendWhatsAppTemplateBroadcast: (input: SendWhatsAppTemplateBroadcastInput) =>
    invoke<WhatsAppBroadcastResult>("send_whatsapp_template_broadcast", { input }),
  getWhatsAppInboundRoutingSettings: () =>
    invoke<WhatsAppInboundRoutingSettings>("get_whatsapp_inbound_routing_settings"),
  setWhatsAppInboundRoutingSettings: (input: SaveWhatsAppInboundRoutingSettingsInput) =>
    invoke<WhatsAppInboundRoutingSettings>("set_whatsapp_inbound_routing_settings", { input }),
  listWhatsAppInboundMessages: (status?: string, limit?: number) =>
    invoke<WhatsAppInboundMessage[]>("list_whatsapp_inbound_messages", {
      status: status ?? null,
      limit: limit ?? null,
    }),
  routeWhatsAppInboundMessage: (inboundId: string) =>
    invoke<WhatsAppInboundMessage>("route_whatsapp_inbound_message", { inboundId }),
  ignoreWhatsAppInboundMessage: (inboundId: string) =>
    invoke<void>("ignore_whatsapp_inbound_message", { inboundId }),
  createWhatsAppScheduledBroadcast: (input: CreateWhatsAppScheduledBroadcastInput) =>
    invoke<WhatsAppScheduledBroadcast>("create_whatsapp_scheduled_broadcast", { input }),
  listWhatsAppScheduledBroadcasts: (groupId?: string, limit?: number) =>
    invoke<WhatsAppScheduledBroadcast[]>("list_whatsapp_scheduled_broadcasts", {
      groupId: groupId ?? null,
      limit: limit ?? null,
    }),
  cancelWhatsAppScheduledBroadcast: (id: string) =>
    invoke<void>("cancel_whatsapp_scheduled_broadcast", { id }),
  runDueWhatsAppScheduledBroadcasts: () =>
    invoke<WhatsAppScheduledRunResult>("run_due_whatsapp_scheduled_broadcasts"),
  getWhatsAppComplianceSettings: () =>
    invoke<WhatsAppComplianceSettings>("get_whatsapp_compliance_settings"),
  saveWhatsAppComplianceSettings: (input: SaveWhatsAppComplianceSettingsInput) =>
    invoke<WhatsAppComplianceSettings>("save_whatsapp_compliance_settings", { input }),
  exportWhatsAppGdpr: (userId: string) =>
    invoke<WhatsAppGdprExport>("export_whatsapp_gdpr", { userId }),
  listWhatsAppConsentLog: (userId?: string, limit?: number) =>
    invoke<WhatsAppConsentLogEntry[]>("list_whatsapp_consent_log", {
      userId: userId ?? null,
      limit: limit ?? null,
    }),
  getWhatsAppGroupRosterDiff: (groupId: string) =>
    invoke<WhatsAppGroupRosterDiff>("get_whatsapp_group_roster_diff", { groupId }),
  syncNativeWhatsAppGroupRoster: (input: SyncNativeWhatsAppGroupRosterInput) =>
    invoke<SyncNativeWhatsAppGroupRosterResult>("sync_native_whatsapp_group_roster", { input }),
  listWhatsAppGroupJoinRequests: (groupId: string) =>
    invoke<WhatsAppJoinRequest[]>("list_whatsapp_group_join_requests", { groupId }),
  approveWhatsAppGroupJoinRequests: (input: ManageWhatsAppJoinRequestsInput) =>
    invoke<WhatsAppBroadcastResult>("approve_whatsapp_group_join_requests", { input }),
  rejectWhatsAppGroupJoinRequests: (input: ManageWhatsAppJoinRequestsInput) =>
    invoke<WhatsAppBroadcastResult>("reject_whatsapp_group_join_requests", { input }),
  listWhatsAppGroupParticipantEvents: (groupId: string, limit?: number) =>
    invoke<WhatsAppGroupParticipantEvent[]>("list_whatsapp_group_participant_events", {
      groupId,
      limit: limit ?? null,
    }),
  listWhatsAppGroupSettingsEvents: (groupId: string, limit?: number) =>
    invoke<WhatsAppGroupSettingsEvent[]>("list_whatsapp_group_settings_events", {
      groupId,
      limit: limit ?? null,
    }),
  getCashbookSettings: () => invoke<CashbookSettings>("get_cashbook_settings"),
  saveCashbookSettings: (input: SaveCashbookSettingsInput) =>
    invoke<CashbookSettings>("save_cashbook_settings", { input }),
  testInvoiceNinjaConnection: () =>
    invoke<CashbookIntegrationTest>("test_invoice_ninja_connection"),
  listCashbookEntries: (fromDate?: string, toDate?: string) =>
    invoke<CashbookEntry[]>("list_cashbook_entries", {
      fromDate: fromDate ?? null,
      toDate: toDate ?? null,
    }),
  createCashbookEntry: (input: CreateCashbookEntryInput) =>
    invoke<CashbookEntry>("create_cashbook_entry", { input }),
  deleteCashbookEntry: (id: string) => invoke<void>("delete_cashbook_entry", { id }),
  getCashbookSummary: (fromDate?: string, toDate?: string) =>
    invoke<CashbookSummary>("get_cashbook_summary", {
      fromDate: fromDate ?? null,
      toDate: toDate ?? null,
    }),
  exportCashbookCsv: (fromDate?: string, toDate?: string) =>
    invoke<string>("export_cashbook_csv", { fromDate: fromDate ?? null, toDate: toDate ?? null }),
  getTenancyContext: () => invoke<TenancyContext>("get_tenancy_context"),
  setActiveSchool: (schoolId: string) =>
    invoke<TenancyContext>("set_active_school", { schoolId }),
  createSchool: (input: CreateSchoolInput) => invoke<School>("create_school", { input }),
  updateSchool: (input: UpdateSchoolInput) => invoke<School>("update_school", { input }),
  listSchoolMembers: (schoolId: string) =>
    invoke<SchoolMember[]>("list_school_members", { schoolId }),
  addSchoolMember: (input: AddSchoolMemberInput) =>
    invoke<SchoolMember>("add_school_member", { input }),
  removeSchoolMember: (membershipId: string) =>
    invoke<void>("remove_school_member", { membershipId }),
  getPushSettings: () => invoke<PushSettings>("get_push_settings"),
  savePushSettings: (input: SavePushSettingsInput) =>
    invoke<PushSettings>("save_push_settings", { input }),
  getPushReminderStatus: () => invoke<PushReminderStatus>("get_push_reminder_status"),
  setPushReminderSettings: (enabled: boolean) =>
    invoke<void>("set_push_reminder_settings", { enabled }),
  runPushRemindersNow: () => invoke<PushReminderRunResult>("run_push_reminders_now"),
  testPushNotification: (input: TestPushInput) =>
    invoke<string>("test_push_notification", { input }),
  listPushLog: (limit?: number) => invoke<PushLogEntry[]>("list_push_log", { limit: limit ?? null }),
};
