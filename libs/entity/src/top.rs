use anyhow::Context;
use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Top {
    pub processes: Processes,
    pub load_avg: LoadAvg,
    pub cpu_usage: CpuUsage,
    pub shared_libs: SharedLibs,
    pub mem_regions: MemRegions,
    pub phys_mem: PhysMem,
    pub vm: Vm,
    pub networks: Networks,
    pub disks: Disks,
}

impl Top {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let mut top = Top::default();
        let lines: Vec<&str> = str.lines().collect();
        for line in lines {
            if line.starts_with("Processes") {
                top.processes = Processes::new_from_str(line)?;
            } else if line.starts_with("Load") {
                top.load_avg = LoadAvg::new_from_str(line)?;
            } else if line.starts_with("CPU") {
                top.cpu_usage = CpuUsage::new_from_str(line)?;
            } else if line.starts_with("SharedLibs") {
                top.shared_libs = SharedLibs::new_from_str(line)?;
            } else if line.starts_with("MemRegions") {
                top.mem_regions = MemRegions::new_from_str(line)?;
            } else if line.starts_with("PhysMem") {
                top.phys_mem = PhysMem::new_from_str(line)?;
            } else if line.starts_with("VM") {
                top.vm = Vm::new_from_str(line)?;
            } else if line.starts_with("Networks") {
                top.networks = Networks::new_from_str(line)?;
            } else if line.starts_with("Disks") {
                top.disks = Disks::new_from_str(line)?;
            }
        }
        Ok(top)
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Processes {
    total: u32,
    running: u32,
    stuck: Option<u32>,
    sleeping: u32,
    threads: u32,
    datetime: DateTime<Utc>,
}

// Processes: 450 total, 6 running, 1 stuck, 443 sleeping, 3166 threads 2024/04/11 01:58:07
// or
// Processes: 450 total, 6 running, 443 sleeping, 3166 threads 2024/04/11 01:58:07
impl Processes {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"Processes: (\d+) total, (\d+) running, (\d+) stuck, (\d+) sleeping, (\d+) threads (\d+/\d+/\d+ \d+:\d+:\d+)",
        )?;
        let caps = re.captures(str);
        if let Some(caps) = caps {
            let total =
                caps.get(1).context("total")?.as_str().parse::<u32>()?;
            let running =
                caps.get(2).context("running")?.as_str().parse::<u32>()?;
            let stuck =
                caps.get(3).context("stuck")?.as_str().parse::<u32>()?;
            let sleeping =
                caps.get(4).context("sleeping")?.as_str().parse::<u32>()?;
            let threads =
                caps.get(5).context("threads")?.as_str().parse::<u32>()?;
            let datetime = NaiveDateTime::parse_from_str(
                caps.get(6).context("datetime")?.as_str(),
                "%Y/%m/%d %H:%M:%S",
            )? - TimeDelta::try_hours(9)
                .context("try_hours 9")?;
            Ok(Processes {
                total,
                running,
                stuck: Some(stuck),
                sleeping,
                threads,
                datetime: datetime.and_utc(),
            })
        } else {
            let re = Regex::new(
                r"Processes: (\d+) total, (\d+) running, (\d+) sleeping, (\d+) threads (\d+/\d+/\d+ \d+:\d+:\d+)",
            )?;
            let caps = re.captures(str).context("processes")?;
            let total =
                caps.get(1).context("total")?.as_str().parse::<u32>()?;
            let running =
                caps.get(2).context("running")?.as_str().parse::<u32>()?;
            let sleeping =
                caps.get(3).context("sleeping")?.as_str().parse::<u32>()?;
            let threads =
                caps.get(4).context("threads")?.as_str().parse::<u32>()?;
            let datetime = NaiveDateTime::parse_from_str(
                caps.get(5).context("datetime")?.as_str(),
                "%Y/%m/%d %H:%M:%S",
            )? - TimeDelta::try_hours(9)
                .context("try_hours 9")?;
            Ok(Processes {
                total,
                running,
                stuck: None,
                sleeping,
                threads,
                datetime: datetime.and_utc(),
            })
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct LoadAvg {
    one: f32,
    five: f32,
    fifteen: f32,
}

// Load Avg: 3.69, 3.91, 3.86
impl LoadAvg {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(r"Load Avg: (\d+\.\d+), (\d+\.\d+), (\d+\.\d+) ")?;
        let caps = re.captures(str).context("load avg")?;
        let one = caps.get(1).context("one")?.as_str().parse::<f32>()?;
        let five = caps.get(2).context("five")?.as_str().parse::<f32>()?;
        let fifteen =
            caps.get(3).context("fifteen")?.as_str().parse::<f32>()?;
        Ok(LoadAvg { one, five, fifteen })
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct CpuUsage {
    user: f32,
    system: f32,
    idle: f32,
}

// CPU usage: 28.80% user, 14.81% sys, 56.37% idle
impl CpuUsage {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"CPU usage: (\d+\.\d+)% user, (\d+\.\d+)% sys, (\d+\.\d+)% idle",
        )?;
        let caps = re.captures(str).context("cpu usage")?;
        let user = caps.get(1).context("user")?.as_str().parse::<f32>()?;
        let system = caps.get(2).context("system")?.as_str().parse::<f32>()?;
        let idle = caps.get(3).context("idle")?.as_str().parse::<f32>()?;
        Ok(CpuUsage { user, system, idle })
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct SharedLibs {
    resident: u32,
    data: u32,
    linkedit: u32,
}

// SharedLibs: 374M resident, 84M data, 27M linkedit.
impl SharedLibs {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"SharedLibs: (\d+[M|G|T]) resident, (\d+[M|G|T]) data, (\d+[M|G|T]) linkedit",
        )?;
        let caps = re.captures(str).context("shared libs")?;
        let resident = replace_unit(caps.get(1).context("resident")?.as_str())
            .parse::<u32>()?;
        let data = replace_unit(caps.get(2).context("data")?.as_str())
            .parse::<u32>()?;
        let linkedit = replace_unit(caps.get(3).context("linkedit")?.as_str())
            .parse::<u32>()?;
        Ok(SharedLibs {
            resident,
            data,
            linkedit,
        })
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct MemRegions {
    total: u32,
    resident: u32,
    private: u32,
    shared: u32,
}

// MemRegions: 380552 total, 1947M resident, 80M private, 2638M shared.
impl MemRegions {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"MemRegions: (\d+) total, (\d+[M|G|T]) resident, (\d+[M|G|T]) private, (\d+[M|G|T]) shared",
        )?;
        let caps = re.captures(str).context("mem regions")?;

        let total = caps.get(1).context("total")?.as_str().parse::<u32>()?;
        let resident = replace_unit(caps.get(2).context("resident")?.as_str())
            .parse::<u32>()?;
        let private = replace_unit(caps.get(3).context("private")?.as_str())
            .parse::<u32>()?;
        let shared = replace_unit(caps.get(4).context("shared")?.as_str())
            .parse::<u32>()?;
        Ok(MemRegions {
            total,
            resident,
            private,
            shared,
        })
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct PhysMem {
    used: u32,
    wired: u32,
    compressor: u32,
    unused: u32,
}

// PhysMem: 15G used (1930M wired, 7883M compressor), 80M unused.
impl PhysMem {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"PhysMem: (\d+[M|G|T]) used \((\d+[M|G|T]) wired, (\d+[M|G|T]) compressor\), (\d+[M|G|T]) unused",
        )?;
        let caps = re.captures(str).context("phys mem")?;

        let used = replace_unit(caps.get(1).context("used")?.as_str())
            .parse::<u32>()?;
        let wired = replace_unit(caps.get(2).context("wired")?.as_str())
            .parse::<u32>()?;
        let compressor =
            replace_unit(caps.get(3).context("compressor")?.as_str())
                .parse::<u32>()?;
        let unused = replace_unit(caps.get(4).context("unused")?.as_str())
            .parse::<u32>()?;
        Ok(PhysMem {
            used,
            wired,
            compressor,
            unused,
        })
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Vm {
    vsize: u64,
    framework_vsize: u64,
    swapins: u32,
    swapouts: u32,
}

// VM: 211T vsize, 4773M framework vsize, 1964944(12) swapins, 2631760(0) swapouts.
impl Vm {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"VM: (\d+[M|G|T]) vsize, (\d+[M|G|T]) framework vsize, (\d+)\(\d+\) swapins, (\d+)\(\d+\) swapouts",
        )?;
        let caps = re.captures(str).context("vm")?;
        let vsize = replace_unit(caps.get(1).context("vsize")?.as_str())
            .parse::<u64>()?;
        let framework_vsize =
            replace_unit(caps.get(2).context("framework vsize")?.as_str())
                .parse::<u64>()?;
        let swapins =
            caps.get(3).context("swapins")?.as_str().parse::<u32>()?;
        let swapouts =
            caps.get(4).context("swapouts")?.as_str().parse::<u32>()?;
        Ok(Vm {
            vsize,
            framework_vsize,
            swapins,
            swapouts,
        })
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Networks {
    packets_in: u32,
    packets_out: u32,
    in_total: u32,
    out_total: u32,
}

// Networks: packets: 106614114/58G in, 98394391/46G out.
impl Networks {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"Networks: packets: (\d+)\/(\d+[M|G|T]) in, (\d+)\/(\d+[M|G|T]) out",
        )?;
        let caps = re.captures(str).context("networks")?;
        let packets_in =
            caps.get(1).context("packets in")?.as_str().parse::<u32>()?;
        let packets_out = caps
            .get(3)
            .context("packets out")?
            .as_str()
            .parse::<u32>()?;
        let in_total = replace_unit(caps.get(2).context("in total")?.as_str())
            .parse::<u32>()?;
        let out_total =
            replace_unit(caps.get(4).context("out total")?.as_str())
                .parse::<u32>()?;
        Ok(Networks {
            packets_in,
            packets_out,
            in_total,
            out_total,
        })
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Disks {
    read: u32,
    written: u32,
    read_total: u32,
    written_total: u32,
}

// Disks: 48697305/927G read, 34308023/523G written.
impl Disks {
    pub fn new_from_str(str: &str) -> anyhow::Result<Self> {
        let re = Regex::new(
            r"Disks: (\d+)\/(\d+[M|G|T]) read, (\d+)\/(\d+[M|G|T]) written",
        )?;
        let caps = re.captures(str).context("disks")?;
        let read = caps.get(1).context("read")?.as_str().parse::<u32>()?;
        let written =
            caps.get(3).context("written")?.as_str().parse::<u32>()?;
        let read_total =
            replace_unit(caps.get(2).context("read total")?.as_str())
                .parse::<u32>()?;
        let written_total =
            replace_unit(caps.get(4).context("written total")?.as_str())
                .parse::<u32>()?;
        Ok(Disks {
            read,
            written,
            read_total,
            written_total,
        })
    }
}

fn replace_unit(s: &str) -> String {
    s.replace('K', "000")
        .replace('M', "000000")
        .replace('G', "000000000")
        .replace('T', "000000000000")
}
