extern crate cocoa;
extern crate gtk;
extern crate objc;

use gtk::prelude::*;
use gtk::{Label, ListBox, ListBoxRow, Window, WindowType};
use log::{info, warn};

use std::process::Command;
use std::str;
use std::thread;
use std::time::{Duration, Instant};

fn get_active_application_name() -> String {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("tell application \"System Events\" to get the unix id of every process whose frontmost is true")
        .output()
        .expect("Failed to execute osascript");

    let stdout = str::from_utf8(&output.stdout).expect("Could not convert to string");
    let pid: i32 = stdout.trim().parse().expect("Failed to parse PID");

    let ps_output = Command::new("ps")
        .arg("-p")
        .arg(pid.to_string())
        .arg("-o")
        .arg("command=")
        .output()
        .expect("Failed to execute ps");

    let ps_stdout = str::from_utf8(&ps_output.stdout)
        .expect("Could not convert to string")
        .trim()
        .to_string();

    if ps_stdout.contains("Code") {
        return "VS Code".to_string();
    }
    // Add more conditions for other Electron apps if needed
    ps_stdout
}

fn block_unauthorized_launch(app_name: &str) {
    info!("Unauthorized launch of {} detected. Blocking...", app_name);
    // Get the process ID of the unauthorized application
    let output = Command::new("pgrep")
        .arg("-f")
        .arg(app_name)
        .output()
        .expect("Failed to execute pgrep");

    let pid_str = str::from_utf8(&output.stdout)
        .expect("Failed to convert to string")
        .trim();
    let pids: Vec<&str> = pid_str.split('\n').collect();
    info!("PIDs to kill: {:?}", pids); // Debugging line

    if pids.is_empty() || pids[0].is_empty() {
        warn!("No matching PID found.");
        return;
    }
    // Terminate the unauthorized applications
    for pid in pids {
        if !pid.is_empty() {
            let kill_output = Command::new("kill")
                .arg("-9")
                .arg(pid)
                .output()
                .expect("Failed to execute kill");

            // Additional debug information
            info!("Kill command output for PID {}: {:?}", pid, kill_output);
        }
    }
}

fn initialize_gui(whitelist: Vec<&str>) {
    // Initialize GTK
    gtk::init().expect("Failed to initialize GTK.");
    // Create a new top-level window and set its title
    let window = Window::new(WindowType::Toplevel);
    window.set_title("Whitelisted Apps");
    window.set_default_size(800, 1200);
    let list_box = ListBox::new();
    // Populate the ListBox with the whitelisted apps
    for app in whitelist.iter() {
        let row = ListBoxRow::new();
        let label = Label::new(Some(app));
        row.add(&label);
        list_box.add(&row);
    }
    window.add(&list_box);
    window.show_all();
    // Handle the 'destroy' event to terminate the GTK main loop when the window is closed
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::glib::signal::Propagation::Stop
    });

    // GTK main event loop
    gtk::main();
}

fn main() {
    env_logger::init();
    let whitelist = vec![
        "VS Code",
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    ];

    let whitelist_clone = whitelist.clone();

    // Spawn a new thread to run your focus session loop
    thread::spawn(move || {
        let focus_duration = Duration::from_secs(1 * 60);
        let start_time = Instant::now();
        loop {
            let app_name = get_active_application_name();
            info!("Current application: {}", app_name);
            thread::sleep(Duration::from_secs(2));
            let elapsed_time = Instant::now().duration_since(start_time);
            if elapsed_time <= focus_duration {
                info!("Focus session in progress.");
            }
            if elapsed_time >= focus_duration {
                info!("Focus session ended. You can now use any application.");
                break;
            }
            if !whitelist_clone.contains(&app_name.as_str()) {
                info!("Unauthorized launch of {} detected. Blocking...", app_name);
                block_unauthorized_launch(&app_name);
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    // Initialize GUI
    initialize_gui(whitelist);
}
