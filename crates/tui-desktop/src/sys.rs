//! Acceso a Linux (/proc, sistema de ficheros) para las apps del escritorio.
//! Los parsers son funciones puras sobre `&str` para poder testearlos sin /proc.
//! Más adelante esto se promoverá a una API propia de TUI-OS (arquitectura §17).

use std::fs;
use std::io;
use std::path::Path;

/// Contadores acumulados de CPU (jiffies) de una línea `cpu`/`cpuN` de /proc/stat.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CpuTimes {
    pub total: u64,
    pub idle: u64,
}

impl CpuTimes {
    /// Uso (0..=1) entre dos muestras.
    pub fn usage_since(&self, prev: &CpuTimes) -> f32 {
        let dt = self.total.saturating_sub(prev.total);
        let di = self.idle.saturating_sub(prev.idle);
        if dt == 0 {
            0.0
        } else {
            (dt - di.min(dt)) as f32 / dt as f32
        }
    }
}

/// Primera entrada: agregado; resto: núcleos, en orden.
pub fn parse_stat(s: &str) -> Vec<CpuTimes> {
    s.lines()
        .filter(|l| l.starts_with("cpu"))
        .filter_map(|l| {
            let f: Vec<u64> = l
                .split_whitespace()
                .skip(1)
                .filter_map(|x| x.parse().ok())
                .collect();
            if f.len() < 5 {
                return None;
            }
            // user nice system idle iowait irq softirq steal
            Some(CpuTimes {
                total: f.iter().take(8).sum(),
                idle: f[3] + f[4],
            })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MemInfo {
    pub total_kb: u64,
    pub available_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
}

impl MemInfo {
    pub fn used_kb(&self) -> u64 {
        self.total_kb.saturating_sub(self.available_kb)
    }

    pub fn used_frac(&self) -> f32 {
        if self.total_kb == 0 {
            0.0
        } else {
            self.used_kb() as f32 / self.total_kb as f32
        }
    }

    pub fn swap_used_kb(&self) -> u64 {
        self.swap_total_kb.saturating_sub(self.swap_free_kb)
    }
}

pub fn parse_meminfo(s: &str) -> MemInfo {
    let mut m = MemInfo::default();
    for l in s.lines() {
        let mut it = l.split_whitespace();
        let (Some(k), Some(v)) = (it.next(), it.next()) else {
            continue;
        };
        let v: u64 = v.parse().unwrap_or(0);
        match k {
            "MemTotal:" => m.total_kb = v,
            "MemAvailable:" => m.available_kb = v,
            "SwapTotal:" => m.swap_total_kb = v,
            "SwapFree:" => m.swap_free_kb = v,
            _ => {}
        }
    }
    m
}

pub fn parse_uptime(s: &str) -> u64 {
    s.split_whitespace()
        .next()
        .and_then(|x| x.parse::<f64>().ok())
        .map_or(0, |x| x as u64)
}

pub fn parse_loadavg(s: &str) -> [f32; 3] {
    let mut out = [0.0; 3];
    for (o, x) in out.iter_mut().zip(s.split_whitespace()) {
        *o = x.parse().unwrap_or(0.0);
    }
    out
}

/// PRETTY_NAME de os-release, sin comillas.
pub fn parse_pretty_name(s: &str) -> Option<String> {
    s.lines()
        .find_map(|l| l.strip_prefix("PRETTY_NAME="))
        .map(|v| v.trim_matches('"').to_string())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proc {
    pub pid: u32,
    pub name: String,
    pub state: char,
    pub rss_kb: u64,
}

/// `/proc/<pid>/stat`: el nombre va entre paréntesis y puede contener espacios y paréntesis.
pub fn parse_proc_stat(pid: u32, s: &str, page_kb: u64) -> Option<Proc> {
    let open = s.find('(')?;
    let close = s.rfind(')')?;
    let name = s.get(open + 1..close)?.to_string();
    let rest: Vec<&str> = s.get(close + 1..)?.split_whitespace().collect();
    // rest[0] = estado (campo 3); rss = campo 24 -> índice 21
    let state = rest.first()?.chars().next()?;
    let rss_pages: u64 = rest.get(21)?.parse().ok()?;
    Some(Proc {
        pid,
        name,
        state,
        rss_kb: rss_pages.saturating_mul(page_kb),
    })
}

pub struct Snapshot {
    pub cpu: Vec<CpuTimes>,
    pub mem: MemInfo,
    pub uptime_s: u64,
    pub load: [f32; 3],
}

fn read(p: &str) -> String {
    fs::read_to_string(p).unwrap_or_default()
}

pub fn snapshot() -> Snapshot {
    Snapshot {
        cpu: parse_stat(&read("/proc/stat")),
        mem: parse_meminfo(&read("/proc/meminfo")),
        uptime_s: parse_uptime(&read("/proc/uptime")),
        load: parse_loadavg(&read("/proc/loadavg")),
    }
}

pub fn hostname() -> String {
    read("/proc/sys/kernel/hostname").trim().to_string()
}

pub fn kernel_release() -> String {
    read("/proc/sys/kernel/osrelease").trim().to_string()
}

pub fn os_name() -> String {
    parse_pretty_name(&read("/etc/os-release")).unwrap_or_else(|| "Linux".into())
}

/// Uid real desde `/proc/self/status` (línea `Uid:\treal\teffective...`).
pub fn parse_uid(status: &str) -> Option<u32> {
    status
        .lines()
        .find_map(|l| l.strip_prefix("Uid:"))?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

/// Nombre de usuario de `uid` según el contenido de `/etc/passwd`.
pub fn parse_passwd_name(passwd: &str, uid: u32) -> Option<String> {
    passwd.lines().find_map(|l| {
        let mut f = l.split(':');
        let name = f.next()?;
        (f.nth(1)? == uid.to_string()).then(|| name.to_string())
    })
}

/// «nombre (uid N)» del usuario que ejecuta el escritorio.
pub fn current_user() -> String {
    match parse_uid(&read("/proc/self/status")) {
        Some(uid) => match parse_passwd_name(&read("/etc/passwd"), uid) {
            Some(n) => format!("{n} (uid {uid})"),
            None => format!("uid {uid}"),
        },
        None => "?".into(),
    }
}

pub fn cpu_model() -> String {
    read("/proc/cpuinfo")
        .lines()
        .find_map(|l| l.strip_prefix("model name"))
        .map(|v| v.trim_start_matches([' ', '\t', ':']).trim().to_string())
        .unwrap_or_else(|| "CPU".into())
}

/// Procesos con su memoria residente, de mayor a menor.
pub fn processes() -> Vec<Proc> {
    let mut v: Vec<Proc> = fs::read_dir("/proc")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let pid: u32 = e.file_name().to_str()?.parse().ok()?;
            parse_proc_stat(pid, &fs::read_to_string(e.path().join("stat")).ok()?, 4)
        })
        .collect();
    v.sort_by(|a, b| b.rss_kb.cmp(&a.rss_kb).then(a.pid.cmp(&b.pid)));
    v
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Contenido de un directorio: carpetas primero, luego por nombre sin distinguir mayúsculas.
pub fn read_dir(path: &Path) -> io::Result<Vec<Entry>> {
    let mut v: Vec<Entry> = fs::read_dir(path)?
        .flatten()
        .map(|e| {
            let md = e.metadata().ok();
            Entry {
                name: e.file_name().to_string_lossy().into_owned(),
                is_dir: md.as_ref().is_some_and(|m| m.is_dir()),
                size: md.map_or(0, |m| m.len()),
            }
        })
        .collect();
    v.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(v)
}

/// Tamaño legible: 1.5K, 12M…
pub fn human(bytes: u64) -> String {
    const U: [&str; 5] = ["B", "K", "M", "G", "T"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes}B")
    } else {
        format!("{v:.1}{}", U[i])
    }
}

pub fn human_kb(kb: u64) -> String {
    human(kb.saturating_mul(1024))
}

pub fn fmt_uptime(s: u64) -> String {
    let (d, h, m) = (s / 86400, s % 86400 / 3600, s % 3600 / 60);
    if d > 0 {
        format!("{d}d {h:02}h {m:02}m")
    } else {
        format!("{h:02}h {m:02}m {:02}s", s % 60)
    }
}

/// Hora UTC `hh:mm:ss` (la imagen va en UTC; sin dependencias de zona horaria).
pub fn clock_utc() -> String {
    let s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    format!("{:02}:{:02}:{:02}", s / 3600 % 24, s / 60 % 60, s % 60)
}

#[cfg(test)]
mod tests {
    #[test]
    fn uid_and_passwd_name() {
        assert_eq!(
            parse_uid("Name:\tx\nUid:\t1000\t1000\t1000\t1000\n"),
            Some(1000)
        );
        assert_eq!(parse_uid("Name:\tx\n"), None);
        let pw = "root:x:0:0::/root:/bin/bash\ntui:x:1000:1000:TUI:/home/tui:/usr/bin/nologin\n";
        assert_eq!(parse_passwd_name(pw, 1000).as_deref(), Some("tui"));
        assert_eq!(parse_passwd_name(pw, 0).as_deref(), Some("root"));
        assert_eq!(parse_passwd_name(pw, 5), None);
    }

    use super::*;

    #[test]
    fn stat_aggregate_and_cores() {
        let s = "cpu  100 0 100 700 100 0 0 0 0 0\ncpu0 50 0 50 350 50 0 0 0 0 0\nintr 1\n";
        let v = parse_stat(s);
        assert_eq!(v.len(), 2);
        assert_eq!(
            v[0],
            CpuTimes {
                total: 1000,
                idle: 800
            }
        );
    }

    #[test]
    fn cpu_usage_between_samples() {
        let a = CpuTimes {
            total: 1000,
            idle: 800,
        };
        let b = CpuTimes {
            total: 1200,
            idle: 900,
        };
        assert!((b.usage_since(&a) - 0.5).abs() < 1e-6);
        assert_eq!(a.usage_since(&a), 0.0, "sin intervalo no divide por cero");
    }

    #[test]
    fn meminfo_used_and_swap() {
        let m = parse_meminfo(
            "MemTotal: 1000 kB\nMemAvailable: 250 kB\nSwapTotal: 100 kB\nSwapFree: 40 kB\n",
        );
        assert_eq!((m.used_kb(), m.swap_used_kb()), (750, 60));
        assert!((m.used_frac() - 0.75).abs() < 1e-6);
        assert_eq!(MemInfo::default().used_frac(), 0.0);
    }

    #[test]
    fn uptime_loadavg_and_os_name() {
        assert_eq!(parse_uptime("12345.67 999.0\n"), 12345);
        assert_eq!(parse_loadavg("0.50 1.25 2.00 1/200 99\n"), [0.5, 1.25, 2.0]);
        assert_eq!(
            parse_pretty_name("NAME=\"X\"\nPRETTY_NAME=\"TUI-OS (Arch base)\"\n").as_deref(),
            Some("TUI-OS (Arch base)")
        );
        assert_eq!(parse_pretty_name("NAME=x"), None);
    }

    #[test]
    fn proc_stat_with_tricky_name() {
        // comm con espacios y paréntesis; rss = campo 24
        let s = "42 (my (weird) app) S 1 42 42 0 -1 4194560 100 0 0 0 1 2 0 0 20 0 1 0 100 1000000 321 18446744073709551615 0 0 0 0 0 0 0 0 0 0 0 0 17 0 0 0 0 0 0";
        let p = parse_proc_stat(42, s, 4).unwrap();
        assert_eq!(
            (p.name.as_str(), p.state, p.rss_kb),
            ("my (weird) app", 'S', 321 * 4)
        );
        assert!(parse_proc_stat(1, "garbage", 4).is_none());
    }

    #[test]
    fn processes_includes_self() {
        let me = std::process::id();
        assert!(processes().iter().any(|p| p.pid == me));
    }

    #[test]
    fn read_dir_puts_dirs_first() {
        let d = std::env::temp_dir().join(format!("tuidesk-{}", std::process::id()));
        fs::create_dir_all(d.join("zdir")).unwrap();
        fs::write(d.join("a.txt"), b"hi").unwrap();
        let v = read_dir(&d).unwrap();
        fs::remove_dir_all(&d).unwrap();
        assert_eq!(
            v.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
            ["zdir", "a.txt"]
        );
        assert_eq!(v[1].size, 2);
    }

    #[test]
    fn formatting() {
        assert_eq!(human(512), "512B");
        assert_eq!(human(1536), "1.5K");
        assert_eq!(human_kb(2048), "2.0M");
        assert_eq!(fmt_uptime(90061), "1d 01h 01m");
        assert_eq!(fmt_uptime(3725), "01h 02m 05s");
    }
}
