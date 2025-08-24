# Reminder Display

A fullscreen reminder display application built with Rust and egui, designed to show rotating work reminders on a dedicated display.

## Features

- **Fullscreen Display**: Runs in fullscreen mode with no window decorations for a clean, dedicated display
- **Auto-rotating Reminders**: Automatically cycles through reminders every 30 seconds
- **Time-based Filtering**: Shows reminders only during their configured time ranges and days
- **Priority Color Coding**: 
  - 🔴 High priority (Red)
  - 🟡 Medium priority (Yellow)  
  - 🔵 Low priority (Blue)
- **Live Reload**: Automatically detects and loads changes to the reminders file
- **Progress Tracking**: Shows current reminder position and countdown to next rotation

## Installation

### Prerequisites

- Rust toolchain (install from [rustup.rs](https://rustup.rs/))
- Build essentials for your platform

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd reminder-display

# Build the application
cargo build --release

# Run the application
cargo run --release
```

### Linux Setup (Raspberry Pi/Ubuntu)

A setup script is provided for Linux systems:

```bash
chmod +x setup.sh
./setup.sh
```

This will:
- Install Rust if not present
- Install required system dependencies
- Build the application

## Configuration

Reminders are configured in `work_reminders.json`. Example format:

```json
[
  {
    "text": "Take a 5-minute break - stretch and hydrate",
    "category": "Health",
    "priority": "medium",
    "time_range": null,
    "days": null
  },
  {
    "text": "Check and respond to team messages",
    "category": "Communication", 
    "priority": "medium",
    "time_range": "09:00-17:00",
    "days": ["monday", "tuesday", "wednesday", "thursday", "friday"]
  }
]
```

### Reminder Fields

- **text** (required): The reminder message to display
- **category**: Category label for organization
- **priority**: `"high"`, `"medium"`, or `"low"` (affects color)
- **time_range**: Time window in "HH:MM-HH:MM" format (24-hour)
- **days**: Array of weekdays when reminder should show

## Usage

1. Create or edit `work_reminders.json` with your reminders
2. Run the application:
   ```bash
   ./target/release/reminder-display
   ```
3. The display will:
   - Show reminders fullscreen
   - Rotate through active reminders every 30 seconds
   - Filter based on current time and day
   - Reload automatically when JSON file changes

## Display Information

The application shows:
- Current time at the top
- Main reminder text (large, centered)
- Category and time range (if configured)
- Progress indicator showing position in reminder list
- Countdown to next reminder rotation
- Status bar with total reminders and last update time

## Development

### Project Structure

```
reminder-display/
├── src/
│   ├── main.rs          # Main application and UI
│   └── reminders.rs     # Reminder management logic
├── Cargo.toml           # Rust dependencies
├── work_reminders.json  # Reminder configuration
└── setup.sh            # Linux setup script
```

### Dependencies

- **eframe/egui**: Cross-platform GUI framework
- **tokio**: Async runtime for background tasks
- **serde/serde_json**: JSON parsing
- **chrono**: Date/time handling
- **notify**: File system watching
- **dirs**: User directory paths

## License

[Your License Here]

## Contributing

[Your Contributing Guidelines Here]