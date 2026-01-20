use std::collections::{VecDeque, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Runtime, State};
use sysinfo::{System, Networks, CpuRefreshKind, RefreshKind, MemoryRefreshKind, ProcessRefreshKind, Pid, Signal};
use serde::Serialize;
use tokio::sync::Notify;

// --- Data Structures ---

#[derive(Clone, Serialize, Debug)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub disk_read_speed: u64,  // Bytes per second
    pub disk_write_speed: u64, // Bytes per second
    pub user: Option<String>,
    pub status: String, // "Run", "Sleep", "Idle", "Stop" (Suspended)
    pub exe_path: Option<String>,
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
    pub top_processes: Vec<ProcessInfo>, 
}

#[derive(Clone, Serialize, Debug)]
#[serde(tag = "type", content = "metadata")]
pub enum AnomalyType {
    CpuSpike { 
        value: f32,
        culprit: Option<String>,
        culprit_pid: Option<u32>,
        confidence: f32
    },
    MemoryLeak { 
        process_name: String,
        pid: u32,
        rate_mb_per_sec: f64,
        confidence: f64
    },
    NetworkSurge { 
        current: u64, 
        avg: u64,
        culprit: Option<String> 
    },
    SystemSaturation {
        resource: String,
        usage_percent: f32,
        confidence: f32
    },
}

#[derive(Clone, Serialize, Debug)]
pub struct AnomalyEvent {
    pub timestamp: u64,
    #[serde(flatten)]
    pub anomaly_type: AnomalyType,
    pub severity: String,
}

// --- Logic & State ---

struct ProcessTracker {
    // PID -> RingBuffer of Memory Usage (last 60 samples ~ 30s)
    mem_history: HashMap<u32, VecDeque<u64>>,
}

impl ProcessTracker {
    fn new() -> Self {
        Self { mem_history: HashMap::new() }
    }

    fn update(&mut self, procs: &[ProcessInfo]) {
        let current_pids: Vec<u32> = procs.iter().map(|p| p.pid).collect();
        
        // Remove dead processes
        self.mem_history.retain(|pid, _| current_pids.contains(pid));

        // Update history for top memory consumers (optimization: only track top 50)
        for p in procs {
            let history = self.mem_history.entry(p.pid).or_insert_with(|| VecDeque::with_capacity(60));
            history.push_back(p.memory);
            if history.len() > 60 {
                history.pop_front();
            }
        }
    }

    // Returns slope (bytes per tick), confidence (0.0-1.0)
    fn detect_leak(&self, pid: u32) -> (f64, f64) {
        if let Some(history) = self.mem_history.get(&pid) {
            if history.len() < 30 { return (0.0, 0.0); }
            
            // Simple linear regression: y = mx + c
            let n = history.len() as f64;
            let mut sum_x = 0.0;
            let mut sum_y = 0.0;
            let mut sum_xy = 0.0;
            let mut sum_xx = 0.0;

            for (i, &val) in history.iter().enumerate() {
                let x = i as f64;
                let y = val as f64;
                sum_x += x;
                sum_y += y;
                sum_xy += x * y;
                sum_xx += x * x;
            }

            let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
            
            // Check if monotonic
            let mut monotonic_count = 0;
            let len = history.len();
            // Use manual iteration instead of windows() which VecDeque doesn't support directly
            for i in 0..len-1 {
                if history[i+1] >= history[i] {
                    monotonic_count += 1;
                }
            }
            
            let confidence = monotonic_count as f64 / (n - 1.0);

            (slope, confidence)
        } else {
            (0.0, 0.0)
        }
    }
}

struct AnomalyDetector {
    // Global metrics history for Z-Score
    cpu_history: VecDeque<f32>,
    ram_history: VecDeque<u64>,
    net_rx_history: VecDeque<u64>,
    
    // Track per-process metrics
    process_tracker: ProcessTracker,
    
    // Debounce alerts
    last_alert_time: HashMap<String, Instant>,
}

