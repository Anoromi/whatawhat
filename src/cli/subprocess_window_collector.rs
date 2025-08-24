use std::{
    io::{self, Write},
    process::Stdio,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::Mutex,
    task::JoinHandle,
};
use tracing::{debug, error, warn};
use whatawhat_lib::{
    simple_cache::CacheConfig, ActiveWindowData, GenericWindowManager, WindowManager,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CurrentInformation {
    pub window_title: Arc<str>,
    pub process_path: Option<Arc<str>>,
    pub app_identifier: Option<Arc<str>>,
    pub app_name: Option<Arc<str>>,
    idle: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum CollectionResponse {
    Ok(CurrentInformation),
    Err(String),
}

async fn run_collection_subprocess(
    executable: String,
    current_data: Arc<Mutex<Option<CollectionResponse>>>,
    collection_interval: Duration,
    idle_interval: Duration,
) -> Result<()> {
    let mut command = Command::new(executable);
    command.args(&["--collect"]);
    command.args(&["--force"]);
    command.args(&["--collect-interval", &collection_interval.as_secs().to_string()]);
    command.args(&["--idle-time", &idle_interval.as_secs().to_string()]);
    command.kill_on_drop(true);
    debug!("Running command: {:?}", command);
    let stdin = Stdio::piped();
    let stdout = Stdio::piped();
    command.stdin(stdin);
    command.stdout(stdout);
    let child = command.spawn()?;
    let stdout = child.stdout.expect("Should be present");

    let mut buf_reader = BufReader::new(stdout).lines();
    debug!("Running subprocess");
    while let Ok(Some(data)) = buf_reader.next_line().await {
        debug!("Received data: {}", data);
        let data: CollectionResponse = match serde_json::from_str(&data) {
            Ok(data) => data,
            Err(e) => {
                warn!("Error parsing data: {}, message: {}", e, &data);
                continue;
            }
        };
        *current_data.lock().await = Some(data);
    }

    Ok(())
}

pub fn run_collector(
    writer: impl io::Write,
    collection_interval: Duration,
    idle_interval: Duration,
) -> Result<()> {
    let mut writer = std::io::BufWriter::new(writer);
    let mut collector = GenericWindowManager::new(
        idle_interval,
        Some(CacheConfig {
            ttl: Duration::from_secs(1 * 60 * 60),
            max_size: 200,
        }),
    )?;
    loop {
        let data = collector
            .get_active_window_data()
            .inspect_err(|e| error!("Error getting active window data: {}", e));
        let is_idle = collector
            .is_idle()
            .inspect_err(|e| error!("Error getting idle state: {}", e));
        match (data, is_idle) {
            (Ok(data), Ok(is_idle)) => {
                let data: CurrentInformation = CurrentInformation {
                    window_title: data.window_title,
                    process_path: data.process_path,
                    app_identifier: data.app_identifier,
                    app_name: data.app_name,
                    idle: is_idle,
                };
                writer
                    .write_all(serde_json::to_string(&CollectionResponse::Ok(data))?.as_bytes())?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
            (Err(e), Err(e2)) => {
                writer.write_all(
                    serde_json::to_string(&CollectionResponse::Err(format!(
                        "Error getting active window data: {} and error getting idle state: {}",
                        e, e2
                    )))?
                    .as_bytes(),
                )?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
            (Err(e), _) => {
                writer.write_all(
                    serde_json::to_string(&CollectionResponse::Err(format!(
                        "Error getting active window data: {}",
                        e
                    )))?
                    .as_bytes(),
                )?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
            (_, Err(e)) => {
                writer.write_all(
                    serde_json::to_string(&CollectionResponse::Err(format!(
                        "Error getting idle state: {}",
                        e
                    )))?
                    .as_bytes(),
                )?;
                writer.write_all(b"\n")?;
                writer.flush()?;
            }
        }
        std::thread::sleep(collection_interval);
    }
}

pub struct SubprocessWindowManager {
    current_data: Arc<Mutex<Option<CollectionResponse>>>,
    _subprocess_handle: JoinHandle<Result<()>>,
}

impl SubprocessWindowManager {

    pub fn new_with_intervals(collection_interval: Duration, idle_interval: Duration) -> Result<Self> {
        dbg!("Spawning");
        let current_data = Arc::new(Mutex::new(None));
        let current_data_clone = current_data.clone();

        let executable = std::env::current_exe()?;

        // Spawn the subprocess in a tokio task
        let subprocess_handle = tokio::spawn(async move {
            run_collection_subprocess(
                executable.to_string_lossy().to_string(),
                current_data_clone,
                collection_interval,
                idle_interval,
            ).await
        });

        Ok(Self {
            current_data,
            _subprocess_handle: subprocess_handle,
        })
    }
}

impl WindowManager for SubprocessWindowManager {
    fn get_active_window_data(&mut self) -> Result<ActiveWindowData> {
        // Get the current data from the subprocess
        let current_data = futures::executor::block_on(async {
            self.current_data.lock().await.clone()
        });

        match current_data {
            Some(CollectionResponse::Ok(data)) => Ok(ActiveWindowData {
                window_title: data.window_title,
                process_path: data.process_path,
                app_identifier: data.app_identifier,
                app_name: data.app_name,
            }),
            Some(CollectionResponse::Err(e)) => Err(anyhow::anyhow!("Subprocess error: {}", e)),
            None => Err(anyhow::anyhow!("No data available from subprocess yet")),
        }
    }

    fn is_idle(&mut self) -> Result<bool> {
        // Get the current data from the subprocess
        let current_data = futures::executor::block_on(async {
            self.current_data.lock().await.clone()
        });

        match current_data {
            Some(CollectionResponse::Ok(data)) => Ok(data.idle),
            Some(CollectionResponse::Err(e)) => Err(anyhow::anyhow!("Subprocess error: {}", e)),
            None => Ok(false), // Default to not idle if no data yet
        }
    }
}
