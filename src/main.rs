mod gpu;

use std::collections::HashMap;
use std::sync::Arc;

use openaction::*;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

type PollTaskMap = HashMap<InstanceId, JoinHandle<()>>;

/// Map of instance_id -> polling task handle, so we can cancel on will_disappear.
static POLL_TASKS: std::sync::LazyLock<Arc<Mutex<PollTaskMap>>> =
	std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct GpuMonitorSettings {
	gpu_index: Option<u32>,
	interval: Option<u64>,
}

impl GpuMonitorSettings {
	fn gpu_index(&self) -> u32 {
		self.gpu_index.unwrap_or(0)
	}

	fn interval_secs(&self) -> u64 {
		self.interval.unwrap_or(2).max(1)
	}
}

fn start_polling(instance_id: InstanceId, metric: &'static str, settings: &GpuMonitorSettings) {
	let gpu_index = settings.gpu_index();
	let interval = settings.interval_secs();
	let instance_id_clone = instance_id.clone();

	let handle = tokio::spawn(async move {
		loop {
			let text = match gpu::query_gpu(gpu_index) {
				Ok(stats) => stats.format_metric(metric),
				Err(e) => {
					log::warn!("GPU query failed: {e}");
					format!("Error\n{}", short_error(&e))
				}
			};

			if let Some(instance) = get_instance(instance_id_clone.clone()).await {
				if let Err(e) = instance.set_title(Some(text), None).await {
					log::warn!("Failed to set title: {e}");
				}
			} else {
				break;
			}

			tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
		}
	});

	let tasks = POLL_TASKS.clone();
	tokio::spawn(async move {
		tasks.lock().await.insert(instance_id, handle);
	});
}

async fn stop_polling(instance_id: &InstanceId) {
	if let Some(handle) = POLL_TASKS.lock().await.remove(instance_id) {
		handle.abort();
	}
}

fn short_error(e: &anyhow::Error) -> String {
	let msg = e.to_string();
	if msg.len() > 20 {
		format!("{}...", &msg[..17])
	} else {
		msg
	}
}

/// Macro to define a GPU monitor action for a specific metric.
macro_rules! gpu_action {
	($name:ident, $uuid:expr, $metric:expr) => {
		pub struct $name;
		#[async_trait]
		impl Action for $name {
			const UUID: &'static str = $uuid;
			type Settings = GpuMonitorSettings;

			async fn will_appear(
				&self,
				instance: &Instance,
				settings: &Self::Settings,
			) -> OpenActionResult<()> {
				start_polling(instance.instance_id.clone(), $metric, settings);
				Ok(())
			}

			async fn will_disappear(
				&self,
				instance: &Instance,
				_settings: &Self::Settings,
			) -> OpenActionResult<()> {
				stop_polling(&instance.instance_id).await;
				Ok(())
			}

			async fn did_receive_settings(
				&self,
				instance: &Instance,
				settings: &Self::Settings,
			) -> OpenActionResult<()> {
				stop_polling(&instance.instance_id).await;
				start_polling(instance.instance_id.clone(), $metric, settings);
				Ok(())
			}

			async fn key_down(
				&self,
				instance: &Instance,
				settings: &Self::Settings,
			) -> OpenActionResult<()> {
				let gpu_index = settings.gpu_index();
				let instance_id = instance.instance_id.clone();
				tokio::spawn(async move {
					let text = match gpu::query_gpu(gpu_index) {
						Ok(stats) => stats.format_metric($metric),
						Err(e) => format!("Error\n{}", short_error(&e)),
					};
					if let Some(instance) = get_instance(instance_id).await {
						let _ = instance.set_title(Some(text), None).await;
					}
				});
				Ok(())
			}
		}
	};
}

gpu_action!(
	GpuUtilizationAction,
	"linux-gpu-monitor.utilization",
	"utilization"
);
gpu_action!(
	GpuTemperatureAction,
	"linux-gpu-monitor.temperature",
	"temperature"
);
gpu_action!(GpuMemoryAction, "linux-gpu-monitor.memory", "memory");
gpu_action!(GpuPowerAction, "linux-gpu-monitor.power", "power");

#[tokio::main]
async fn main() -> OpenActionResult<()> {
	{
		use simplelog::*;
		if let Err(error) = TermLogger::init(
			LevelFilter::Debug,
			Config::default(),
			TerminalMode::Stdout,
			ColorChoice::Never,
		) {
			eprintln!("Logger initialization failed: {}", error);
		}
	}

	register_action(GpuUtilizationAction).await;
	register_action(GpuTemperatureAction).await;
	register_action(GpuMemoryAction).await;
	register_action(GpuPowerAction).await;
	run(std::env::args().collect()).await
}
