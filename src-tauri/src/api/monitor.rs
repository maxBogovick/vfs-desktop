use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use sysinfo::{System, Networks, Disks, CpuRefreshKind, RefreshKind, MemoryRefreshKind, Process, ProcessRefreshKind, Pid};
use serde::Serialize;
use tokio::sync::Notify;

// --- Data Structures ---

#[derive(Clone, Serialize, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub user: Option<String>,
    pub status: String,
}

#[derive(Clone, Serialize, Debug)]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub cpu_global_usage: f32,
    pub cpu_cores_usage: Vec<f32>,
    pub ram_used: u64,
    pub ram_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
    pub process_count: usize,
    pub top_processes: Vec<ProcessInfo>, // Top 100 by CPU
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", content = "metadata")]
pub enum AnomalyType {
    CpuSpike { value: f32 },
    MemoryLeak { start_val: u64, current_val: u64 },
    NetworkSurge { current: u64, avg: u64 },
    DiskLatency, // Placeholder
}

#[derive(Clone, Serialize, Debug)]
pub struct AnomalyEvent {
    pub timestamp: u64,
    #[serde(flatten)]
    pub anomaly_type: AnomalyType,
    pub severity: String, // "WARN" | "CRITICAL"
}

// --- Logic & State ---

struct AnomalyDetector {
    history: VecDeque<SystemSnapshot>,
    cpu_ema: f32,
    net_rx_ema: f32,
    net_tx_ema: f32,
    cpu_spike_start: Option<Instant>,
    memory_monotonic_start: Option<(Instant, u64)>,
}

impl AnomalyDetector {
    fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(1200),
            cpu_ema: 0.0,
            net_rx_ema: 0.0,
            net_tx_ema: 0.0,
            cpu_spike_start: None,
            memory_monotonic_start: None,
        }
    }

    fn update_ema(&mut self, snap: &SystemSnapshot) {
        let alpha = 2.0 / (20.0 + 1.0);
        if self.history.is_empty() {
            self.cpu_ema = snap.cpu_global_usage;
            self.net_rx_ema = snap.network_rx_bytes as f32;
            self.net_tx_ema = snap.network_tx_bytes as f32;
        } else {
            self.cpu_ema = alpha * snap.cpu_global_usage + (1.0 - alpha) * self.cpu_ema;
            self.net_rx_ema = alpha * (snap.network_rx_bytes as f32) + (1.0 - alpha) * self.net_rx_ema;
            self.net_tx_ema = alpha * (snap.network_tx_bytes as f32) + (1.0 - alpha) * self.net_tx_ema;
        }
    }

    fn check_anomalies(&mut self, snap: &SystemSnapshot) -> Vec<AnomalyEvent> {
        let mut events = Vec::new();
        let now = Instant::now();

        if snap.cpu_global_usage > 85.0 {
            if let Some(start) = self.cpu_spike_start {
                if now.duration_since(start) >= Duration::from_secs(30) {
                     events.push(AnomalyEvent {
                        timestamp: snap.timestamp,
                        anomaly_type: AnomalyType::CpuSpike { value: snap.cpu_global_usage },
                        severity: "CRITICAL".to_string(),
                     });
                }
            } else {
                self.cpu_spike_start = Some(now);
            }
        } else {
            self.cpu_spike_start = None;
        }

        if let Some(prev) = self.history.back() {
            if snap.ram_used >= prev.ram_used {
                 if let Some((start_time, start_val)) = self.memory_monotonic_start {
                     if now.duration_since(start_time) >= Duration::from_secs(300) {
                         events.push(AnomalyEvent {
                            timestamp: snap.timestamp,
                            anomaly_type: AnomalyType::MemoryLeak { start_val, current_val: snap.ram_used },
                            severity: "WARN".to_string(),
                         });
                     }
                 } else {
                     self.memory_monotonic_start = Some((now, snap.ram_used));
                 }
            } else {
                 self.memory_monotonic_start = None;
            }
        }

        if self.net_rx_ema > 1024.0 {
            if (snap.network_rx_bytes as f32) > (self.net_rx_ema * 5.0) {
                events.push(AnomalyEvent {
                    timestamp: snap.timestamp,
                    anomaly_type: AnomalyType::NetworkSurge { current: snap.network_rx_bytes, avg: self.net_rx_ema as u64 },
                    severity: "WARN".to_string(),
                });
            }
        }

        self.update_ema(snap);
        self.history.push_back(snap.clone());
        if self.history.len() > 100 {
            self.history.pop_front();
        }

        events
    }
}