impl AnomalyDetector {
    fn new() -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(300), // 2.5 mins history
            ram_history: VecDeque::with_capacity(300),
            net_rx_history: VecDeque::with_capacity(300),
            process_tracker: ProcessTracker::new(),
            last_alert_time: HashMap::new(),
        }
    }

    fn calculate_z_score_f32(history: &VecDeque<f32>, value: f32) -> f32 {
        if history.len() < 30 { return 0.0; }
        let sum: f32 = history.iter().sum();
        let mean = sum / history.len() as f32;
        let variance: f32 = history.iter().map(|&x| (x - mean).powi(2)).sum::<f32>() / history.len() as f32;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 { 0.0 } else { (value - mean) / std_dev }
    }

    fn calculate_z_score_u64(history: &VecDeque<u64>, value: u64) -> f32 {
        if history.len() < 30 { return 0.0; }
        let sum: u64 = history.iter().sum();
        let mean = sum as f32 / history.len() as f32;
        let variance: f32 = history.iter().map(|&x| (x as f32 - mean).powi(2)).sum::<f32>() / history.len() as f32;
        let std_dev = variance.sqrt();
        if std_dev == 0.0 { 0.0 } else { (value as f32 - mean) / std_dev }
    }

    fn should_alert(&mut self, key: &str, cooldown: Duration) -> bool {
        let now = Instant::now();
        if let Some(last) = self.last_alert_time.get(key) {
            if now.duration_since(*last) < cooldown {
                return false;
            }
        }
        self.last_alert_time.insert(key.to_string(), now);
        true
    }

    fn check_anomalies(&mut self, snap: &SystemSnapshot) -> Vec<AnomalyEvent> {
        let mut events = Vec::new();
        self.process_tracker.update(&snap.top_processes);

        // Update Global History
        self.cpu_history.push_back(snap.cpu_global_usage);
        if self.cpu_history.len() > 300 { self.cpu_history.pop_front(); }
        
        self.ram_history.push_back(snap.ram_used);
        if self.ram_history.len() > 300 { self.ram_history.pop_front(); }

        self.net_rx_history.push_back(snap.network_rx_bytes);
        if self.net_rx_history.len() > 300 { self.net_rx_history.pop_front(); }

        // --- 1. CPU Anomaly (Statistical) ---
        let cpu_z = Self::calculate_z_score_f32(&self.cpu_history, snap.cpu_global_usage);
        if (cpu_z > 3.0 && snap.cpu_global_usage > 50.0) || snap.cpu_global_usage > 90.0 {
            if self.should_alert("cpu_high", Duration::from_secs(10)) {
                // Attribution
                let culprit = snap.top_processes.iter()
                    .max_by(|a, b| a.cpu_usage.partial_cmp(&b.cpu_usage).unwrap());
                
                let metadata = if let Some(proc) = culprit {
                    if proc.cpu_usage > (snap.cpu_global_usage * 0.4) {
                        AnomalyType::CpuSpike { 
                            value: snap.cpu_global_usage,
                            culprit: Some(proc.name.clone()),
                            culprit_pid: Some(proc.pid),
                            confidence: 0.9 
                        }
                    } else {
                        AnomalyType::CpuSpike { 
                            value: snap.cpu_global_usage, 
                            culprit: None, culprit_pid: None, confidence: 0.6 
                        }
                    }
                } else {
                    AnomalyType::CpuSpike { 
                        value: snap.cpu_global_usage, 
                        culprit: None, culprit_pid: None, confidence: 0.5 
                    }
                };

                events.push(AnomalyEvent {
                    timestamp: snap.timestamp,
                    anomaly_type: metadata,
                    severity: if snap.cpu_global_usage > 90.0 { "CRITICAL".into() } else { "WARN".into() },
                });
            }
        }

        // --- 2. Memory Leak Detection (Regression) ---
        // Scan top processes for leaks
        for proc in snap.top_processes.iter().take(10) {
            let (slope, confidence) = self.process_tracker.detect_leak(proc.pid);
            // Increased Threshold: > 500KB per 0.5s (1MB/s) to reduce noise
            if slope > 256000.0 && confidence > 0.9 {
                if self.should_alert(&format!("leak_{}", proc.pid), Duration::from_secs(120)) {
                    events.push(AnomalyEvent {
                        timestamp: snap.timestamp,
                        anomaly_type: AnomalyType::MemoryLeak { 
                            process_name: proc.name.clone(),
                            pid: proc.pid,
                            rate_mb_per_sec: (slope * 2.0) / 1024.0 / 1024.0, // tick is 0.5s
                            confidence 
                        },
                        severity: "WARN".into(),
                    });
                }
            }
        }

        // --- 3. Global RAM Saturation (Z-Score) ---
        let ram_z = Self::calculate_z_score_u64(&self.ram_history, snap.ram_used);
        let ram_percent = (snap.ram_used as f64 / snap.ram_total as f64) * 100.0;
        
        if (ram_z > 3.0 && ram_percent > 80.0) || ram_percent > 95.0 {
             if self.should_alert("ram_high", Duration::from_secs(15)) {
                 events.push(AnomalyEvent {
                    timestamp: snap.timestamp,
                    anomaly_type: AnomalyType::SystemSaturation { 
                        resource: "RAM".into(),
                        usage_percent: ram_percent as f32,
                        confidence: 1.0 
                    },
                    severity: "CRITICAL".into(),
                 });
             }
        }

        // --- 4. Network Surge (Z-Score) ---
        let net_z = Self::calculate_z_score_u64(&self.net_rx_history, snap.network_rx_bytes);
        // Alert if Z > 4 (very unusual) and speed > 1MB/s (ignore background noise)
        if net_z > 4.0 && snap.network_rx_bytes > 1_048_576 {
            if self.should_alert("net_surge", Duration::from_secs(30)) {
                // Determine average from history for comparison
                let sum: u64 = self.net_rx_history.iter().sum();
                let avg = if !self.net_rx_history.is_empty() { sum / self.net_rx_history.len() as u64 } else { 0 };
                
                events.push(AnomalyEvent {
                    timestamp: snap.timestamp,
                    anomaly_type: AnomalyType::NetworkSurge { 
                        current: snap.network_rx_bytes, 
                        avg, 
                        culprit: None // Hard to attribute network to process without Packet inspection
                    },
                    severity: "WARN".into(),
                });
            }
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

// Helper to calculate disk delta
struct ProcessDiskStats {
    prev_read: u64,
    prev_write: u64,
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
        
        // Cache for calculating disk I/O speed (Bytes/sec)
        // Map<Pid, (prev_read, prev_write)>
        let mut disk_cache: HashMap<u32, ProcessDiskStats> = HashMap::new();

        loop {
            if !*is_running_clone.lock().unwrap() {
                break;
            }

            let start_time = Instant::now();

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
                    let pid_u32 = pid.as_u32();
                    let disk = p.disk_usage();
                    
                    // Calculate speed
                    let (read_speed, write_speed) = if let Some(prev) = disk_cache.get(&pid_u32) {
                        let r = disk.read_bytes.saturating_sub(prev.prev_read);
                        let w = disk.written_bytes.saturating_sub(prev.prev_write);
                        // Multiply by 2 because interval is 500ms? 
                        // Actually logic is: Delta Bytes per Interval.
                        // If interval is 500ms, bytes/sec = delta * 2.
                        (r * 2, w * 2) 
                    } else {
                        (0, 0)
                    };

                    // Update cache
                    disk_cache.insert(pid_u32, ProcessDiskStats {
                        prev_read: disk.read_bytes,
                        prev_write: disk.written_bytes,
                    });

                    ProcessInfo {
                        pid: pid_u32,
                        name: p.name().to_string_lossy().to_string(),
                        cpu_usage: p.cpu_usage(),
                        memory: p.memory(),
                        disk_read_speed: read_speed,
                        disk_write_speed: write_speed,
                        user: p.user_id().map(|uid| uid.to_string()),
                        status: format!("{:?}", p.status()),
                        exe_path: p.exe().map(|path| path.to_string_lossy().to_string()),
                    }
                })
                .collect();
            
            // Clean up cache for dead processes
            disk_cache.retain(|k, _| sys.processes().contains_key(&Pid::from_u32(*k)));

            // Sort by CPU usage descending
            procs.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
            
            let top_procs = procs.into_iter().take(150).collect(); // Increase limit slightly

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

            // Accurate sleep
            let elapsed = start_time.elapsed();
            if elapsed < Duration::from_millis(500) {
                tokio::select! {
                    _ = stop_notify.notified() => { break; }
                    _ = tokio::time::sleep(Duration::from_millis(500) - elapsed) => {}
                }
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

// Robust process finding
fn get_process_instance(pid: u32) -> Result<(System, Pid), String> {
    // Use new() instead of new_all() for performance, only refreshing processes
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    
    let target_pid = Pid::from_u32(pid);
    
    if sys.processes().contains_key(&target_pid) {
        Ok((sys, target_pid))
    } else {
        println!("Process with PID {} not found during action", pid);
        Err(format!("Process with PID {} not found", pid))
    }
}

#[tauri::command]
pub async fn kill_process(pid: u32) -> Result<(), String> {
    println!("Attempting to kill process PID: {}", pid);
    
    // Try sysinfo first
    let sys_kill_result = if let Ok((sys, target_pid)) = get_process_instance(pid) {
        if let Some(process) = sys.process(target_pid) {
            process.kill()
        } else {
            false
        }
    } else {
        false
    };

    if sys_kill_result {
        println!("Sysinfo kill signal sent to PID {}", pid);
        return Ok(());
    }

    println!("Sysinfo kill failed or process not found. Attempting 'kill -9' fallback...");

    // Fallback to system command
    #[cfg(unix)]
    {
        use std::process::Command;
        match Command::new("kill").arg("-9").arg(pid.to_string()).output() {
            Ok(output) => {
                if output.status.success() {
                    println!("'kill -9' successful for PID {}", pid);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    println!("'kill -9' failed: {}", err);
                    Err(format!("Failed to kill PID {}: {}", pid, err))
                }
            }
            Err(e) => Err(format!("Failed to execute kill command: {}", e)),
        }
    }
    #[cfg(not(unix))]
    {
         Err(format!("Failed to kill PID {} and no fallback available on non-unix", pid))
    }
}

#[tauri::command]
pub async fn suspend_process(pid: u32) -> Result<(), String> {
    println!("Attempting to suspend process PID: {}", pid);
    
    // Try sysinfo first
    let sys_suspend_result = if let Ok((sys, target_pid)) = get_process_instance(pid) {
        if let Some(process) = sys.process(target_pid) {
            #[cfg(unix)]
            { process.kill_with(Signal::Stop).unwrap_or(false) }
            #[cfg(not(unix))]
            { false }
        } else {
            false
        }
    } else {
        false
    };

    if sys_suspend_result {
        println!("Sysinfo suspend signal sent to PID {}", pid);
        return Ok(());
    }

    println!("Sysinfo suspend failed. Attempting 'kill -STOP' fallback...");

    #[cfg(unix)]
    {
        use std::process::Command;
        match Command::new("kill").arg("-STOP").arg(pid.to_string()).output() {
             Ok(output) => {
                if output.status.success() {
                    println!("'kill -STOP' successful for PID {}", pid);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Failed to suspend PID {}: {}", pid, err))
                }
            }
            Err(e) => Err(format!("Failed to execute suspend command: {}", e)),
        }
    }
    #[cfg(not(unix))]
    {
         Err("Suspend only supported on Unix-like systems".to_string())
    }
}

#[tauri::command]
pub async fn resume_process(pid: u32) -> Result<(), String> {
    println!("Attempting to resume process PID: {}", pid);

    // Try sysinfo first
    let sys_resume_result = if let Ok((sys, target_pid)) = get_process_instance(pid) {
        if let Some(process) = sys.process(target_pid) {
             #[cfg(unix)]
            { process.kill_with(Signal::Continue).unwrap_or(false) }
            #[cfg(not(unix))]
            { false }
        } else {
            false
        }
    } else {
        false
    };

    if sys_resume_result {
        println!("Sysinfo resume signal sent to PID {}", pid);
        return Ok(());
    }

    println!("Sysinfo resume failed. Attempting 'kill -CONT' fallback...");

    #[cfg(unix)]
    {
        use std::process::Command;
        match Command::new("kill").arg("-CONT").arg(pid.to_string()).output() {
             Ok(output) => {
                if output.status.success() {
                    println!("'kill -CONT' successful for PID {}", pid);
                    Ok(())
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    Err(format!("Failed to resume PID {}: {}", pid, err))
                }
            }
            Err(e) => Err(format!("Failed to execute resume command: {}", e)),
        }
    }
    #[cfg(not(unix))]
    {
         Err("Resume only supported on Unix-like systems".to_string())
    }
}