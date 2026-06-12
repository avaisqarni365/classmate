export interface User {
  id: string;
  email: string;
  name: string;
  role: string;
  phone?: string | null;
  created_at: string;
}

export interface Course {
  id: string;
  title: string;
  code: string;
  description?: string | null;
  teacher_id?: string | null;
  teacher_name?: string | null;
  term?: string | null;
  student_count: number;
  created_at: string;
}

export interface Assignment {
  id: string;
  course_id: string;
  title: string;
  description?: string | null;
  due_at?: string | null;
  max_points: number;
  created_at: string;
}

export interface DashboardStats {
  user_count: number;
  course_count: number;
  student_count: number;
  assignment_count: number;
  active_sessions: number;
}

export interface CreateUserInput {
  email: string;
  name: string;
  role: string;
  password: string;
}

export interface CreateCourseInput {
  title: string;
  code: string;
  description?: string;
  teacher_id?: string;
  term?: string;
}

export interface LoginInput {
  email: string;
  password: string;
}

export interface CreateAssignmentInput {
  course_id: string;
  title: string;
  description?: string;
  due_at?: string;
  max_points: number;
}

export interface GradeCell {
  assignment_id: string;
  points?: number | null;
  feedback?: string | null;
  graded_at?: string | null;
}

export interface GradebookStudent {
  student_id: string;
  student_name: string;
  grades: Record<string, GradeCell>;
}

export interface Gradebook {
  course_id: string;
  course_title: string;
  assignments: Assignment[];
  students: GradebookStudent[];
}

export interface Enrollment {
  id: string;
  course_id: string;
  student_id: string;
  student_name: string;
  student_email: string;
  status: string;
  enrolled_at: string;
}

export interface CourseMaterial {
  id: string;
  course_id: string;
  title: string;
  kind: string;
  content: string;
  created_at: string;
}

export interface CreateMaterialInput {
  course_id: string;
  title: string;
  kind: string;
  content: string;
}

export interface OpenStaxBook {
  slug: string;
  title: string;
  subjects: string[];
  read_url: string;
  pdf_url?: string | null;
}

export interface TextbookMaterialContent {
  provider: string;
  book_slug: string;
  book_title: string;
  subjects: string[];
  read_url: string;
  pdf_url?: string | null;
  notes?: string;
}

export interface AiLabDefinition {
  slug: string;
  name: string;
  description: string;
  tools: string[];
}

export interface MaterialAiLab {
  lab: AiLabDefinition;
  url: string;
  embed_url: string;
  activities: string[];
}

export interface MaterialWithAiLab {
  material: CourseMaterial;
  ai_lab: MaterialAiLab;
  lab_completed?: boolean;
}

export interface SpeakNoteContent {
  body: string;
  outline?: string;
  transcript?: string;
}

export interface HandwritingContent {
  ink_json: string;
  preview_data_url?: string | null;
  session_id?: string;
}

export interface CaptureSession {
  id: string;
  course_id: string;
  created_by: string;
  title: string;
  ink_json: string;
  preview_data_url?: string | null;
  status: string;
  material_id?: string | null;
  expires_at: string;
  updated_at: string;
  created_at: string;
  pad_url?: string | null;
}

export interface CreateCaptureSessionInput {
  course_id: string;
  title: string;
}

export interface ArtizAiConfig {
  base_url: string;
  labs: AiLabDefinition[];
}

export interface HubStatus {
  running: boolean;
  port: number;
  local_ip?: string | null;
  join_url?: string | null;
  session_id?: string | null;
  course_title?: string | null;
  pin?: string | null;
  video_url?: string | null;
  video_running: boolean;
}

export interface Announcement {
  id: string;
  course_id?: string | null;
  title: string;
  body: string;
  author_id?: string | null;
  created_at: string;
}

export interface CreateAnnouncementInput {
  course_id?: string;
  title: string;
  body: string;
}

export interface ForumTopic {
  id: string;
  course_id: string;
  title: string;
  author_name: string;
  post_count: number;
  created_at: string;
}

export interface ForumPost {
  id: string;
  topic_id: string;
  author_name: string;
  body: string;
  created_at: string;
}

export interface CreateForumTopicInput {
  course_id: string;
  title: string;
  author_name: string;
  body: string;
}

