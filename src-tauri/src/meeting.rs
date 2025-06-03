use std::collections::HashMap;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub async fn save_speaker_names(app: AppHandle, meeting_id: &str, names: HashMap<String, String>) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_local_data_dir()
        .expect("Failed to get app local data directory");
    let base_dir = app_dir.join("uploads").join(meeting_id);

    let transcript_txt_path = base_dir.join(format!("{}.txt", meeting_id));
    let transcript_json_path = base_dir.join(format!("{}.json", meeting_id));
    
    let mut transcript_txt = tokio::fs::read_to_string(&transcript_txt_path).await.map_err(|e| e.to_string())?;
    let mut transcript_json = tokio::fs::read_to_string(&transcript_json_path).await.map_err(|e| e.to_string())?;
    
    for (key, value) in &names {
    //     Key:     Old Name
    //     Value:   New Name
        // Replace old name with new name in transcript text
        transcript_txt = transcript_txt.replace(key.as_str(), value.as_str());
        
        // Replace old name with new name in transcript JSON
        transcript_json = transcript_json.replace(key.as_str(), value.as_str());
    }

    // Write the updated transcript text back to the file
    tokio::fs::write(&transcript_txt_path, transcript_txt)
        .await
        .map_err(|e| e.to_string())?;
    // Write the updated transcript JSON back to the file
    tokio::fs::write(&transcript_json_path, transcript_json)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}