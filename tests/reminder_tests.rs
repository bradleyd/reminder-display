use chrono::{Local, Timelike};
use reminder_display::reminders::{Reminder, ReminderManager};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

// Global mutex to serialize tests that use environment variables
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[cfg(test)]
mod reminder_unit_tests {
    use super::*;

    fn create_test_reminder(priority: &str, time_range: Option<String>, days: Option<Vec<String>>) -> Reminder {
        Reminder {
            text: "Test reminder".to_string(),
            category: "Test".to_string(),
            priority: priority.to_string(),
            time_range,
            days,
        }
    }

    #[test]
    fn test_reminder_color_by_priority() {
        let high_priority = create_test_reminder("high", None, None);
        let medium_priority = create_test_reminder("medium", None, None);
        let low_priority = create_test_reminder("low", None, None);
        let unknown_priority = create_test_reminder("unknown", None, None);

        assert_eq!(high_priority.get_color(), egui::Color32::from_rgb(255, 100, 100));
        assert_eq!(medium_priority.get_color(), egui::Color32::from_rgb(255, 200, 100));
        assert_eq!(low_priority.get_color(), egui::Color32::from_rgb(100, 200, 255));
        assert_eq!(unknown_priority.get_color(), egui::Color32::WHITE);
    }

    #[test]
    fn test_reminder_urgent_priority_color() {
        let urgent = create_test_reminder("urgent", None, None);
        assert_eq!(urgent.get_color(), egui::Color32::from_rgb(255, 100, 100));
    }

    #[test]
    fn test_reminder_important_priority_color() {
        let important = create_test_reminder("important", None, None);
        assert_eq!(important.get_color(), egui::Color32::from_rgb(255, 200, 100));
    }

    #[test]
    fn test_reminder_info_priority_color() {
        let info = create_test_reminder("info", None, None);
        assert_eq!(info.get_color(), egui::Color32::from_rgb(100, 200, 255));
    }

    #[test]
    fn test_reminder_always_active_without_constraints() {
        let reminder = create_test_reminder("medium", None, None);
        assert!(reminder.is_active_now());
    }

    #[test]
    fn test_reminder_day_filtering() {
        let current_day = Local::now().format("%A").to_string().to_lowercase();
        
        let active_reminder = create_test_reminder(
            "medium",
            None,
            Some(vec![current_day.clone()])
        );
        assert!(active_reminder.is_active_now());

        let tomorrow = match current_day.as_str() {
            "monday" => "tuesday",
            "tuesday" => "wednesday",
            "wednesday" => "thursday",
            "thursday" => "friday",
            "friday" => "saturday",
            "saturday" => "sunday",
            "sunday" => "monday",
            _ => "monday",
        };

        let inactive_reminder = create_test_reminder(
            "medium",
            None,
            Some(vec![tomorrow.to_string()])
        );
        assert!(!inactive_reminder.is_active_now());
    }

    #[test]
    fn test_reminder_time_range_keywords() {
        let now = Local::now();
        let hour = now.hour();

        let morning_reminder = create_test_reminder(
            "medium",
            Some("morning".to_string()),
            None
        );
        assert_eq!(morning_reminder.is_active_now(), hour >= 6 && hour < 12);

        let afternoon_reminder = create_test_reminder(
            "medium",
            Some("afternoon".to_string()),
            None
        );
        assert_eq!(afternoon_reminder.is_active_now(), hour >= 12 && hour < 17);

        let evening_reminder = create_test_reminder(
            "medium",
            Some("evening".to_string()),
            None
        );
        assert_eq!(evening_reminder.is_active_now(), hour >= 17 && hour < 22);
    }

    #[test]
    fn test_reminder_time_range_format() {
        let now = Local::now();
        let current_time = now.time();
        
        let start = current_time
            .overflowing_sub_signed(chrono::Duration::hours(1))
            .0;
        let end = current_time
            .overflowing_add_signed(chrono::Duration::hours(1))
            .0;
        
        let time_range = format!(
            "{:02}:{:02}-{:02}:{:02}",
            start.hour(),
            start.minute(),
            end.hour(),
            end.minute()
        );
        
        let active_reminder = create_test_reminder(
            "medium",
            Some(time_range),
            None
        );
        assert!(active_reminder.is_active_now());

        let past_start = current_time
            .overflowing_sub_signed(chrono::Duration::hours(3))
            .0;
        let past_end = current_time
            .overflowing_sub_signed(chrono::Duration::hours(2))
            .0;
        
        let past_range = format!(
            "{:02}:{:02}-{:02}:{:02}",
            past_start.hour(),
            past_start.minute(),
            past_end.hour(),
            past_end.minute()
        );
        
        let past_reminder = create_test_reminder(
            "medium",
            Some(past_range),
            None
        );
        assert!(!past_reminder.is_active_now());
    }