export interface CreateForumPostInput {
  topic_id: string;
  author_name: string;
  body: string;
}

export interface LtiTool {
  id: string;
  name: string;
  launch_url: string;
  consumer_key: string;
  created_at: string;
}

export interface CreateLtiToolInput {
  name: string;
  launch_url: string;
  consumer_key: string;
  shared_secret: string;
}

export interface ParentStudentSummary {
  student_id: string;
  student_name: string;
  courses: ParentCourseSummary[];
}

export interface ParentCourseSummary {
  course_id: string;
  course_title: string;
  average_percent?: number | null;
  assignment_count: number;
}

export interface LinkParentInput {
  parent_id: string;
  student_id: string;
}

export interface CertificateInfo {
  id: string;
  course_id: string;
  course_title: string;
  student_id: string;
  student_name: string;
  issued_at: string;
  html: string;
}

export interface VideoStatus {
  running: boolean;
  port: number;
  room: string;
  url?: string | null;
  galene_installed: boolean;
  message: string;
}

export interface ImportResult {
  imported: number;
  skipped: number;
  errors: string[];
}

export interface BackupPayload {
  version: string;
  exported_at: string;
  users: unknown[];
  courses: unknown[];
  enrollments: unknown[];
  assignments: unknown[];
  grades: unknown[];
  materials: unknown[];
  announcements: unknown[];
  settings: unknown[];
}

export interface Quiz {
  id: string;
  course_id: string;
  title: string;
  description?: string | null;
  time_limit_minutes?: number | null;
  max_points: number;
  status: string;
  question_count: number;
  attempt_count: number;
  created_at: string;
}

export interface QuizQuestion {
  id: string;
  quiz_id: string;
  prompt: string;
  kind: string;
  options: string[];
  correct_index: number;
  correct_text?: string | null;
  points: number;
  sort_order: number;
}

export interface QuizDetail {
  quiz: Quiz;
  questions: QuizQuestion[];
}

export interface QuizAttempt {
  id: string;
  quiz_id: string;
  student_name: string;
  score: number;
  max_score: number;
  review_status: string;
  feedback?: string | null;
  submitted_at: string;
}

export interface QuizAttemptAnswer {
  question_id: string;
  selected_index?: number | null;
  text_answer?: string | null;
}

export interface QuizAttemptDetail {
  attempt: QuizAttempt;
  questions: QuizQuestion[];
  answers: QuizAttemptAnswer[];
}

export interface GradeQuizAttemptInput {
  attempt_id: string;
  score: number;
  feedback?: string;
}

export interface CreateQuizInput {
  course_id: string;
  title: string;
  description?: string;
  time_limit_minutes?: number;
}

export interface CreateQuizQuestionInput {
  quiz_id: string;
  prompt: string;
  kind?: string;
  options: string[];
  correct_index: number;
  correct_text?: string;
  points: number;
}

export interface AtRiskStudent {
  student_id: string;
  student_name: string;
  course_id: string;
  course_title: string;
  average_percent: number;
}

export interface CourseAnalytics {
  course_id: string;
  course_title: string;
  student_count: number;
  average_percent?: number | null;
  attendance_rate?: number | null;
}

export interface SessionSummary {
  session_id: string;
  course_title: string;
  title: string;
  attendance_count: number;
  started_at?: string | null;
}

export interface AnalyticsReport {
  at_risk_students: AtRiskStudent[];
  course_summaries: CourseAnalytics[];
  recent_sessions: SessionSummary[];
  total_quiz_attempts: number;
}

export interface ScheduleSlot {
  id: string;
  course_id: string;
  course_title: string;
  course_code: string;
  day_of_week: number;
  start_time: string;
  end_time: string;
  room?: string | null;
  title?: string | null;
}

export interface CreateScheduleSlotInput {
  course_id: string;
  day_of_week: number;
  start_time: string;
  end_time: string;
  room?: string;
  title?: string;
}

export interface SessionPoll {
  id: string;
  session_id: string;
  question: string;
  options: string[];
  status: string;
  created_at: string;
}

export interface PollResults {
  poll: SessionPoll;
  vote_counts: number[];
  total_votes: number;
}

export interface CreateSessionPollInput {
  session_id: string;
  question: string;
  options: string[];
}

