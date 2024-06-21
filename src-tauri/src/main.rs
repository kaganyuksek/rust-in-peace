// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;

use tauri::Manager;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn shutdown_pc() {
    #[cfg(target_os = "windows")]
    {
        Command::new("shutdown")
            .args(&["/s", "/t", "0"])
            .spawn()
            .expect("Failed to shutdown the system");
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to shut down")
            .spawn()
            .expect("Failed to shutdown the system");
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("shutdown")
            .args(&["-h", "now"])
            .spawn()
            .expect("Failed to shutdown the system");
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![shutdown_pc])
        .setup(|app| {
            let splashscreen_window = app.get_window("splashscreen").unwrap();
            let main_window = app.get_window("main").unwrap();

            // we perform the initialization code on a new task so the app doesn't freeze
            tauri::async_runtime::spawn(async move {
                // initialize your app here instead of sleeping :)
                println!("Initializing...");
                std::thread::sleep(std::time::Duration::from_millis(750));
                println!("Done initializing.");

                // After it's done, close the splashscreen and display the main window
                splashscreen_window.close().unwrap();
                main_window.show().unwrap();
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
