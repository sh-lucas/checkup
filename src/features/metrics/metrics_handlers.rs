use super::{MetricsCache, MetricsResponse};
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

fn get_cpu_pressure() -> Option<(f64, u64)> {
    let content = std::fs::read_to_string("/proc/pressure/cpu").ok()?;
    for line in content.lines() {
        if line.starts_with("some ") {
            let mut avg10 = None;
            let mut total = None;
            for part in line.split_whitespace().skip(1) {
                if let Some(val_str) = part.strip_prefix("avg10=") {
                    avg10 = val_str.parse::<f64>().ok();
                } else if let Some(val_str) = part.strip_prefix("total=") {
                    total = val_str.parse::<u64>().ok();
                }
            }
            if let (Some(avg), Some(tot)) = (avg10, total) {
                return Some((avg, tot));
            }
        }
    }
    None
}

#[allow(clippy::cast_precision_loss)]
async fn calculate_uptime(pool: &Pool<Sqlite>) -> f64 {
    let now = chrono::Utc::now();
    let Some(cutoff) = now.checked_sub_signed(chrono::Duration::days(7)) else {
        return 100.0;
    };

    // Query events
    let Ok(events) = sqlx::query!(
        r#"SELECT event as "event!", timestamp as "timestamp: chrono::DateTime<chrono::Utc>"
           FROM system_uptime_events
           WHERE reference = 'system' AND timestamp >= ?
           ORDER BY timestamp ASC, id ASC"#,
        cutoff
    )
    .fetch_all(pool)
    .await
    else {
        return 100.0;
    };

    if events.is_empty() {
        return 100.0;
    }

    let first_timestamp = events.first().map_or(now, |e| e.timestamp);
    let start_time = first_timestamp.max(cutoff);

    let mut total_online_secs = 0.0;
    let mut last_time = start_time;

    for e in &events {
        let e_time = e.timestamp.clamp(start_time, now);
        let duration = e_time.signed_duration_since(last_time).num_seconds().max(0) as f64;

        if e.event == 0 {
            // event = 0 means online_until
            total_online_secs += duration;
        }
        last_time = e_time;
    }

    // Trailing time after the last event until now
    if let Some(last_event) = events.last() {
        let gap = now
            .signed_duration_since(last_event.timestamp)
            .num_seconds()
            .max(0);
        if last_event.event == 0 {
            // online_until: if within 9 minutes, we count it as online
            if gap <= 540 {
                // 9 minutes = 540 seconds
                let duration = now.signed_duration_since(last_time).num_seconds().max(0) as f64;
                total_online_secs += duration;
            }
        } else {
            // offline_until: means the server is back online since the last_event.timestamp (which was offline_until t_offline, implying uptime resumed then)
            let duration = now.signed_duration_since(last_time).num_seconds().max(0) as f64;
            total_online_secs += duration;
        }
    }

    let total_possible = now.signed_duration_since(start_time).num_seconds().max(0) as f64;
    if total_possible <= 0.0 {
        return 100.0;
    }

    let uptime = (total_online_secs / total_possible) * 100.0;
    uptime.clamp(0.0, 100.0)
}

// Struct to keep previous ticks for calculation across updates
#[derive(Default)]
pub struct MetricsState {
    prev_ticks: Option<CpuTicks>,
    last_uptime: Option<f64>,
    last_uptime_calc: Option<std::time::Instant>,
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
    let (avg_psi, total_psi) = get_cpu_pressure().unwrap_or((0.0, 0));

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

    let uptime_percent =
        if let (Some(last_uptime), Some(last_calc)) = (state.last_uptime, state.last_uptime_calc) {
            if last_calc.elapsed() < std::time::Duration::from_secs(60) {
                last_uptime
            } else {
                let val = calculate_uptime(pool).await;
                state.last_uptime = Some(val);
                state.last_uptime_calc = Some(std::time::Instant::now());
                val
            }
        } else {
            let val = calculate_uptime(pool).await;
            state.last_uptime = Some(val);
            state.last_uptime_calc = Some(std::time::Instant::now());
            val
        };

    let metrics = MetricsResponse {
        uptime_percent,
        load_avg,
        memory_percent,
        cpu_percent,
        cpu_usage,
        io_percent,
        disk_percent,
        avg_psi,
        total_psi,
    };

    metrics_cache.set_payload(metrics);
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::arithmetic_side_effects,
    clippy::float_cmp
)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[sqlx::test]
    async fn test_calculate_uptime_scenarios(pool: Pool<Sqlite>) {
        let now = Utc::now();

        // Scenario A: No events at all -> uptime is 100.0%
        let uptime = calculate_uptime(&pool).await;
        assert_eq!(uptime, 100.0);

        // Scenario B: Monitored for 2 hours, all online (single online_until updated to now)
        sqlx::query!(
            "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES ('system', 0, ?)",
            now
        )
        .execute(&pool)
        .await
        .unwrap();

        let uptime = calculate_uptime(&pool).await;
        assert_eq!(uptime, 100.0);

        // Scenario C: Measurable uptime with multiple events:
        sqlx::query!("DELETE FROM system_uptime_events")
            .execute(&pool)
            .await
            .unwrap();

        // 1. Started at t - 60 mins: online_until t - 40 mins
        let t_start = now - Duration::minutes(60);
        let t_first_online = now - Duration::minutes(40);
        sqlx::query!(
            "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES ('system', 0, ?)",
            t_start
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES ('system', 0, ?)",
            t_first_online
        )
        .execute(&pool)
        .await
        .unwrap();

        // 2. Offine until t - 20 mins
        let t_offline = now - Duration::minutes(20);
        sqlx::query!(
            "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES ('system', 1, ?)",
            t_offline
        )
        .execute(&pool)
        .await
        .unwrap();

        // 3. Online until now - 5 mins
        let t_last_online = now - Duration::minutes(5);
        sqlx::query!(
            "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES ('system', 0, ?)",
            t_last_online
        )
        .execute(&pool)
        .await
        .unwrap();

        // Expected Uptime = 40 / 60 * 100 = 66.6666...%
        let uptime = calculate_uptime(&pool).await;
        assert!(uptime > 66.6 && uptime < 66.7);
    }
}
