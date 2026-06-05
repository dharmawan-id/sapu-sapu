// Sapu - neo-brutalist disk cleaner for Windows.
// Release builds hide the console window; debug keeps it for panics.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cleaner;

fn main() {
    tauri::Builder::default()
        .manage(cleaner::ScanState::default())
        .invoke_handler(tauri::generate_handler![
            cleaner::disk_info,
            cleaner::scan_overview,
            cleaner::cancel_scan,
            cleaner::list_clean_targets,
            cleaner::scan_projects,
            cleaner::clean_paths,
            cleaner::delete_projects,
            cleaner::empty_recycle_bin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Sapu");
}
