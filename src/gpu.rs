use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GpuStats {
	pub utilization: Option<f32>,
	pub temperature: Option<f32>,
	pub memory_used: Option<f32>,
	pub memory_total: Option<f32>,
	pub power_draw: Option<f32>,
	pub power_limit: Option<f32>,
}

impl GpuStats {
	pub fn format_metric(&self, metric: &str) -> String {
		match metric {
			"utilization" => match self.utilization {
				Some(v) => format!("GPU\n{v:.0}%"),
				None => "GPU\nN/A".to_owned(),
			},
			"temperature" => match self.temperature {
				Some(v) => format!("Temp\n{v:.0}\u{00B0}C"),
				None => "Temp\nN/A".to_owned(),
			},
			"memory" => match (self.memory_used, self.memory_total) {
				(Some(used), Some(total)) => {
					let used_gb = used / 1024.0;
					let total_gb = total / 1024.0;
					let pct = if total > 0.0 {
						used / total * 100.0
					} else {
						0.0
					};
					format!("VRAM\n{used_gb:.1}/{total_gb:.1}G\n{pct:.0}%")
				}
				_ => "VRAM\nN/A".to_owned(),
			},
			"power" => match self.power_draw {
				Some(draw) => match self.power_limit {
					Some(limit) => format!("Power\n{draw:.0}/{limit:.0}W"),
					None => format!("Power\n{draw:.0}W"),
				},
				None => "Power\nN/A".to_owned(),
			},
			_ => "N/A".to_owned(),
		}
	}
}

/// Query GPU stats via nvidia-smi for the given GPU index.
pub fn query_nvidia(gpu_index: u32) -> Result<GpuStats, anyhow::Error> {
	let output = Command::new("nvidia-smi")
		.args([
			"--query-gpu=utilization.gpu,temperature.gpu,memory.used,memory.total,power.draw,power.limit",
			"--format=csv,noheader,nounits",
			&format!("--id={gpu_index}"),
		])
		.output()?;

	if !output.status.success() {
		let stderr = String::from_utf8_lossy(&output.stderr);
		anyhow::bail!("nvidia-smi failed: {stderr}");
	}

	let stdout = String::from_utf8_lossy(&output.stdout);
	let line = stdout.trim();
	let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

	if fields.len() < 6 {
		anyhow::bail!("unexpected nvidia-smi output: {line}");
	}

	Ok(GpuStats {
		utilization: fields[0].parse().ok(),
		temperature: fields[1].parse().ok(),
		memory_used: fields[2].parse().ok(),
		memory_total: fields[3].parse().ok(),
		power_draw: fields[4].parse().ok(),
		power_limit: fields[5].parse().ok(),
	})
}

/// Query GPU stats from AMD sysfs for the given card index.
pub fn query_amd(card_index: u32) -> Result<GpuStats, anyhow::Error> {
	let base = format!("/sys/class/drm/card{card_index}/device");

	let stats = GpuStats {
		utilization: read_sysfs_f32(&format!("{base}/gpu_busy_percent")).ok(),
		temperature: read_sysfs_f32(&format!("{base}/hwmon/hwmon*/temp1_input"))
			.map(|v| v / 1000.0)
			.ok(),
		memory_used: read_sysfs_f32(&format!("{base}/mem_info_vram_used"))
			.map(|v| v / 1048576.0)
			.ok(),
		memory_total: read_sysfs_f32(&format!("{base}/mem_info_vram_total"))
			.map(|v| v / 1048576.0)
			.ok(),
		power_draw: read_sysfs_f32(&format!("{base}/hwmon/hwmon*/power1_average"))
			.map(|v| v / 1000000.0)
			.ok(),
		power_limit: read_sysfs_f32(&format!("{base}/hwmon/hwmon*/power1_cap"))
			.map(|v| v / 1000000.0)
			.ok(),
	};

	if stats.utilization.is_none()
		&& stats.temperature.is_none()
		&& stats.memory_used.is_none()
		&& stats.power_draw.is_none()
	{
		anyhow::bail!("no AMD GPU stats found at {base}");
	}

	Ok(stats)
}

/// Read a sysfs file, resolving globs (for hwmon paths).
fn read_sysfs_f32(pattern: &str) -> Result<f32, anyhow::Error> {
	let path = if pattern.contains('*') {
		glob_first(pattern)?
	} else {
		pattern.to_owned()
	};

	let content = std::fs::read_to_string(&path)?;
	Ok(content.trim().parse()?)
}

/// Return the first match for a glob pattern.
fn glob_first(pattern: &str) -> Result<String, anyhow::Error> {
	// Simple glob resolution using ls via shell
	let output = Command::new("sh")
		.args(["-c", &format!("ls -1 {pattern} 2>/dev/null | head -1")])
		.output()?;
	let path = String::from_utf8_lossy(&output.stdout).trim().to_owned();
	if path.is_empty() {
		anyhow::bail!("no match for glob: {pattern}");
	}
	Ok(path)
}

/// Auto-detect GPU vendor and query stats.
pub fn query_gpu(gpu_index: u32) -> Result<GpuStats, anyhow::Error> {
	// Try NVIDIA first (nvidia-smi is the most reliable)
	if let Ok(stats) = query_nvidia(gpu_index) {
		return Ok(stats);
	}
	// Fall back to AMD sysfs
	if let Ok(stats) = query_amd(gpu_index) {
		return Ok(stats);
	}
	anyhow::bail!("no supported GPU found (tried nvidia-smi and AMD sysfs)")
}
