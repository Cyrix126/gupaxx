use crate::helper::{Helper, HumanNumber};

#[derive(Debug, Clone)]
pub struct Sys {
    pub gupax_uptime: String,
    pub gupax_cpu_usage: String,
    pub gupax_memory_used_mb: String,
    pub system_cpu_model: String,
    pub system_memory: String,
    pub system_cpu_usage: String,
}

impl Sys {
    pub fn new() -> Self {
        Self {
            gupax_uptime: "0 seconds".to_string(),
            gupax_cpu_usage: "???%".to_string(),
            gupax_memory_used_mb: "??? megabytes".to_string(),
            system_cpu_usage: "???%".to_string(),
            system_memory: "???GB / ???GB".to_string(),
            system_cpu_model: "???".to_string(),
        }
    }
}
impl Default for Sys {
    fn default() -> Self {
        Self::new()
    }
}
impl Helper {
    #[inline(always)] // called once
    pub fn update_pub_sys_from_sysinfo(
        sysinfo: &sysinfo::System,
        pub_sys: &mut Sys,
        pid: &sysinfo::Pid,
        helper: &Helper,
        max_threads: u16,
    ) {
        let gupax_uptime = helper.uptime.display(true);
        let cpu = &sysinfo.cpus()[0];
        let gupax_cpu_usage;
        let gupax_memory_used_mb;

        if let Some(process) = sysinfo.process(*pid) {
            gupax_cpu_usage = format!("{:.2}%", process.cpu_usage() / (max_threads as f32));
            gupax_memory_used_mb = format!(
                "{} megabytes",
                HumanNumber::from_u64(process.memory() / 1_000_000)
            );
        } else {
            gupax_cpu_usage = "???".to_string();
            gupax_memory_used_mb = "??? megabytes".to_string();
        };
        let system_cpu_model = format!("{} ({}MHz)", cpu.brand(), cpu.frequency());
        let system_memory = {
            let used = (sysinfo.used_memory() as f64) / 1_000_000_000.0;
            let total = (sysinfo.total_memory() as f64) / 1_000_000_000.0;
            format!("{used:.3} GB / {total:.3} GB")
        };
        let system_cpu_usage = {
            let mut total: f32 = 0.0;
            for cpu in sysinfo.cpus() {
                total += cpu.cpu_usage();
            }
            format!("{:.2}%", total / (max_threads as f32))
        };
        *pub_sys = Sys {
            gupax_uptime,
            gupax_cpu_usage,
            gupax_memory_used_mb,
            system_cpu_usage,
            system_memory,
            system_cpu_model,
        };
    }
}
