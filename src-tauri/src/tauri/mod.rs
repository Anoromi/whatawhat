use std::{path::PathBuf, sync::Arc};

use anyhow::Result;
use specta_typescript::Typescript;
use tauri::Manager as _;
use tauri_specta::collect_commands;
use tokio::sync::Mutex;
use tracing::error;

use crate::{
    daemon::{start_daemon_with_intervals, storage::record_storage::RecordStorageImpl},
    utils::dir::create_application_default_path,
};

pub mod commands;

pub fn run_ui(
    app_dir: Option<PathBuf>,
    collection_interval: std::time::Duration,
    idle_interval: std::time::Duration,
) -> Result<()> {
    let app_dir = app_dir.map_or_else(create_application_default_path, Ok)?;
    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .commands(collect_commands![commands::get_timeline_window_data, commands::get_timeline_app_data]);

    specta_builder
        .export(Typescript::default().bigint(specta_typescript::BigIntExportBehavior::Number), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .invoke_handler(specta_builder.invoke_handler())
        .setup(move |app| {
            specta_builder.mount_events(app);
            // if cfg!(debug_assertions) {
            // app.handle().plugin(
            //   tauri_plugin_log::Builder::default()
            //     .level(log::LevelFilter::Info)
            //     .build(),
            // )?;
            // }
            let ui_app = UiApp::new(app_dir, collection_interval, idle_interval).unwrap();
            app.manage(ui_app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::get_timeline_window_data, commands::get_timeline_app_data])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    Ok(())
}

pub struct UiApp {
    collection_handle: Arc<Mutex<tauri::async_runtime::JoinHandle<()>>>,
    record_storage: RecordStorageImpl,
}
impl UiApp {
    pub fn new(
        app_dir: PathBuf,
        collection_interval: std::time::Duration,
        idle_interval: std::time::Duration,
    ) -> Result<Self> {
        let record_storage =
            RecordStorageImpl::new(create_application_default_path()?.join("records"))?;

        Ok(Self {
            collection_handle: Arc::new(Mutex::new(tauri::async_runtime::spawn(async move {
                if let Err(e) =
                    start_daemon_with_intervals(app_dir, collection_interval, idle_interval).await
                {
                    error!("Error running daemon: {:?}", e);
                }
            }))),
            record_storage
        })
    }
}
