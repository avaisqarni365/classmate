use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub phone: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: String,
    pub title: String,
    pub code: String,
    pub description: Option<String>,
    pub teacher_id: Option<String>,
    pub teacher_name: Option<String>,
    pub term: Option<String>,
    pub student_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<String>,
    pub max_points: f64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub user_count: i64,
    pub course_count: i64,
    pub student_count: i64,
    pub assignment_count: i64,
    pub active_sessions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub email: String,
    pub name: String,
    pub role: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCourseInput {
    pub title: String,
    pub code: String,
    pub description: Option<String>,
    pub teacher_id: Option<String>,
    pub term: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct School {
    pub id: String,
    pub name: String,
    pub code: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolSummary {
    pub id: String,
    pub name: String,
    pub code: String,
    pub member_count: i64,
    pub course_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenancyContext {
    pub active_school_id: String,
    pub active_school_name: String,
    pub schools: Vec<SchoolSummary>,
    pub is_org_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSchoolInput {
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSchoolInput {
    pub id: String,
    pub name: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchoolMember {
    pub id: String,
    pub school_id: String,
    pub user_id: String,
    pub user_name: String,
    pub user_email: String,
    pub user_role: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSchoolMemberInput {
    pub school_id: String,
    pub user_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushSettings {
    pub enabled: bool,
    pub fcm_configured: bool,
    pub fcm_project_id: String,
    pub fcm_service_account_set: bool,
    pub apns_configured: bool,
    pub apns_key_id: String,
    pub apns_team_id: String,
    pub apns_bundle_id: String,
    pub apns_private_key_set: bool,
    pub apns_use_sandbox: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePushSettingsInput {
    pub enabled: bool,
    pub fcm_project_id: String,
    pub fcm_service_account_json: Option<String>,
    pub apns_key_id: String,
    pub apns_team_id: String,
    pub apns_bundle_id: String,
    pub apns_private_key: Option<String>,
    pub apns_use_sandbox: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushDevice {
    pub id: String,
    pub user_id: String,
    pub platform: String,
    pub token: String,
    pub device_name: Option<String>,
    pub created_at: String,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterPushDeviceInput {
    pub platform: String,
    pub token: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestPushInput {
    pub platform: String,
    pub token: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPushInput {
    pub user_id: String,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPushResult {
    pub sent: i64,
    pub failed: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushLogEntry {
    pub id: String,
    pub user_id: Option<String>,
    pub platform: String,
    pub token: String,
    pub title: String,
    pub body: String,
    pub status: String,
    pub provider_response: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushReminderStatus {
    pub push_configured: bool,
    pub enabled: bool,
    pub last_run_at: Option<String>,
    pub device_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushReminderRunResult {
    pub sent: i64,
    pub failed: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginInput {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssignmentInput {
    pub course_id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<String>,
    pub max_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGradeInput {
    pub assignment_id: String,
    pub student_id: String,
    pub points: Option<f64>,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeCell {
    pub assignment_id: String,
    pub points: Option<f64>,
    pub feedback: Option<String>,
    pub graded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradebookStudent {
    pub student_id: String,
    pub student_name: String,
    pub grades: HashMap<String, GradeCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradebook {
    pub course_id: String,
    pub course_title: String,
    pub assignments: Vec<Assignment>,
    pub students: Vec<GradebookStudent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Enrollment {
    pub id: String,
    pub course_id: String,
    pub student_id: String,
    pub student_name: String,
    pub student_email: String,
    pub status: String,
    pub enrolled_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrollStudentInput {
    pub course_id: String,
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseMaterial {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub kind: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMaterialInput {
    pub course_id: String,
    pub title: String,
    pub kind: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HubStatus {
    pub running: bool,
    pub port: u16,
    pub local_ip: Option<String>,
    pub join_url: Option<String>,
    pub session_id: Option<String>,
    pub course_title: Option<String>,
    pub pin: Option<String>,
    pub video_url: Option<String>,
    pub video_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartHubInput {
    pub course_id: String,
    pub title: Option<String>,
    pub enable_video: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentJoinInfo {
    pub session_id: String,
    pub course_title: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentAssignmentView {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub due_at: Option<String>,
    pub max_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentMaterialView {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Announcement {
    pub id: String,
    pub course_id: Option<String>,
    pub title: String,
    pub body: String,
    pub author_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAnnouncementInput {
    pub course_id: Option<String>,
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumTopic {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub author_name: String,
    pub post_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForumPost {
    pub id: String,
    pub topic_id: String,
    pub author_name: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateForumTopicInput {
    pub course_id: String,
    pub title: String,
    pub author_name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateForumPostInput {
    pub topic_id: String,
    pub author_name: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LtiTool {
    pub id: String,
    pub name: String,
    pub launch_url: String,
    pub consumer_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLtiToolInput {
    pub name: String,
    pub launch_url: String,
    pub consumer_key: String,
    pub shared_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentStudentSummary {
    pub student_id: String,
    pub student_name: String,
    pub courses: Vec<ParentCourseSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentCourseSummary {
    pub course_id: String,
    pub course_title: String,
    pub average_percent: Option<f64>,
    pub assignment_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkParentInput {
    pub parent_id: String,
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub id: String,
    pub course_id: String,
    pub course_title: String,
    pub student_id: String,
    pub student_name: String,
    pub issued_at: String,
    pub html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCertificateInput {
    pub course_id: String,
    pub student_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStatus {
    pub running: bool,
    pub port: u16,
    pub room: String,
    pub url: Option<String>,
    pub galene_installed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub imported: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    pub version: String,
    pub exported_at: String,
    pub users: Vec<serde_json::Value>,
    pub courses: Vec<serde_json::Value>,
    pub enrollments: Vec<serde_json::Value>,
    pub assignments: Vec<serde_json::Value>,
    pub grades: Vec<serde_json::Value>,
    pub materials: Vec<serde_json::Value>,
    pub announcements: Vec<serde_json::Value>,
    pub settings: Vec<serde_json::Value>,
    #[serde(default)]
    pub schools: Vec<serde_json::Value>,
    #[serde(default)]
    pub school_members: Vec<serde_json::Value>,
    #[serde(default)]
    pub class_sessions: Vec<serde_json::Value>,
    #[serde(default)]
    pub attendance: Vec<serde_json::Value>,
    #[serde(default)]
    pub forum_topics: Vec<serde_json::Value>,
    #[serde(default)]
    pub forum_posts: Vec<serde_json::Value>,
    #[serde(default)]
    pub parent_links: Vec<serde_json::Value>,
    #[serde(default)]
    pub lti_tools: Vec<serde_json::Value>,
    #[serde(default)]
    pub certificates: Vec<serde_json::Value>,
    #[serde(default)]
    pub quizzes: Vec<serde_json::Value>,
    #[serde(default)]
    pub quiz_questions: Vec<serde_json::Value>,
    #[serde(default)]
    pub quiz_attempts: Vec<serde_json::Value>,
    #[serde(default)]
    pub schedule_slots: Vec<serde_json::Value>,
    #[serde(default)]
    pub session_polls: Vec<serde_json::Value>,
    #[serde(default)]
    pub session_poll_votes: Vec<serde_json::Value>,
    #[serde(default)]
    pub assignment_submissions: Vec<serde_json::Value>,
    #[serde(default)]
    pub assignment_rubrics: Vec<serde_json::Value>,
    #[serde(default)]
    pub whatsapp_groups: Vec<serde_json::Value>,
    #[serde(default)]
    pub whatsapp_group_members: Vec<serde_json::Value>,
    #[serde(default)]
    pub whatsapp_group_links: Vec<serde_json::Value>,
    #[serde(default)]
    pub whatsapp_consent: Vec<serde_json::Value>,
    #[serde(default)]
    pub whatsapp_outbound_messages: Vec<serde_json::Value>,
    #[serde(default)]
    pub whatsapp_inbound_messages: Vec<serde_json::Value>,
    #[serde(default)]
    pub whatsapp_scheduled_broadcasts: Vec<serde_json::Value>,
    #[serde(default)]
    pub whatsapp_consent_log: Vec<serde_json::Value>,
    #[serde(default)]
    pub email_log: Vec<serde_json::Value>,
    #[serde(default)]
    pub push_devices: Vec<serde_json::Value>,
    #[serde(default)]
    pub push_log: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppGroup {
    pub id: String,
    pub course_id: String,
    pub course_title: String,
    pub name: String,
    pub kind: String,
    pub member_count: i64,
    pub created_at: String,
    #[serde(default)]
    pub invite_link: Option<String>,
    #[serde(default)]
    pub external_name: Option<String>,
    #[serde(default)]
    pub external_group_id: Option<String>,
    #[serde(default)]
    pub native_status: Option<String>,
    #[serde(default)]
    pub creation_error: Option<String>,
    #[serde(default)]
    pub group_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppGroupMember {
    pub user_id: String,
    pub name: String,
    pub phone: Option<String>,
    pub role: String,
    #[serde(default)]
    pub opted_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWhatsAppGroupInput {
    pub course_id: String,
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPhoneInput {
    pub user_id: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppShareInput {
    pub kind: String,
    #[serde(default)]
    pub assignment_id: Option<String>,
    #[serde(default)]
    pub announcement_id: Option<String>,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub custom_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppShareRecipient {
    pub user_id: String,
    pub name: String,
    pub phone: Option<String>,
    pub wa_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppSharePlan {
    pub message: String,
    pub group_paste: String,
    pub recipients: Vec<WhatsAppShareRecipient>,
    pub missing_phones: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppBusinessSettings {
    pub configured: bool,
    pub api_version: String,
    pub phone_number_id: String,
    pub access_token_set: bool,
    pub webhook_verify_token_set: bool,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWhatsAppBusinessSettingsInput {
    pub phone_number_id: String,
    #[serde(default)]
    pub api_version: Option<String>,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub webhook_verify_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppGroupLink {
    pub group_id: String,
    pub invite_link: String,
    pub external_name: Option<String>,
    pub linked_at: String,
    #[serde(default)]
    pub external_group_id: Option<String>,
    #[serde(default)]
    pub native_status: Option<String>,
    #[serde(default)]
    pub creation_error: Option<String>,
    #[serde(default)]
    pub group_description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateNativeWhatsAppGroupInput {
    pub group_id: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub join_approval_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeWhatsAppGroupResult {
    pub group_id: String,
    pub external_group_id: Option<String>,
    pub invite_link: Option<String>,
    pub native_status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendWhatsAppGroupInvitesInput {
    pub group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkWhatsAppGroupInput {
    pub group_id: String,
    pub invite_link: String,
    #[serde(default)]
    pub external_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendWhatsAppBroadcastInput {
    pub kind: String,
    pub group_id: String,
    #[serde(default)]
    pub assignment_id: Option<String>,
    #[serde(default)]
    pub announcement_id: Option<String>,
    #[serde(default)]
    pub custom_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppBroadcastResult {
    pub sent: i64,
    pub failed: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConnectionTest {
    pub ok: bool,
    pub display_phone_number: Option<String>,
    pub verified_name: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppOutboundMessage {
    pub id: String,
    pub group_id: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub kind: String,
    pub ref_id: Option<String>,
    pub phone: String,
    pub body: String,
    pub status: String,
    pub wa_message_id: Option<String>,
    pub error: Option<String>,
    pub created_at: String,
    pub sent_at: Option<String>,
    pub delivered_at: Option<String>,
    pub read_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppTemplateSettings {
    pub assignment_name: String,
    pub assignment_language: String,
    pub assignment_configured: bool,
    pub group_invite_name: String,
    pub group_invite_language: String,
    pub group_invite_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWhatsAppTemplateSettingsInput {
    pub assignment_name: String,
    pub assignment_language: String,
    #[serde(default)]
    pub group_invite_name: Option<String>,
    #[serde(default)]
    pub group_invite_language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendWhatsAppTemplateBroadcastInput {
    pub group_id: String,
    pub assignment_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppTemplatePreview {
    pub template_name: String,
    pub language: String,
    pub parameters: Vec<String>,
    pub parameter_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppInboundMessage {
    pub id: String,
    pub wa_message_id: String,
    pub from_phone: String,
    pub from_user_id: Option<String>,
    pub from_user_name: Option<String>,
    pub body: String,
    pub status: String,
    pub routed_topic_id: Option<String>,
    pub routed_post_id: Option<String>,
    pub received_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppInboundRoutingSettings {
    pub enabled: bool,
    pub course_id: String,
    pub topic_id: Option<String>,
    pub topic_title: Option<String>,
    pub pending_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWhatsAppInboundRoutingSettingsInput {
    pub enabled: bool,
    pub course_id: String,
    #[serde(default)]
    pub topic_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppScheduledBroadcast {
    pub id: String,
    pub group_id: String,
    pub group_name: Option<String>,
    pub broadcast_kind: String,
    pub kind: Option<String>,
    pub assignment_id: Option<String>,
    pub announcement_id: Option<String>,
    pub custom_message: Option<String>,
    pub scheduled_at: String,
    pub status: String,
    pub sent_at: Option<String>,
    pub result_sent: Option<i64>,
    pub result_failed: Option<i64>,
    pub result_skipped: Option<i64>,
    pub error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWhatsAppScheduledBroadcastInput {
    pub broadcast_kind: String,
    pub group_id: String,
    pub scheduled_at: String,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub assignment_id: Option<String>,
    #[serde(default)]
    pub announcement_id: Option<String>,
    #[serde(default)]
    pub custom_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppScheduledRunResult {
    pub processed: i64,
    pub sent: i64,
    pub failed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConsentLogEntry {
    pub id: String,
    pub user_id: String,
    pub opted_in: bool,
    pub source: String,
    pub note: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppConsentSnapshot {
    pub opted_in: bool,
    pub opted_in_at: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppGdprExport {
    pub exported_at: String,
    pub user_id: String,
    pub user_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub consent: Option<WhatsAppConsentSnapshot>,
    pub consent_log: Vec<WhatsAppConsentLogEntry>,
    pub groups: Vec<String>,
    pub outbound_messages: Vec<WhatsAppOutboundMessage>,
    pub inbound_messages: Vec<WhatsAppInboundMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppComplianceSettings {
    pub auto_unsubscribe: bool,
    pub unsubscribe_keywords: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWhatsAppComplianceSettingsInput {
    pub auto_unsubscribe: bool,
    pub unsubscribe_keywords: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncServerStatus {
    pub running: bool,
    pub port: u16,
    pub local_ip: Option<String>,
    pub sync_url: Option<String>,
    pub sync_token: String,
    #[serde(default)]
    pub public_base_url: Option<String>,
    #[serde(default)]
    pub webhook_url: Option<String>,
    #[serde(default)]
    pub hub_join_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPeerResult {
    pub message: String,
    pub exported_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassSessionRecord {
    pub id: String,
    pub course_id: String,
    pub course_title: String,
    pub title: String,
    pub status: String,
    pub pin: String,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub attendance_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitMyAssignmentInput {
    pub assignment_id: String,
    pub body: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub file_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    pub id: String,
    pub course_id: String,
    pub title: String,
    pub description: Option<String>,
    pub time_limit_minutes: Option<i64>,
    pub max_points: f64,
    pub status: String,
    pub question_count: i64,
    pub attempt_count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub id: String,
    pub quiz_id: String,
    pub prompt: String,
    pub kind: String,
    pub options: Vec<String>,
    pub correct_index: i64,
    pub correct_text: Option<String>,
    pub points: f64,
    pub sort_order: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentQuizQuestion {
    pub id: String,
    pub prompt: String,
    pub kind: String,
    pub options: Vec<String>,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizDetail {
    pub quiz: Quiz,
    pub questions: Vec<QuizQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuizInput {
    pub course_id: String,
    pub title: String,
    pub description: Option<String>,
    pub time_limit_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuizQuestionInput {
    pub quiz_id: String,
    pub prompt: String,
    pub kind: Option<String>,
    pub options: Vec<String>,
    pub correct_index: i64,
    pub correct_text: Option<String>,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAttempt {
    pub id: String,
    pub quiz_id: String,
    pub student_name: String,
    pub score: f64,
    pub max_score: f64,
    pub review_status: String,
    pub feedback: Option<String>,
    pub submitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAttemptAnswer {
    pub question_id: String,
    #[serde(default)]
    pub selected_index: Option<i64>,
    #[serde(default)]
    pub text_answer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAttemptDetail {
    pub attempt: QuizAttempt,
    pub questions: Vec<QuizQuestion>,
    pub answers: Vec<QuizAttemptAnswer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeQuizAttemptInput {
    pub attempt_id: String,
    pub score: f64,
    pub feedback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAnswerInput {
    pub question_id: String,
    #[serde(default)]
    pub selected_index: Option<i64>,
    #[serde(default)]
    pub text_answer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitQuizInput {
    pub quiz_id: String,
    pub student_name: String,
    pub answers: Vec<QuizAnswerInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitQuizResult {
    pub score: f64,
    pub max_score: f64,
    pub percent: f64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtRiskStudent {
    pub student_id: String,
    pub student_name: String,
    pub course_id: String,
    pub course_title: String,
    pub average_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseAnalytics {
    pub course_id: String,
    pub course_title: String,
    pub student_count: i64,
    pub average_percent: Option<f64>,
    pub attendance_rate: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub course_title: String,
    pub title: String,
    pub attendance_count: i64,
    pub started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsReport {
    pub at_risk_students: Vec<AtRiskStudent>,
    pub course_summaries: Vec<CourseAnalytics>,
    pub recent_sessions: Vec<SessionSummary>,
    pub total_quiz_attempts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSlot {
    pub id: String,
    pub course_id: String,
    pub course_title: String,
    pub course_code: String,
    pub day_of_week: i64,
    pub start_time: String,
    pub end_time: String,
    pub room: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScheduleSlotInput {
    pub course_id: String,
    pub day_of_week: i64,
    pub start_time: String,
    pub end_time: String,
    pub room: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPoll {
    pub id: String,
    pub session_id: String,
    pub question: String,
    pub options: Vec<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PollResults {
    pub poll: SessionPoll,
    pub vote_counts: Vec<i64>,
    pub total_votes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionPollInput {
    pub session_id: String,
    pub question: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotePollInput {
    pub poll_id: String,
    pub student_name: String,
    pub option_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentSubmission {
    pub id: String,
    pub assignment_id: String,
    pub assignment_title: String,
    pub student_name: String,
    pub student_id: Option<String>,
    pub body: String,
    pub file_name: Option<String>,
    pub file_data: Option<String>,
    pub points: Option<f64>,
    pub feedback: Option<String>,
    pub status: String,
    pub submitted_at: String,
    pub graded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitAssignmentInput {
    pub assignment_id: String,
    pub student_name: String,
    pub body: String,
    #[serde(default)]
    pub file_name: Option<String>,
    #[serde(default)]
    pub file_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeSubmissionInput {
    pub submission_id: String,
    pub points: f64,
    pub feedback: Option<String>,
    #[serde(default)]
    pub rubric_scores: Option<Vec<RubricScoreInput>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricScoreInput {
    pub criterion_id: String,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricCriterion {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub max_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricCriterionInput {
    pub id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub max_points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentRubric {
    pub assignment_id: String,
    pub criteria: Vec<RubricCriterion>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveAssignmentRubricInput {
    pub assignment_id: String,
    pub criteria: Vec<RubricCriterionInput>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentDigest {
    pub parent_name: String,
    pub generated_at: String,
    pub html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCourseSummary {
    pub course_id: String,
    pub title: String,
    pub code: String,
    pub teacher_name: Option<String>,
    pub average_percent: Option<f64>,
    pub assignment_count: i64,
    pub graded_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentAssignmentGrade {
    pub assignment_id: String,
    pub course_id: String,
    pub course_title: String,
    pub title: String,
    pub due_at: Option<String>,
    pub max_points: f64,
    pub points: Option<f64>,
    pub feedback: Option<String>,
    #[serde(default)]
    pub rubric_scores: Option<Vec<RubricScoreDisplay>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RubricScoreDisplay {
    pub criterion_id: String,
    pub name: String,
    pub max_points: f64,
    pub points: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentGradeEntry {
    pub student_id: String,
    pub student_name: String,
    pub course_title: String,
    pub assignment_id: String,
    pub assignment_title: String,
    pub points: Option<f64>,
    pub max_points: f64,
    pub feedback: Option<String>,
    pub graded_at: Option<String>,
    #[serde(default)]
    pub rubric_scores: Option<Vec<RubricScoreDisplay>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpSettings {
    pub configured: bool,
    pub host: String,
    pub port: i64,
    pub username: String,
    pub from: String,
    pub password_set: bool,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSmtpSettingsInput {
    pub host: String,
    pub port: i64,
    pub username: String,
    pub from: String,
    pub use_tls: bool,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConnectionTest {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendParentDigestEmailInput {
    pub parent_id: String,
    #[serde(default)]
    pub to_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendParentDigestEmailResult {
    pub recipient: String,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailLogEntry {
    pub id: String,
    pub recipient: String,
    pub subject: String,
    pub kind: String,
    pub status: String,
    pub error: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestEmailSettings {
    pub enabled: bool,
    pub interval: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestEmailStatus {
    pub settings: DigestEmailSettings,
    pub smtp_configured: bool,
    pub parent_count: i64,
    pub last_run_at: Option<String>,
    pub next_due_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestEmailRunResult {
    pub sent: i64,
    pub failed: i64,
    pub skipped: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentDashboard {
    pub courses: Vec<StudentCourseSummary>,
    pub upcoming: Vec<StudentAssignmentGrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentCourseDetail {
    pub course: StudentCourseSummary,
    pub assignments: Vec<StudentAssignmentGrade>,
    pub materials: Vec<CourseMaterial>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub school_name: String,
    pub theme: String,
    pub font_scale: String,
    pub accent_color: String,
    pub locale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBackupSettings {
    pub enabled: bool,
    pub interval: String,
    pub max_keep: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBackupEntry {
    pub filename: String,
    pub exported_at: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoBackupStatus {
    pub settings: AutoBackupSettings,
    pub last_backup_at: Option<String>,
    pub next_due_at: Option<String>,
    pub backup_count: i64,
    pub backup_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashbookEntry {
    pub id: String,
    pub school_id: String,
    pub direction: String,
    pub category: String,
    pub amount: f64,
    pub currency: String,
    pub description: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub course_id: Option<String>,
    pub course_title: Option<String>,
    pub payment_method: String,
    pub reference: Option<String>,
    pub entry_date: String,
    pub created_by: Option<String>,
    pub created_by_name: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCashbookEntryInput {
    pub direction: String,
    pub category: String,
    pub amount: f64,
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub course_id: Option<String>,
    pub payment_method: String,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub entry_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashbookSummary {
    pub currency: String,
    pub total_income: f64,
    pub total_expense: f64,
    pub balance: f64,
    pub student_fees: f64,
    pub teacher_salary: f64,
    pub entry_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashbookSettings {
    pub currency: String,
    pub invoice_ninja_url: String,
    pub invoice_ninja_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveCashbookSettingsInput {
    pub currency: String,
    #[serde(default)]
    pub invoice_ninja_url: Option<String>,
    #[serde(default)]
    pub invoice_ninja_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashbookIntegrationTest {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppRosterMember {
    pub user_id: String,
    pub name: String,
    pub phone: Option<String>,
    pub normalized_phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppNativeParticipant {
    pub wa_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppGroupRosterDiff {
    pub group_id: String,
    pub native_available: bool,
    pub classmate_members: Vec<WhatsAppRosterMember>,
    pub whatsapp_participants: Vec<WhatsAppNativeParticipant>,
    pub matched_count: i64,
    pub only_in_classmate: Vec<WhatsAppRosterMember>,
    pub only_in_whatsapp: Vec<WhatsAppNativeParticipant>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppJoinRequest {
    pub join_request_id: String,
    pub wa_id: String,
    pub creation_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncNativeWhatsAppGroupRosterInput {
    pub group_id: String,
    #[serde(default)]
    pub send_invites: bool,
    #[serde(default)]
    pub remove_orphans: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncNativeWhatsAppGroupRosterResult {
    pub diff: WhatsAppGroupRosterDiff,
    pub invites: WhatsAppBroadcastResult,
    pub removed: i64,
    pub remove_errors: Vec<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManageWhatsAppJoinRequestsInput {
    pub group_id: String,
    pub join_request_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppGroupParticipantEvent {
    pub id: String,
    pub group_id: Option<String>,
    pub external_group_id: String,
    pub event_type: String,
    pub direction: Option<String>,
    pub wa_id: Option<String>,
    pub reason: Option<String>,
    pub join_request_id: Option<String>,
    pub received_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhatsAppGroupSettingsEvent {
    pub id: String,
    pub group_id: Option<String>,
    pub external_group_id: String,
    pub event_type: String,
    pub setting_kind: Option<String>,
    pub setting_value: Option<String>,
    pub update_successful: Option<bool>,
    pub error_summary: Option<String>,
    pub received_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpInfo {
    pub app_version: String,
    #[serde(default)]
    pub public_base_url: Option<String>,
    #[serde(default)]
    pub public_hub_path: Option<String>,
    #[serde(default)]
    pub webhook_url: Option<String>,
    #[serde(default)]
    pub hub_join_url: Option<String>,
    #[serde(default)]
    pub download_web_url: Option<String>,
    pub sync_running: bool,
    pub github_url: String,
    pub windows_installer_available: bool,
}
