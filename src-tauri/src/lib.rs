mod commands;
pub mod db;
pub mod headless;
mod hub;
mod models;
mod sync_server;
mod video;

use hub::HubRuntime;
use models::User;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use sync_server::SyncRuntime;
use tauri::Manager;
use video::VideoRuntime;

pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub hub: Mutex<HubRuntime>,
    pub sync: Mutex<SyncRuntime>,
    pub video: VideoRuntime,
    pub session: Mutex<Option<User>>,
    pub active_school_id: Mutex<Option<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            std::fs::create_dir_all(&app_data).expect("failed to create app data directory");

            let db_path = app_data.join("classmate.db");
            let conn = db::init(&db_path).expect("failed to initialize database");
            let db = Arc::new(Mutex::new(conn));

            app.manage(AppState {
                db: db.clone(),
                hub: Mutex::new(HubRuntime::new()),
                sync: Mutex::new(SyncRuntime::new()),
                video: VideoRuntime::new(),
                session: Mutex::new(None),
                active_school_id: Mutex::new(None),
            });

            let app_data_for_backup = app_data.clone();
            std::thread::spawn(move || {
                if let Ok(conn) = db.lock() {
                    let _ = commands::backup::maybe_run_scheduled_backup(&app_data_for_backup, &conn);
                    let _ = commands::email::maybe_run_scheduled_digest(&conn);
                    let _ = commands::whatsapp_api::maybe_run_scheduled_whatsapp_broadcasts(&conn);
                    let _ = commands::push::maybe_run_scheduled_push_reminders(&conn);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::login,
            commands::auth::logout,
            commands::auth::get_session,
            commands::get_dashboard_stats,
            commands::list_users,
            commands::create_user,
            commands::list_courses,
            commands::create_course,
            commands::list_assignments,
            commands::gradebook::create_assignment,
            commands::gradebook::get_gradebook,
            commands::gradebook::save_grade,
            commands::gradebook::export_gradebook_csv,
            commands::enrollments::list_enrollments,
            commands::enrollments::enroll_student,
            commands::enrollments::unenroll_student,
            commands::enrollments::list_students,
            commands::materials::list_materials,
            commands::materials::create_material,
            commands::materials::delete_material,
            commands::announcements::list_announcements,
            commands::announcements::create_announcement,
            commands::announcements::delete_announcement,
            commands::forums::list_forum_topics,
            commands::forums::list_forum_posts,
            commands::forums::create_forum_topic,
            commands::forums::create_forum_post,
            commands::sync::export_backup,
            commands::sync::import_backup,
            commands::sync::get_setting,
            commands::sync::set_setting,
            commands::sync::list_settings,
            commands::sync::import_oneroster_csv,
            commands::lti::list_lti_tools,
            commands::lti::create_lti_tool,
            commands::lti::delete_lti_tool,
            commands::lti::get_lti_launch_url,
            commands::parent::link_parent_student,
            commands::parent::get_parent_dashboard,
            commands::parent::issue_certificate,
            commands::parent::list_certificates,
            commands::video::get_video_status,
            commands::video::start_video,
            commands::video::stop_video,
            commands::quizzes::list_quizzes,
            commands::quizzes::create_quiz,
            commands::quizzes::add_quiz_question,
            commands::quizzes::publish_quiz,
            commands::quizzes::get_quiz_detail,
            commands::quizzes::list_quiz_attempts,
            commands::quizzes::get_quiz_attempt_detail,
            commands::quizzes::grade_quiz_attempt,
            commands::quizzes::submit_quiz,
            commands::analytics::get_analytics,
            commands::analytics::export_attendance_csv,
            commands::schedule::list_schedule,
            commands::schedule::create_schedule_slot,
            commands::schedule::delete_schedule_slot,
            commands::polls::create_session_poll,
            commands::polls::close_session_poll,
            commands::polls::get_active_session_poll,
            commands::polls::get_session_poll_results,
            commands::polls::vote_session_poll,
            commands::submissions::list_submissions,
            commands::submissions::submit_assignment,
            commands::submissions::grade_submission,
            commands::student::get_student_dashboard,
            commands::student::get_my_course,
            commands::sync::get_ui_preferences,
            commands::sync::set_ui_preferences,
            commands::sync::get_sync_status,
            commands::sync::start_sync_server,
            commands::sync::stop_sync_server,
            commands::sync::set_sync_token,
            commands::sync::pull_from_peer,
            commands::sync::push_to_peer,
            commands::sessions::list_class_sessions,
            commands::student::submit_my_assignment,
            commands::backup::get_auto_backup_status,
            commands::backup::set_auto_backup_settings,
            commands::backup::run_auto_backup_now,
            commands::backup::list_auto_backups,
            commands::backup::restore_auto_backup,
            commands::parent::generate_parent_digest,
            commands::parent::list_parent_grades,
            commands::email::get_smtp_settings,
            commands::email::save_smtp_settings,
            commands::email::test_smtp_connection,
            commands::email::send_parent_digest_email,
            commands::email::list_email_log,
            commands::email::get_digest_email_status,
            commands::email::set_digest_email_settings,
            commands::email::run_scheduled_digest_now,
            commands::rubrics::get_assignment_rubric,
            commands::rubrics::save_assignment_rubric,
            commands::backup::push_backup_to_cloud,
            commands::whatsapp::list_whatsapp_groups,
            commands::whatsapp::create_whatsapp_group,
            commands::whatsapp::delete_whatsapp_group,
            commands::whatsapp::list_whatsapp_group_members,
            commands::whatsapp::add_whatsapp_group_member,
            commands::whatsapp::remove_whatsapp_group_member,
            commands::whatsapp::sync_whatsapp_group_members,
            commands::whatsapp::update_user_phone,
            commands::whatsapp::build_whatsapp_share,
            commands::whatsapp_api::get_whatsapp_business_settings,
            commands::whatsapp_api::save_whatsapp_business_settings,
            commands::whatsapp_api::test_whatsapp_business_connection,
            commands::whatsapp_api::link_whatsapp_group,
            commands::whatsapp_api::unlink_whatsapp_group,
            commands::whatsapp_api::get_whatsapp_group_link,
            commands::whatsapp_api::create_native_whatsapp_group,
            commands::whatsapp_api::refresh_native_whatsapp_group,
            commands::whatsapp_api::send_whatsapp_group_invites,
            commands::whatsapp_api::get_whatsapp_group_roster_diff,
            commands::whatsapp_api::sync_native_whatsapp_group_roster,
            commands::whatsapp_api::list_whatsapp_group_join_requests,
            commands::whatsapp_api::approve_whatsapp_group_join_requests,
            commands::whatsapp_api::reject_whatsapp_group_join_requests,
            commands::whatsapp_api::list_whatsapp_group_participant_events,
            commands::whatsapp_api::list_whatsapp_group_settings_events,
            commands::cashbook::get_cashbook_settings,
            commands::cashbook::save_cashbook_settings,
            commands::cashbook::test_invoice_ninja_connection,
            commands::cashbook::list_cashbook_entries,
            commands::cashbook::create_cashbook_entry,
            commands::cashbook::delete_cashbook_entry,
            commands::cashbook::get_cashbook_summary,
            commands::cashbook::export_cashbook_csv,
            commands::whatsapp_api::set_whatsapp_consent,
            commands::whatsapp_api::send_whatsapp_broadcast,
            commands::whatsapp_api::list_whatsapp_outbound_messages,
            commands::whatsapp_api::get_whatsapp_template_settings,
            commands::whatsapp_api::save_whatsapp_template_settings,
            commands::whatsapp_api::preview_whatsapp_assignment_template,
            commands::whatsapp_api::send_whatsapp_template_broadcast,
            commands::whatsapp_api::get_whatsapp_inbound_routing_settings,
            commands::whatsapp_api::set_whatsapp_inbound_routing_settings,
            commands::whatsapp_api::list_whatsapp_inbound_messages,
            commands::whatsapp_api::route_whatsapp_inbound_message,
            commands::whatsapp_api::ignore_whatsapp_inbound_message,
            commands::whatsapp_api::create_whatsapp_scheduled_broadcast,
            commands::whatsapp_api::list_whatsapp_scheduled_broadcasts,
            commands::whatsapp_api::cancel_whatsapp_scheduled_broadcast,
            commands::whatsapp_api::run_due_whatsapp_scheduled_broadcasts,
            commands::whatsapp_api::get_whatsapp_compliance_settings,
            commands::whatsapp_api::save_whatsapp_compliance_settings,
            commands::whatsapp_api::export_whatsapp_gdpr,
            commands::whatsapp_api::list_whatsapp_consent_log,
            commands::schools::get_tenancy_context,
            commands::schools::set_active_school,
            commands::schools::create_school,
            commands::schools::update_school,
            commands::schools::list_school_members,
            commands::schools::add_school_member,
            commands::schools::remove_school_member,
            commands::push::get_push_settings,
            commands::push::save_push_settings,
            commands::push::get_push_reminder_status,
            commands::push::set_push_reminder_settings,
            commands::push::run_push_reminders_now,
            commands::push::test_push_notification,
            commands::push::register_push_device,
            commands::push::unregister_push_device,
            commands::push::list_my_push_devices,
            commands::push::send_push_notification,
            commands::push::list_push_log,
            commands::get_hub_status,
            commands::start_class_hub,
            commands::stop_class_hub,
            commands::get_attendance_count,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