export interface AssignmentSubmission {
  id: string;
  assignment_id: string;
  assignment_title: string;
  student_name: string;
  student_id?: string | null;
  body: string;
  file_name?: string | null;
  file_data?: string | null;
  points?: number | null;
  feedback?: string | null;
  status: string;
  submitted_at: string;
  graded_at?: string | null;
}

export interface SubmitAssignmentInput {
  assignment_id: string;
  student_name: string;
  body: string;
  file_name?: string;
  file_data?: string;
}

export interface GradeSubmissionInput {
  submission_id: string;
  points: number;
  feedback?: string;
  rubric_scores?: RubricScoreInput[];
}

export interface RubricScoreInput {
  criterion_id: string;
  points: number;
}

export interface RubricCriterion {
  id: string;
  name: string;
  description?: string | null;
  max_points: number;
}

export interface RubricCriterionInput {
  id?: string;
  name: string;
  description?: string;
  max_points: number;
}

export interface AssignmentRubric {
  assignment_id: string;
  criteria: RubricCriterion[];
  updated_at: string;
}

export interface SaveAssignmentRubricInput {
  assignment_id: string;
  criteria: RubricCriterionInput[];
}

export interface ParentDigest {
  parent_name: string;
  generated_at: string;
  html: string;
}

export interface UiPreferences {
  school_name: string;
  theme: string;
  font_scale: string;
  accent_color: string;
  locale: string;
}

export interface SyncServerStatus {
  running: boolean;
  port: number;
  local_ip?: string | null;
  sync_url?: string | null;
  sync_token: string;
  public_base_url?: string | null;
  webhook_url?: string | null;
  hub_join_url?: string | null;
}

export interface HelpInfo {
  app_version: string;
  public_base_url?: string | null;
  public_hub_path?: string | null;
  webhook_url?: string | null;
  hub_join_url?: string | null;
  download_web_url?: string | null;
  sync_running: boolean;
  github_url: string;
  windows_installer_available: boolean;
}

export interface SyncPeerResult {
  message: string;
  exported_at: string;
}

export interface ClassSessionRecord {
  id: string;
  course_id: string;
  course_title: string;
  title: string;
  status: string;
  pin: string;
  started_at?: string | null;
  ended_at?: string | null;
  attendance_count: number;
}

export interface AutoBackupSettings {
  enabled: boolean;
  interval: string;
  max_keep: number;
}

export interface AutoBackupEntry {
  filename: string;
  exported_at: string;
  size_bytes: number;
}

export interface AutoBackupStatus {
  settings: AutoBackupSettings;
  last_backup_at?: string | null;
  next_due_at?: string | null;
  backup_count: number;
  backup_dir: string;
}

export interface SubmitMyAssignmentInput {
  assignment_id: string;
  body: string;
}

export interface StudentCourseSummary {
  course_id: string;
  title: string;
  code: string;
  teacher_name?: string | null;
  average_percent?: number | null;
  assignment_count: number;
  graded_count: number;
}

export interface StudentAssignmentGrade {
  assignment_id: string;
  course_id: string;
  course_title: string;
  title: string;
  due_at?: string | null;
  max_points: number;
  points?: number | null;
  feedback?: string | null;
  rubric_scores?: RubricScoreDisplay[];
}

export interface RubricScoreDisplay {
  criterion_id: string;
  name: string;
  max_points: number;
  points: number;
}

export interface ParentGradeEntry {
  student_id: string;
  student_name: string;
  course_title: string;
  assignment_id: string;
  assignment_title: string;
  points?: number | null;
  max_points: number;
  feedback?: string | null;
  graded_at?: string | null;
  rubric_scores?: RubricScoreDisplay[];
}

export interface SmtpSettings {
  configured: boolean;
  host: string;
  port: number;
  username: string;
  from: string;
  password_set: boolean;
  use_tls: boolean;
}

export interface SaveSmtpSettingsInput {
  host: string;
  port: number;
  username: string;
  from: string;
  use_tls: boolean;
  password?: string;
}

export interface SmtpConnectionTest {
  ok: boolean;
  message: string;
}

export interface SendParentDigestEmailInput {
  parent_id: string;
  to_email?: string;
}

export interface SendParentDigestEmailResult {
  recipient: string;
  subject: string;
  message: string;
}

