use tauri::Window;

#[tauri::command]
pub async fn show(window: Window) {
    window.show().unwrap()
}
