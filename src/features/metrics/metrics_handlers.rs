use super::{MetricsCache, MetricsResponse};
use crate::features::pings::{self, PingRecord};
use sqlx::{Pool, Sqlite};
use std::ffi::CString;
use std::mem::MaybeUninit;

#[derive(Default)]
struct CpuTicks {
    total: u64,
    idle: u64,
    iowait: u64,
    cores: usize,
}

fn get_cpu_ticks() -> Option<CpuTicks> {
    let content = std::fs::read_to_string("/proc/stat").ok()?;
    let mut lines = content.lines();
    let cpu_line = lines.next()?;
    if !cpu_line.starts_with("cpu ") {
        return None;
    }
    let parts: Vec<&str> = cpu_line.split_whitespace().skip(1).collect();
    let user: u64 = parts.first()?.parse().ok()?;
    let nice: u64 = parts.get(1)?.parse().ok()?;
    let system: u64 = parts.get(2)?.parse().ok()?;
    let idle: u64 = parts.get(3)?.parse().ok()?;
    let iowait: u64 = parts.get(4)?.parse().ok()?;
    let irq: u64 = parts.get(5)?.parse().ok()?;
    let softirq: u64 = parts.get(6)?.parse().ok()?;
    let steal: u64 = parts.get(7)?.parse().ok()?;

    let idle_ticks = idle.checked_add(iowait)?;
    let non_idle_ticks = user
        .checked_add(nice)?
        .checked_add(system)?
        .checked_add(irq)?
        .checked_add(softirq)?
        .checked_add(steal)?;
    let total_ticks = idle_ticks.checked_add(non_idle_ticks)?;

    let mut cores: usize = 0;
    for line in lines {
        if line.starts_with("cpu") && !line.starts_with("cpu ") {
            cores = cores.checked_add(1)?;
        }
    }
    if cores == 0 {
        cores = 1;
    }

    Some(CpuTicks {
        total: total_ticks,
        idle: idle_ticks,
        iowait,
        cores,
    })
}

fn get_load_avg() -> Option<f64> {
    let content = std::fs::read_to_string("/proc/loadavg").ok()?;
    content.split_whitespace().next()?.parse::<f64>().ok()
}

fn get_memory_percent() -> Option<f64> {
    let content = std::fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0.0;
    let mut available = 0.0;
    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1)?.parse::<f64>().ok()?;
        } else if line.starts_with("MemAvailable:") {
            available = line.split_whitespace().nth(1)?.parse::<f64>().ok()?;
        }
    }
    if total > 0.0 {
        Some(((total - available) / total) * 100.0)
    } else {
        None
    }
}

#[allow(clippy::cast_precision_loss)]
fn get_disk_usage() -> Option<f64> {
    let c_path = CString::new("/").ok()?;
    unsafe {
        let mut stat = MaybeUninit::<libc::statvfs>::uninit();
        if libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) == 0 {
            let stat = stat.assume_init();
            let total = stat.f_blocks as f64 * stat.f_frsize as f64;
            let free = stat.f_bfree as f64 * stat.f_frsize as f64;
            if total > 0.0 {
                let used = total - free;
                Some((used / total) * 100.0)
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[allow(clippy::cast_precision_loss)]
async fn calculate_uptime(pool: &Pool<Sqlite>) -> f64 {
    let Ok(watchers) = sqlx::query!("SELECT id FROM watchers")
        .fetch_all(pool)
        .await
    else {
        return 100.0;
    };

    if watchers.is_empty() {
        return 100.0;
    }

    let now = chrono::Utc::now().naive_utc();
    let Some(start_time) = now.checked_sub_signed(chrono::Duration::days(7)) else {
        return 100.0;
    };
    let cutoff = start_time;

    let Ok(pings) = pings::get_pings_since(pool, cutoff).await else {
        return 100.0;
    };

    let total_window_secs = match now.signed_duration_since(start_time).num_seconds() {
        s if s > 0 => s as f64,
        _ => return 100.0,
    };

    let mut total_offline_secs = 0.0;

    for w in &watchers {
        let w_pings: Vec<&PingRecord> = pings.iter().filter(|p| p.watcher_id == w.id).collect();

        let mut offline_secs = 0.0;
        let mut current_state = "online";
        let mut last_time = start_time;

        if !w_pings.is_empty() {
            if w_pings[0].status == "offline" {
                current_state = "online";
            } else {
                current_state = "offline";
            }

            for p in &w_pings {
                let p_time = p.timestamp;
                let diff_secs = p_time.signed_duration_since(last_time).num_seconds().max(0) as f64;
                if current_state == "offline" {
                    offline_secs += diff_secs;
                }
                current_state = &p.status;
                last_time = p_time;
            }
        }

        let final_secs = now.signed_duration_since(last_time).num_seconds().max(0) as f64;
        if current_state == "offline" {
            offline_secs += final_secs;
        }

        total_offline_secs += offline_secs;
    }

    let total_possible_secs = total_window_secs * watchers.len() as f64;
    if total_possible_secs <= 0.0 {
        return 100.0;
    }
    let uptime = ((total_possible_secs - total_offline_secs) / total_possible_secs) * 100.0;
    uptime.clamp(0.0, 100.0)
}

// Struct to keep previous ticks for calculation across updates
#[derive(Default)]
pub struct MetricsState {
    prev_ticks: Option<CpuTicks>,
}

#[allow(clippy::cast_precision_loss)]
pub async fn update_metrics(
    pool: &Pool<Sqlite>,
    metrics_cache: &MetricsCache,
    state: &mut MetricsState,
) {
    let load_avg = get_load_avg().unwrap_or(0.0);
    let memory_percent = get_memory_percent().unwrap_or(0.0);
    let disk_percent = get_disk_usage().unwrap_or(0.0);

    let mut cpu_percent = 0.0;
    let mut cpu_usage = 0.0;
    let mut io_percent = 0.0;

    let current_ticks = get_cpu_ticks();
    if let (Some(prev), Some(curr)) = (&state.prev_ticks, &current_ticks) {
        let diff_total = curr.total.saturating_sub(prev.total);
        let diff_idle = curr.idle.saturating_sub(prev.idle);
        let diff_iowait = curr.iowait.saturating_sub(prev.iowait);

        if diff_total > 0 {
            cpu_percent =
                ((diff_total.saturating_sub(diff_idle)) as f64 / diff_total as f64) * 100.0;
            cpu_usage = cpu_percent * curr.cores as f64;
            io_percent = (diff_iowait as f64 / diff_total as f64) * 100.0;
        }
    }
    state.prev_ticks = current_ticks;

    let uptime_percent = calculate_uptime(pool).await;

    let metrics = MetricsResponse {
        uptime_percent,
        load_avg,
        memory_percent,
        cpu_percent,
        cpu_usage,
        io_percent,
        disk_percent,
    };

    metrics_cache.set_payload(metrics);
}