export interface EmailLogEntry {
  id: string;
  recipient: string;
  subject: string;
  kind: string;
  status: string;
  error?: string | null;
  created_at: string;
}

export interface DigestEmailSettings {
  enabled: boolean;
  interval: string;
}

export interface DigestEmailStatus {
  settings: DigestEmailSettings;
  smtp_configured: boolean;
  parent_count: number;
  last_run_at?: string | null;
  next_due_at?: string | null;
}

export interface DigestEmailRunResult {
  sent: number;
  failed: number;
  skipped: number;
  errors: string[];
}

export interface StudentDashboard {
  courses: StudentCourseSummary[];
  upcoming: StudentAssignmentGrade[];
}

export interface StudentCourseDetail {
  course: StudentCourseSummary;
  assignments: StudentAssignmentGrade[];
  materials: CourseMaterial[];
}

export interface WhatsAppGroup {
  id: string;
  course_id: string;
  course_title: string;
  name: string;
  kind: string;
  member_count: number;
  created_at: string;
  invite_link?: string | null;
  external_name?: string | null;
  external_group_id?: string | null;
  native_status?: string | null;
  creation_error?: string | null;
  group_description?: string | null;
}

export interface WhatsAppGroupMember {
  user_id: string;
  name: string;
  phone?: string | null;
  role: string;
  opted_in?: boolean;
}

export interface CreateWhatsAppGroupInput {
  course_id: string;
  name: string;
  kind: string;
}

export interface WhatsAppShareInput {
  kind: string;
  assignment_id?: string;
  announcement_id?: string;
  group_id?: string;
  custom_message?: string;
}

export interface WhatsAppShareRecipient {
  user_id: string;
  name: string;
  phone?: string | null;
  wa_url?: string | null;
}

export interface WhatsAppSharePlan {
  message: string;
  group_paste: string;
  recipients: WhatsAppShareRecipient[];
  missing_phones: string[];
}

export interface WhatsAppBusinessSettings {
  configured: boolean;
  api_version: string;
  phone_number_id: string;
  access_token_set: boolean;
  webhook_verify_token_set: boolean;
  webhook_url?: string | null;
}

export interface SaveWhatsAppBusinessSettingsInput {
  phone_number_id: string;
  api_version?: string;
  access_token?: string;
  webhook_verify_token?: string;
}

export interface WhatsAppGroupLink {
  group_id: string;
  invite_link: string;
  external_name?: string | null;
  linked_at: string;
  external_group_id?: string | null;
  native_status?: string | null;
  creation_error?: string | null;
  group_description?: string | null;
}

export interface CreateNativeWhatsAppGroupInput {
  group_id: string;
  description?: string;
  join_approval_mode?: string;
}

export interface NativeWhatsAppGroupResult {
  group_id: string;
  external_group_id?: string | null;
  invite_link?: string | null;
  native_status: string;
  message: string;
}

export interface SendWhatsAppGroupInvitesInput {
  group_id: string;
}

export interface LinkWhatsAppGroupInput {
  group_id: string;
  invite_link: string;
  external_name?: string;
}

export interface SendWhatsAppBroadcastInput {
  kind: string;
  group_id: string;
  assignment_id?: string;
  announcement_id?: string;
  custom_message?: string;
}

export interface WhatsAppBroadcastResult {
  sent: number;
  failed: number;
  skipped: number;
  errors: string[];
}

export interface WhatsAppConnectionTest {
  ok: boolean;
  display_phone_number?: string | null;
  verified_name?: string | null;
  message: string;
}

export interface WhatsAppOutboundMessage {
  id: string;
  group_id?: string | null;
  user_id?: string | null;
  user_name?: string | null;
  kind: string;
  ref_id?: string | null;
  phone: string;
  body: string;
  status: string;
  wa_message_id?: string | null;
  error?: string | null;
  created_at: string;
  sent_at?: string | null;
  delivered_at?: string | null;
  read_at?: string | null;
}

export interface WhatsAppBroadcastSummary {
  batch_key: string;
  broadcast_batch_id?: string | null;
  group_id: string;
  kind: string;
  ref_id?: string | null;
  started_at: string;
  last_update_at: string;
  total: number;
  pending: number;
  sent: number;
  delivered: number;
  read: number;
  failed: number;
}

