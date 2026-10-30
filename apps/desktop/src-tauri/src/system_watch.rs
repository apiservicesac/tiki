use std::thread;
use std::time::Instant;

use serde::Serialize;
use sysinfo::{
    Components, CpuRefreshKind, MemoryRefreshKind, NetworkData, Networks, RefreshKind, System,
};
use tauri::{AppHandle, Manager};

use crate::app_state::AppState;
use crate::desktop_config::DesktopConfig;
use crate::shell_events::ShellEvent;

/// How the computer is doing, for the resting notch when nothing is pending.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStats {
    pub cpu_percent: f32,
    pub memory_percent: u64,
    /// `None` where the computer does not expose a temperature sensor.
    pub temperature_celsius: Option<f32>,
    pub download_bytes_per_second: u64,
}

/// Measures the computer every few seconds while the user wants it shown, and sends each
/// measure to the island.
pub struct SystemWatch;

impl SystemWatch {
    pub fn start(app: AppHandle) {
        let mut sampler = match SystemSampler::new() {
            Ok(sampler) => sampler,
            Err(error) => {
                tracing::warn!(%error, "could not read the computer's state");
                return;
            }
        };
        thread::spawn(move || {
            loop {
                thread::sleep(DesktopConfig::SYSTEM_STATS_INTERVAL);
                let wanted = app
                    .state::<AppState>()
                    .with_workspace(|workspace| Ok(workspace.preferences().show_system_stats));
                if matches!(wanted, Ok(true)) {
                    ShellEvent::SystemStats.emit_with(&app, &sampler.sample());
                }
            }
        });
    }
}

/// Reads CPU and memory use, the hottest CPU sensor and how fast data comes in.
struct SystemSampler {
    system: System,
    components: Components,
    networks: Networks,
    sampled_at: Instant,
}

impl SystemSampler {
    /// Sensor names that belong to the processor, preferred over the others.
    const CPU_SENSOR_HINTS: [&'static str; 4] = ["cpu", "package", "tctl", "core"];

    fn new() -> Result<Self, sysinfo::Error> {
        let refreshes = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        Ok(Self {
            system: System::new_with_specifics(refreshes)?,
            components: Components::new_with_refreshed_list()?,
            networks: Networks::new_with_refreshed_list()?,
            sampled_at: Instant::now(),
        })
    }

    fn sample(&mut self) -> SystemStats {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.components.refresh(false);
        self.networks.refresh(true);
        let elapsed_ms = self.sampled_at.elapsed().as_millis().max(1);
        self.sampled_at = Instant::now();
        let received: u64 = self.networks.values().map(NetworkData::received).sum();
        SystemStats {
            cpu_percent: self.system.global_cpu_usage(),
            memory_percent: self.system.used_memory() * 100 / self.system.total_memory().max(1),
            temperature_celsius: self.temperature(),
            download_bytes_per_second: u64::try_from(u128::from(received) * 1000 / elapsed_ms)
                .unwrap_or(u64::MAX),
        }
    }

    /// The hottest processor sensor, or the hottest sensor when none says it is the processor.
    fn temperature(&self) -> Option<f32> {
        let readings = || {
            self.components.iter().filter_map(|component| {
                Some((component.label().to_lowercase(), component.temperature()?))
            })
        };
        readings()
            .filter(|(label, _)| {
                Self::CPU_SENSOR_HINTS
                    .iter()
                    .any(|hint| label.contains(hint))
            })
            .map(|(_, celsius)| celsius)
            .reduce(f32::max)
            .or_else(|| readings().map(|(_, celsius)| celsius).reduce(f32::max))
    }
}
