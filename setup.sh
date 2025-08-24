#!/bin/bash
echo "Setting up Work Reminder Display..."

# Install Rust if needed
if ! command -v cargo &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  source ~/.cargo/env
fi

# Install dependencies
sudo apt update
sudo apt install -y build-essential pkg-config libfontconfig1-dev

# Create project
mkdir -p ~/reminder_display/src
cd ~/reminder_display

# Build
cargo build --release

echo "Setup complete!"
echo ""
echo "To customize your reminders, edit: /home/bradleydsmith/work_reminders.json"
echo "To run: ./target/release/display_app"
echo ""
echo "The display will:"
echo "- Rotate reminders every 30 seconds"
echo "- Show only reminders for current time/day"
echo "- Auto-reload when you update the JSON file"
echo "- Use color coding: Red=High priority, Yellow=Medium, Blue=Low"