export interface WhatsAppMessageStatusEvent {
  id: string;
  outbound_id?: string | null;
  wa_message_id: string;
  group_id?: string | null;
  status: string;
  event_at: string;
  received_at: string;
}

export interface WhatsAppTemplateSettings {
  assignment_name: string;
  assignment_language: string;
  assignment_configured: boolean;
  group_invite_name: string;
  group_invite_language: string;
  group_invite_configured: boolean;
}

export interface SaveWhatsAppTemplateSettingsInput {
  assignment_name: string;
  assignment_language: string;
  group_invite_name?: string;
  group_invite_language?: string;
}

export interface SendWhatsAppTemplateBroadcastInput {
  group_id: string;
  assignment_id: string;
}

export interface WhatsAppTemplatePreview {
  template_name: string;
  language: string;
  parameters: string[];
  parameter_labels: string[];
}

export interface WhatsAppInboundMessage {
  id: string;
  wa_message_id: string;
  from_phone: string;
  from_user_id?: string | null;
  from_user_name?: string | null;
  body: string;
  status: string;
  routed_topic_id?: string | null;
  routed_post_id?: string | null;
  received_at: string;
}

export interface WhatsAppInboundRoutingSettings {
  enabled: boolean;
  course_id: string;
  topic_id?: string | null;
  topic_title?: string | null;
  pending_count: number;
}

export interface SaveWhatsAppInboundRoutingSettingsInput {
  enabled: boolean;
  course_id: string;
  topic_id?: string;
}

export interface WhatsAppScheduledBroadcast {
  id: string;
  group_id: string;
  group_name?: string | null;
  broadcast_kind: string;
  kind?: string | null;
  assignment_id?: string | null;
  announcement_id?: string | null;
  custom_message?: string | null;
  scheduled_at: string;
  status: string;
  sent_at?: string | null;
  result_sent?: number | null;
  result_failed?: number | null;
  result_skipped?: number | null;
  error?: string | null;
  created_at: string;
}

export interface CreateWhatsAppScheduledBroadcastInput {
  broadcast_kind: string;
  group_id: string;
  scheduled_at: string;
  kind?: string;
  assignment_id?: string;
  announcement_id?: string;
  custom_message?: string;
}

export interface WhatsAppScheduledRunResult {
  processed: number;
  sent: number;
  failed: number;
}

export interface WhatsAppConsentLogEntry {
  id: string;
  user_id: string;
  opted_in: boolean;
  source: string;
  note?: string | null;
  created_at: string;
}

export interface WhatsAppConsentSnapshot {
  opted_in: boolean;
  opted_in_at?: string | null;
  source?: string | null;
}

export interface WhatsAppGdprExport {
  exported_at: string;
  user_id: string;
  user_name: string;
  email: string;
  phone?: string | null;
  consent?: WhatsAppConsentSnapshot | null;
  consent_log: WhatsAppConsentLogEntry[];
  groups: string[];
  outbound_messages: WhatsAppOutboundMessage[];
  inbound_messages: WhatsAppInboundMessage[];
}

export interface WhatsAppComplianceSettings {
  auto_unsubscribe: boolean;
  unsubscribe_keywords: string;
}

export interface SaveWhatsAppComplianceSettingsInput {
  auto_unsubscribe: boolean;
  unsubscribe_keywords: string;
}

export interface School {
  id: string;
  name: string;
  code: string;
  created_at: string;
}

export interface SchoolSummary {
  id: string;
  name: string;
  code: string;
  member_count: number;
  course_count: number;
}

export interface TenancyContext {
  active_school_id: string;
  active_school_name: string;
  schools: SchoolSummary[];
  is_org_admin: boolean;
}

export interface CreateSchoolInput {
  name: string;
  code: string;
}

export interface UpdateSchoolInput {
  id: string;
  name: string;
  code: string;
}

export interface SchoolMember {
  id: string;
  school_id: string;
  user_id: string;
  user_name: string;
  user_email: string;
  user_role: string;
  created_at: string;
}

export interface AddSchoolMemberInput {
  school_id: string;
  user_email: string;
}

