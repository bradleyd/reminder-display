use chrono::{DateTime, Local, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reminder {
    pub text: String,
    pub category: String,
    pub priority: String,
    pub time_range: Option<String>, // e.g., "09:00-17:00" or "morning"
    pub days: Option<Vec<String>>,  // e.g., ["monday", "tuesday", "wednesday"]
}

impl Reminder {
    pub fn get_color(&self) -> egui::Color32 {
        match self.priority.to_lowercase().as_str() {
            "high" | "urgent" => egui::Color32::from_rgb(255, 100, 100),
            "medium" | "important" => egui::Color32::from_rgb(255, 200, 100),
            "low" | "info" => egui::Color32::from_rgb(100, 200, 255),
            _ => egui::Color32::WHITE,
        }
    }

    pub fn is_active_now(&self) -> bool {
        let now = Local::now();

        // Check day of week if specified
        if let Some(days) = &self.days {
            let current_day = now.format("%A").to_string().to_lowercase();
            if !days.iter().any(|d| d.to_lowercase() == current_day) {
                return false;
            }
        }

        // Check time range if specified
        if let Some(time_range) = &self.time_range {
            return self.is_in_time_range(time_range, now);
        }

        true
    }

    fn is_in_time_range(&self, time_range: &str, now: DateTime<Local>) -> bool {
        match time_range.to_lowercase().as_str() {
            "morning" => {
                let hour = now.hour();
                hour >= 6 && hour < 12
            }
            "afternoon" => {
                let hour = now.hour();
                hour >= 12 && hour < 17
            }
            "evening" => {
                let hour = now.hour();
                hour >= 17 && hour < 22
            }
            _ => {
                // Parse "HH:MM-HH:MM" format
                if let Some((start_str, end_str)) = time_range.split_once('-') {
                    if let (Ok(start), Ok(end)) = (
                        NaiveTime::parse_from_str(start_str.trim(), "%H:%M"),
                        NaiveTime::parse_from_str(end_str.trim(), "%H:%M"),
                    ) {
                        let current_time = now.time();
                        return current_time >= start && current_time <= end;
                    }
                }
                true // Default to always active if can't parse
            }
        }
    }
}

pub struct ReminderManager {
    reminders: Vec<Reminder>,
    current_index: usize,
    last_rotation: u64,
    rotation_interval: u64, // seconds
    last_file_check: String,
    file_path: String,
}

impl ReminderManager {
    pub fn new() -> Self {
        let mut manager = Self {
            reminders: Vec::new(),
            current_index: 0,
            last_rotation: Self::current_timestamp(),
            rotation_interval: 30, // 30 seconds between reminders
            last_file_check: String::new(),
            file_path: Self::find_reminders_file(),
        };
        manager.load_reminders();
        manager
    }

    fn find_reminders_file() -> String {
        // Check environment variable first
        if let Ok(path) = std::env::var("REMINDERS_FILE") {
            return path;
        }

        // Check multiple locations in order of preference
        let possible_paths = vec![
            PathBuf::from("work_reminders.json"),   // Current directory
            PathBuf::from("./work_reminders.json"), // Explicit current directory
            dirs::home_dir()
                .map(|d| d.join("work_reminders.json"))
                .unwrap_or_default(), // Home directory
            PathBuf::from("/home/bradleydsmith/work_reminders.json"), // Original fallback
        ];

        // Return the first existing file, or default to current directory
        for path in possible_paths {
            if path.exists() {
                return path.to_string_lossy().to_string();
            }
        }

        // Default to current directory if none exist
        "work_reminders.json".to_string()
    }

    pub fn load_reminders(&mut self) {
        match fs::read_to_string(&self.file_path) {
            Ok(content) => {
                match serde_json::from_str::<Vec<Reminder>>(&content) {
                    Ok(reminders) => {
                        self.reminders = reminders;
                        self.last_file_check = Local::now().format("%H:%M:%S").to_string();

                        // Reset index if we have fewer reminders now
                        if self.current_index >= self.reminders.len() && !self.reminders.is_empty()
                        {
                            self.current_index = 0;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing reminders file: {}", e);
                    }
                }
            }
            Err(_) => {
                // Create default file if it doesn't exist
                self.create_default_reminders_file();
            }
        }
    }

    fn create_default_reminders_file(&mut self) {
        let default_reminders = vec![
            Reminder {
                text: "Check your monitoring dashboards".to_string(),
                category: "DevOps".to_string(),
                priority: "high".to_string(),
                time_range: Some("09:00-17:00".to_string()),
                days: Some(vec![
                    "monday".to_string(),
                    "tuesday".to_string(),
                    "wednesday".to_string(),
                    "thursday".to_string(),
                    "friday".to_string(),
                ]),
            },
            Reminder {
                text: "Review and respond to alerts".to_string(),
                category: "DevOps".to_string(),
                priority: "high".to_string(),
                time_range: Some("09:00-17:00".to_string()),
                days: Some(vec![
                    "monday".to_string(),
                    "tuesday".to_string(),
                    "wednesday".to_string(),
                    "thursday".to_string(),
                    "friday".to_string(),
                ]),
            },
            Reminder {
                text: "Take a 5-minute break and stretch".to_string(),
                category: "Health".to_string(),
                priority: "medium".to_string(),
                time_range: None,
                days: None,
            },
            Reminder {
                text: "Check backup status and logs".to_string(),
                category: "DevOps".to_string(),
                priority: "medium".to_string(),
                time_range: Some("morning".to_string()),
                days: Some(vec![
                    "monday".to_string(),
                    "wednesday".to_string(),
                    "friday".to_string(),
                ]),
            },
            Reminder {
                text: "Review security alerts and patches".to_string(),
                category: "Security".to_string(),
                priority: "high".to_string(),
                time_range: Some("morning".to_string()),
                days: Some(vec!["monday".to_string(), "thursday".to_string()]),
            },
        ];

        if let Ok(json) = serde_json::to_string_pretty(&default_reminders) {
            if fs::write(&self.file_path, json).is_ok() {
                self.reminders = default_reminders;
                self.last_file_check = Local::now().format("%H:%M:%S").to_string();
            }
        }
    }

    pub fn check_for_updates(&mut self) {
        self.load_reminders();
    }

    pub fn get_current_reminder(&self) -> Option<&Reminder> {
        let active_reminders: Vec<&Reminder> = self
            .reminders
            .iter()
            .filter(|r| r.is_active_now())
            .collect();

        if active_reminders.is_empty() {
            return None;
        }

        active_reminders
            .get(self.current_index % active_reminders.len())
            .copied()
    }

    pub fn rotate_if_needed(&mut self) {
        let now = Self::current_timestamp();
        if now - self.last_rotation >= self.rotation_interval {
            self.current_index = (self.current_index + 1) % self.get_active_reminder_count().max(1);
            self.last_rotation = now;
        }
    }

    pub fn get_total_reminders(&self) -> usize {
        self.get_active_reminder_count()
    }

    pub fn get_current_index(&self) -> usize {
        self.current_index
    }

    pub fn time_until_next_rotation(&self) -> u64 {
        let now = Self::current_timestamp();
        let elapsed = now - self.last_rotation;
        if elapsed >= self.rotation_interval {
            0
        } else {
            self.rotation_interval - elapsed
        }
    }

    pub fn current_time(&self) -> String {
        Local::now().format("%A, %B %d - %I:%M %p").to_string()
    }

    pub fn last_file_check(&self) -> &str {
        &self.last_file_check
    }

    fn get_active_reminder_count(&self) -> usize {
        self.reminders.iter().filter(|r| r.is_active_now()).count()
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