pub struct MonitorState {
    pub is_running: Arc<Mutex<bool>>,
    pub stop_notify: Arc<Notify>,
}

impl MonitorState {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(Mutex::new(false)),
            stop_notify: Arc::new(Notify::new()),
        }
    }
}

// --- Commands ---

#[tauri::command]
pub async fn start_monitoring<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, MonitorState>,
) -> Result<(), String> {
    let mut running = state.is_running.lock().map_err(|_| "Failed to lock state")?;
    if *running {
        return Ok(());
    }
    *running = true;
    drop(running);

    let stop_notify = state.stop_notify.clone();
    let is_running_clone = state.is_running.clone();

    tauri::async_runtime::spawn(async move {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything())
        );
        let mut networks = Networks::new_with_refreshed_list();
        
        let mut detector = AnomalyDetector::new();

        loop {
            if !*is_running_clone.lock().unwrap() {
                break;
            }

            sys.refresh_cpu_all();
            sys.refresh_memory();
            sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
            networks.refresh(true); 

            let mut net_rx = 0;
            let mut net_tx = 0;
            for (_, network) in &networks {
                net_rx += network.received();
                net_tx += network.transmitted();
            }

            // Top Processes
            let mut procs: Vec<ProcessInfo> = sys.processes().iter()
                .map(|(pid, p)| {
                    ProcessInfo {
                        pid: pid.as_u32(),
                        name: p.name().to_string_lossy().to_string(),
                        cpu_usage: p.cpu_usage(),
                        memory: p.memory(),
                        user: p.user_id().map(|uid| uid.to_string()),
                        status: format!("{:?}", p.status()),
                    }
                })
                .collect();
            
            // Sort by CPU usage descending
            procs.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
            
            // Take top 100
            let top_procs = procs.into_iter().take(100).collect();

            let snap = SystemSnapshot {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                cpu_global_usage: sys.global_cpu_usage(),
                cpu_cores_usage: sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect(),
                ram_used: sys.used_memory(),
                ram_total: sys.total_memory(),
                swap_used: sys.used_swap(),
                swap_total: sys.total_swap(),
                network_rx_bytes: net_rx,
                network_tx_bytes: net_tx,
                process_count: sys.processes().len(),
                top_processes: top_procs,
            };

            let anomalies = detector.check_anomalies(&snap);

            if let Err(e) = app.emit("sys_mon_update", &snap) {
                eprintln!("Failed to emit update: {}", e);
            }
            if !anomalies.is_empty() {
                if let Err(e) = app.emit("sys_mon_anomaly", &anomalies) {
                    eprintln!("Failed to emit anomaly: {}", e);
                }
            }

            tokio::select! {
                _ = stop_notify.notified() => {
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(500)) => {}
            }
        }
        
        let mut running = is_running_clone.lock().unwrap();
        *running = false;
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_monitoring(state: State<'_, MonitorState>) -> Result<(), String> {
    let mut running = state.is_running.lock().map_err(|_| "Failed to lock state")?;
    if *running {
        *running = false;
        state.stop_notify.notify_one();
    }
    Ok(())
}

#[tauri::command]
pub async fn kill_process(pid: u32) -> Result<(), String> {
    let s = System::new_all();
    let pid_u = Pid::from_u32(pid); // sysinfo 0.33 might use `Pid::from(usize)` or `u32` depending on platform
    // On unix Pid is usually wrapper around pid_t
    // sysinfo::Pid is generic on some versions or wrapper.
    // In sysinfo 0.30+, Pid::from_u32 is correct or Pid::from(pid as usize).
    // Let's assume generic cross platform way.
    
    // Actually we need to find the process first to kill it instance-based?
    // Or send signal.
    
    // System::new_all() refreshes everything which is heavy. 
    // We just want to kill. 
    // process::Process::kill_with(Signal::Kill) on instance?
    // or System::process(pid).kill()
    
    // Simpler: 
    // Need to check sysinfo docs for `Pid`.
    // It seems `Pid::from(u32)` or similar.
    
    // Let's try constructing a system, refresh processes, find pid, kill.
    // This is safe.
    
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    
    // Pid construction:
    // sysinfo::Pid depends on platform.
    // But usually simple cast works if `Pid` implements `From<usize>`.
    let target_pid = Pid::from(pid as usize); 
    
    if let Some(process) = sys.process(target_pid) {
        if process.kill() {
            Ok(())
        } else {
            Err("Failed to kill process".to_string())
        }
    } else {
        Err("Process not found".to_string())
    }
}