export interface PushSettings {
  enabled: boolean;
  fcm_configured: boolean;
  fcm_project_id: string;
  fcm_service_account_set: boolean;
  apns_configured: boolean;
  apns_key_id: string;
  apns_team_id: string;
  apns_bundle_id: string;
  apns_private_key_set: boolean;
  apns_use_sandbox: boolean;
}

export interface SavePushSettingsInput {
  enabled: boolean;
  fcm_project_id: string;
  fcm_service_account_json?: string;
  apns_key_id: string;
  apns_team_id: string;
  apns_bundle_id: string;
  apns_private_key?: string;
  apns_use_sandbox: boolean;
}

export interface PushDevice {
  id: string;
  user_id: string;
  platform: string;
  token: string;
  device_name?: string | null;
  created_at: string;
  last_seen_at: string;
}

export interface TestPushInput {
  platform: string;
  token: string;
  title: string;
  body: string;
}

export interface PushLogEntry {
  id: string;
  user_id?: string | null;
  platform: string;
  token: string;
  title: string;
  body: string;
  status: string;
  provider_response?: string | null;
  created_at: string;
}

export interface PushReminderStatus {
  push_configured: boolean;
  enabled: boolean;
  last_run_at?: string | null;
  device_count: number;
}

export interface PushReminderRunResult {
  sent: number;
  failed: number;
  skipped: number;
  errors: string[];
}

export interface CashbookEntry {
  id: string;
  school_id: string;
  direction: string;
  category: string;
  amount: number;
  currency: string;
  description?: string | null;
  user_id?: string | null;
  user_name?: string | null;
  course_id?: string | null;
  course_title?: string | null;
  payment_method: string;
  reference?: string | null;
  entry_date: string;
  created_by?: string | null;
  created_by_name?: string | null;
  created_at: string;
}

export interface CreateCashbookEntryInput {
  direction: string;
  category: string;
  amount: number;
  currency?: string;
  description?: string;
  user_id?: string;
  course_id?: string;
  payment_method: string;
  reference?: string;
  entry_date?: string;
}

export interface CashbookSummary {
  currency: string;
  total_income: number;
  total_expense: number;
  balance: number;
  student_fees: number;
  teacher_salary: number;
  entry_count: number;
}

export interface CashbookSettings {
  currency: string;
  invoice_ninja_url: string;
  invoice_ninja_configured: boolean;
}

export interface SaveCashbookSettingsInput {
  currency: string;
  invoice_ninja_url?: string;
  invoice_ninja_token?: string;
}

export interface CashbookIntegrationTest {
  ok: boolean;
  message: string;
}

export interface WhatsAppRosterMember {
  user_id: string;
  name: string;
  phone?: string | null;
  normalized_phone?: string | null;
}

export interface WhatsAppNativeParticipant {
  wa_id: string;
}

export interface WhatsAppGroupRosterDiff {
  group_id: string;
  native_available: boolean;
  classmate_members: WhatsAppRosterMember[];
  whatsapp_participants: WhatsAppNativeParticipant[];
  matched_count: number;
  only_in_classmate: WhatsAppRosterMember[];
  only_in_whatsapp: WhatsAppNativeParticipant[];
  message: string;
}

export interface WhatsAppJoinRequest {
  join_request_id: string;
  wa_id: string;
  creation_timestamp?: string | null;
}

export interface SyncNativeWhatsAppGroupRosterInput {
  group_id: string;
  send_invites?: boolean;
  remove_orphans?: boolean;
}

export interface SyncNativeWhatsAppGroupRosterResult {
  diff: WhatsAppGroupRosterDiff;
  invites: WhatsAppBroadcastResult;
  removed: number;
  remove_errors: string[];
  message: string;
}

export interface ManageWhatsAppJoinRequestsInput {
  group_id: string;
  join_request_ids: string[];
}

export interface WhatsAppGroupParticipantEvent {
  id: string;
  group_id?: string | null;
  external_group_id: string;
  event_type: string;
  direction?: string | null;
  wa_id?: string | null;
  reason?: string | null;
  join_request_id?: string | null;
  received_at: string;
}

export interface WhatsAppGroupSettingsEvent {
  id: string;
  group_id?: string | null;
  external_group_id: string;
  event_type: string;
  setting_kind?: string | null;
  setting_value?: string | null;
  update_successful?: boolean | null;
  error_summary?: string | null;
  received_at: string;
}
