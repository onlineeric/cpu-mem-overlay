//! System-metric sampling.
//!
//! Owns the `sysinfo::System` handle and produces `MetricSnapshot`s on demand.
//! Pure formatting helpers live here too so they can be unit-tested without
//! standing up a `sysinfo::System`.

use sysinfo::System;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct MetricSnapshot {
    pub(crate) cpu: Option<u8>,
    pub(crate) memory: Option<u8>,
}

pub(crate) fn format_line(label: &str, value: Option<u8>) -> String {
    match value {
        Some(n) => format!("{label} {pct}%", pct = n.min(100)),
        None => format!("{label} --"),
    }
}

pub(crate) struct Sampler {
    sys: System,
}

impl Sampler {
    pub(crate) fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        Self { sys }
    }

    pub(crate) fn sample(&mut self) -> MetricSnapshot {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();
        MetricSnapshot {
            cpu: read_cpu_pct(&self.sys),
            memory: read_memory_pct(&self.sys),
        }
    }
}

fn read_cpu_pct(sys: &System) -> Option<u8> {
    let raw = sys.global_cpu_usage();
    if !raw.is_finite() {
        return None;
    }
    Some(raw.round().clamp(0.0, 100.0) as u8)
}

fn read_memory_pct(sys: &System) -> Option<u8> {
    let total = sys.total_memory();
    if total == 0 {
        return None;
    }
    let used = sys.used_memory();
    let pct = (used as f64 * 100.0) / total as f64;
    if !pct.is_finite() {
        return None;
    }
    Some(pct.round().clamp(0.0, 100.0) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_some_zero() {
        assert_eq!(format_line("CPU", Some(0)), "CPU 0%");
    }

    #[test]
    fn cpu_some_seven() {
        assert_eq!(format_line("CPU", Some(7)), "CPU 7%");
    }

    #[test]
    fn cpu_some_hundred() {
        assert_eq!(format_line("CPU", Some(100)), "CPU 100%");
    }

    #[test]
    fn cpu_none() {
        assert_eq!(format_line("CPU", None), "CPU --");
    }

    #[test]
    fn mem_some_zero() {
        assert_eq!(format_line("MEM", Some(0)), "MEM 0%");
    }

    #[test]
    fn mem_some_seven() {
        assert_eq!(format_line("MEM", Some(7)), "MEM 7%");
    }

    #[test]
    fn mem_some_hundred() {
        assert_eq!(format_line("MEM", Some(100)), "MEM 100%");
    }

    #[test]
    fn mem_none() {
        assert_eq!(format_line("MEM", None), "MEM --");
    }

    #[test]
    fn out_of_range_clamps_to_hundred() {
        assert_eq!(format_line("CPU", Some(123)), "CPU 100%");
    }
}