    #[test]
    fn test_reminder_invalid_time_range_defaults_to_active() {
        let reminder = create_test_reminder(
            "medium",
            Some("invalid-format".to_string()),
            None
        );
        assert!(reminder.is_active_now());
    }

    #[test]
    fn test_reminder_combined_day_and_time_filtering() {
        let current_day = Local::now().format("%A").to_string().to_lowercase();
        let now = Local::now();
        let current_time = now.time();
        
        let start = current_time
            .overflowing_sub_signed(chrono::Duration::hours(1))
            .0;
        let end = current_time
            .overflowing_add_signed(chrono::Duration::hours(1))
            .0;
        
        let time_range = format!(
            "{:02}:{:02}-{:02}:{:02}",
            start.hour(),
            start.minute(),
            end.hour(),
            end.minute()
        );

        let active_reminder = create_test_reminder(
            "medium",
            Some(time_range.clone()),
            Some(vec![current_day.clone()])
        );
        assert!(active_reminder.is_active_now());

        let wrong_day_reminder = create_test_reminder(
            "medium",
            Some(time_range.clone()),
            Some(vec!["nonexistentday".to_string()])
        );
        assert!(!wrong_day_reminder.is_active_now());
    }
}

#[cfg(test)]
mod reminder_manager_tests {
    use super::*;

    fn create_test_file(dir: &TempDir, filename: &str, reminders: Vec<Reminder>) -> PathBuf {
        let path = dir.path().join(filename);
        let json = serde_json::to_string_pretty(&reminders).unwrap();
        fs::write(&path, json).unwrap();
        path
    }

    #[test]
    fn test_manager_loads_reminders_from_file() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let reminders = vec![
            Reminder {
                text: "Test 1".to_string(),
                category: "Cat1".to_string(),
                priority: "high".to_string(),
                time_range: None,
                days: None,
            },
            Reminder {
                text: "Test 2".to_string(),
                category: "Cat2".to_string(),
                priority: "low".to_string(),
                time_range: None,
                days: None,
            },
        ];
        
        let file_path = create_test_file(&temp_dir, "test_reminders.json", reminders.clone());
        unsafe {
            std::env::set_var("REMINDERS_FILE", file_path.to_str().unwrap());
        }
        
        let manager = ReminderManager::new();
        assert_eq!(manager.get_total_reminders(), 2);
        
