mod ai_engine;
mod commands;
mod drawing;
mod embeddings;
mod fs_manager;
mod models;
mod search;
mod stt_engine;
mod watcher;

use commands::{
    ai_commands::*, drawing_commands::*, fs_commands::*, search_commands::*, stt_commands::*,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Run the Tauri application.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            open_vault,
            restore_vault,
            get_file_tree,
            read_note,
            save_note,
            create_note,
            delete_item,
            rename_item,
            create_folder,
            import_asset,
            resolve_asset,
            build_search_index,
            build_embeddings,
            search_notes,
            search_semantic,
            load_ai_model,
            unload_ai_model,
            get_ai_status,
            detect_npu,
            download_ai_model,
            summarize_note,
            ask_question,
            detect_reminders,
            suggest_tags,
            apply_ai_proposal,
            start_lecture_mode,
            stop_lecture_mode,
            list_audio_devices,
            set_audio_device,
            set_stt_model_path,
            render_drawing_png,
            save_drawing,
            load_drawing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
