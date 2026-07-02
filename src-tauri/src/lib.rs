mod db;
mod games;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    name: &'static str,
    version: &'static str,
    identifier: &'static str,
    data_dir: String,
    database_path: String,
}

#[tauri::command]
fn app_info(app: tauri::AppHandle) -> Result<AppInfo, String> {
    let database_path = db::database_path(&app)?;
    let data_dir = database_path
        .parent()
        .ok_or_else(|| "无法解析应用数据目录".to_string())?
        .to_string_lossy()
        .to_string();

    Ok(AppInfo {
        name: "Game Shift",
        version: env!("CARGO_PKG_VERSION"),
        identifier: "com.gameshift.app",
        data_dir,
        database_path: database_path.to_string_lossy().to_string(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            db::initialize(app.handle())?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_info,
            games::list_games_command,
            games::get_game_command,
            games::create_game_command,
            games::update_game_command,
            games::delete_game_command,
            games::launch_game_command,
            games::scan_games_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running Game Shift");
}
