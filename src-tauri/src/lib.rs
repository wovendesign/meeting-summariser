use tauri_plugin_fs::FsExt;
use tokio::fs;
use tauri::AppHandle;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_meetings(app: AppHandle) -> Result<Vec<String>, String> {
    // resolve <app>/uploads
    let app_dir = app
        .path_resolver()
        .app_dir()
        .ok_or("failed to resolve app dir")?;
    let uploads = app_dir.join("uploads");

    // read directory
    let mut rd = fs::read_dir(uploads)
        .await
        .map_err(|e| e.to_string())?;

    let mut folders = Vec::new();
    while let Some(entry) = rd
        .next_entry()
        .await
        .map_err(|e| e.to_string())?
    {
        let ft = entry
            .file_type()
            .await
            .map_err(|e| e.to_string())?;
        if ft.is_dir() {
            folders.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(folders)
}


#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let scope = app.fs_scope();
            // scope.allow_directory("/path/to/directory", false);
            // dbg!(scope.allowed());

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_meetings])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
