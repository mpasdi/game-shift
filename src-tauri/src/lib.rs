mod db;
mod games;

#[derive(serde::Serialize)]
struct AppInfo {
    name: &'static str,
    version: &'static str,
}

#[tauri::command]
fn app_info() -> AppInfo {
    AppInfo {
        name: "Game Shift",
        version: env!("CARGO_PKG_VERSION"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            db::initialize(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![app_info, games::list_games_command])
        .run(tauri::generate_context!())
        .expect("error while running Game Shift");
}