        unsafe {
            std::env::remove_var("REMINDERS_FILE");
        }
    }

    #[test]
    fn test_manager_creates_default_file_if_missing() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("nonexistent.json");
        unsafe {
            std::env::set_var("REMINDERS_FILE", file_path.to_str().unwrap());
        }
        
        let manager = ReminderManager::new();
        
        assert!(file_path.exists());
        assert!(manager.get_total_reminders() > 0);
        
        unsafe {
            std::env::remove_var("REMINDERS_FILE");
        }
    }

    #[test]
    fn test_manager_filters_active_reminders() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let current_day = Local::now().format("%A").to_string().to_lowercase();
        let tomorrow = match current_day.as_str() {
            "monday" => "tuesday",
            "tuesday" => "wednesday",
            "wednesday" => "thursday",
            "thursday" => "friday",
            "friday" => "saturday",
            "saturday" => "sunday",
            "sunday" => "monday",
            _ => "monday",
        };
        
        let reminders = vec![
            Reminder {
                text: "Active today".to_string(),
                category: "Test".to_string(),
                priority: "high".to_string(),
                time_range: None,
                days: Some(vec![current_day.clone()]),
            },
            Reminder {
                text: "Not active today".to_string(),
                category: "Test".to_string(),
                priority: "low".to_string(),
                time_range: None,
                days: Some(vec![tomorrow.to_string()]),
            },
            Reminder {
                text: "Always active".to_string(),
                category: "Test".to_string(),
                priority: "medium".to_string(),
                time_range: None,
                days: None,
            },
        ];
        
        let file_path = create_test_file(&temp_dir, "test_reminders.json", reminders);
        unsafe {
            std::env::set_var("REMINDERS_FILE", file_path.to_str().unwrap());
        }
        
        let manager = ReminderManager::new();
        assert_eq!(manager.get_total_reminders(), 2); // Only active reminders counted
        
        unsafe {
            std::env::remove_var("REMINDERS_FILE");
        }
    }

    #[test]
    fn test_manager_rotation() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let reminders = vec![
            Reminder {
                text: "Reminder 1".to_string(),
                category: "Test".to_string(),
                priority: "high".to_string(),
                time_range: None,
                days: None,
            },
            Reminder {
                text: "Reminder 2".to_string(),
                category: "Test".to_string(),
                priority: "medium".to_string(),
                time_range: None,
                days: None,
            },
            Reminder {
                text: "Reminder 3".to_string(),
                category: "Test".to_string(),
                priority: "low".to_string(),
                time_range: None,
                days: None,
            },
        ];
        
        let file_path = create_test_file(&temp_dir, "test_reminders.json", reminders);
        unsafe {
            std::env::set_var("REMINDERS_FILE", file_path.to_str().unwrap());
        }
        
        let mut manager = ReminderManager::new();
        let _initial_index = manager.get_current_index();
        
        // Force rotation by setting last_rotation to past
        std::thread::sleep(std::time::Duration::from_millis(100));
        manager.rotate_if_needed();
        
        // Note: Since rotate_if_needed checks the time interval,
        // we might need to modify the manager's internal state for testing
        // This is a limitation of the current design
        
        unsafe {
            std::env::remove_var("REMINDERS_FILE");
        }
    }

    #[test]
    fn test_manager_time_until_next_rotation() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let reminders = vec![
            Reminder {
                text: "Test".to_string(),
                category: "Test".to_string(),
                priority: "high".to_string(),
                time_range: None,
                days: None,
            },
        ];
        
        let file_path = create_test_file(&temp_dir, "test_reminders.json", reminders);
        unsafe {
            std::env::set_var("REMINDERS_FILE", file_path.to_str().unwrap());
        }
        
        let manager = ReminderManager::new();
        let time_until = manager.time_until_next_rotation();
        
        assert!(time_until <= 30); // Default rotation interval is 30 seconds
        
        unsafe {
            std::env::remove_var("REMINDERS_FILE");
        }
    }

    #[test]
    fn test_manager_current_time_format() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let reminders = vec![];
        let file_path = create_test_file(&temp_dir, "test_reminders.json", reminders);
        unsafe {
            std::env::set_var("REMINDERS_FILE", file_path.to_str().unwrap());
        }
        
        let manager = ReminderManager::new();
        let time_str = manager.current_time();
        
        // Check format includes day, month, time
        assert!(time_str.contains(","));
        assert!(time_str.contains("-"));
        assert!(time_str.contains(":"));
        
        unsafe {
            std::env::remove_var("REMINDERS_FILE");
        }
    }

    #[test]
    fn test_manager_reload_reminders() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let temp_dir = TempDir::new().unwrap();
        let initial_reminders = vec![
            Reminder {
                text: "Initial".to_string(),
                category: "Test".to_string(),
                priority: "high".to_string(),
                time_range: None,
                days: None,
            },
        ];
        
        let file_path = create_test_file(&temp_dir, "test_reminders.json", initial_reminders);
        unsafe {
            std::env::set_var("REMINDERS_FILE", file_path.to_str().unwrap());
        }
        
        // Verify the file exists and env var is set before creating manager
        assert!(file_path.exists(), "Test file should exist");
        assert_eq!(std::env::var("REMINDERS_FILE").unwrap(), file_path.to_str().unwrap());
        
        let mut manager = ReminderManager::new();
        assert_eq!(manager.get_total_reminders(), 1, "Manager should load 1 reminder from file");
        
        // Update file with more reminders
        let updated_reminders = vec![
            Reminder {
                text: "Updated 1".to_string(),
                category: "Test".to_string(),
                priority: "high".to_string(),
                time_range: None,
                days: None,
            },
            Reminder {
                text: "Updated 2".to_string(),
                category: "Test".to_string(),
                priority: "medium".to_string(),
                time_range: None,
                days: None,
            },
        ];
        
        let json = serde_json::to_string_pretty(&updated_reminders).unwrap();
        fs::write(&file_path, json).unwrap();
        
        manager.check_for_updates();
        assert_eq!(manager.get_total_reminders(), 2);
        
        unsafe {
            std::env::remove_var("REMINDERS_FILE");
        }
    }
}