use eframe::egui;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time;

mod reminders;
use reminders::ReminderManager;

struct ReminderDisplayApp {
    reminder_manager: Arc<Mutex<ReminderManager>>,
}

impl ReminderDisplayApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let reminder_manager = Arc::new(Mutex::new(ReminderManager::new()));

        // Start background file watcher and rotation
        let manager_clone = reminder_manager.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Ok(mut manager) = manager_clone.lock() {
                    manager.check_for_updates();
                    manager.rotate_if_needed();
                }
            }
        });

        Self { reminder_manager }
    }
}

impl eframe::App for ReminderDisplayApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_secs(1));

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Ok(manager) = self.reminder_manager.lock() {
                ui.vertical_centered(|ui| {
                    // Current time
                    ui.add_space(20.0);
                    ui.heading(manager.current_time());
                    ui.add_space(30.0);

                    // Main reminder display
                    if let Some(reminder) = manager.get_current_reminder() {
                        // Large, centered text for current reminder
                        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(&reminder.text)
                                    .size(64.0)
                                    .color(reminder.get_color()),
                            );

                            ui.add_space(20.0);

                            // Category and time info
                            ui.horizontal(|ui| {
                                if !reminder.category.is_empty() {
                                    ui.label(
                                        egui::RichText::new(format!("📂 {}", reminder.category))
                                            .size(20.0)
                                            .color(egui::Color32::GRAY),
                                    );
                                }

                                if let Some(time_range) = &reminder.time_range {
                                    ui.label(
                                        egui::RichText::new(format!("⏰ {}", time_range))
                                            .size(20.0)
                                            .color(egui::Color32::GRAY),
                                    );
                                }
                            });
                        });
                    } else {
                        ui.label(
                            egui::RichText::new("No reminders configured")
                                .size(64.0)
                                .color(egui::Color32::GRAY),
                        );
                    }

                    ui.add_space(40.0);

                    // Progress indicator
                    let total = manager.get_total_reminders();
                    if total > 1 {
                        let current_index = manager.get_current_index();
                        ui.horizontal(|ui| {
                            ui.label(format!("Reminder {} of {}", current_index + 1, total));
                            let progress = (current_index as f32) / (total as f32).max(1.0);
                            ui.add(egui::ProgressBar::new(progress).desired_width(200.0));
                        });
                    }

                    // Next rotation countdown
                    if total > 1 {
                        let next_rotation = manager.time_until_next_rotation();
                        ui.label(format!("Next reminder in: {}s", next_rotation));
                    }

                    ui.add_space(30.0);

                    // Status information
                    ui.separator();
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label(format!("📄 {} reminders loaded", total));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("Last updated: {}", manager.last_file_check()));
                        });
                    });
                });
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 480.0])
            .with_fullscreen(true)
            .with_always_on_top()
            .with_decorations(false)
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Work Reminders",
        options,
        Box::new(|cc| Ok(Box::new(ReminderDisplayApp::new(cc)))),
    )
}
