use clap::{Parser, ValueEnum};
use colored::*;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::time::timeout;

#[derive(Parser, Debug)]
#[command(name = "port-scanner", version = "0.1.0", about = "Fast async TCP port scanner")]
struct Args {
#[arg(short, long)]
target: String,

#[arg(short, long, default_value = "1-1024")]  
ports: String,  

#[arg(short = 'c', long, default_value_t = 512)]  
concurrency: usize,  

#[arg(short = 't', long, default_value_t = 800)]  
timeout_ms: u64,  

#[arg(short, long, value_enum, default_value_t = Verbosity::Normal)]  
verbose: Verbosity,  

#[arg(long)]  
json: Option<PathBuf>,  

#[arg(long)]  
output: Option<PathBuf>,  

#[arg(short = 'q', long)]  
quiet: bool,

}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
enum Verbosity {
Quiet,
Normal,
Verbose,
Debug,
}

#[derive(Error, Debug)]
#[error("Scan error")]
enum ScanError {
#[error("Invalid target address")]
InvalidTarget,

#[error("Network error: {0}")]  
Io(#[from] std::io::Error),

}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PortResult {
port: u16,
status: PortStatus,
banner: Option<String>,
service: Option<String>,
duration_ms: u128,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum PortStatus {
Open,
Closed,
Filtered,
}

impl fmt::Display for PortStatus {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
match self {
PortStatus::Open => write!(f, "{}", "open".bright_green()),
PortStatus::Closed => write!(f, "{}", "closed".bright_red()),
PortStatus::Filtered => write!(f, "{}", "filtered".yellow()),
}
}
}

#[derive(Debug, Serialize, Deserialize)]
struct ScanSummary {
target: String,
scanned_ports: usize,
open_ports: usize,
closed_ports: usize,
filtered_ports: usize,
total_time_ms: u128,
results: Vec<PortResult>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
let args = Args::parse();

let log_level = match args.verbose {  
    Verbosity::Quiet => LevelFilter::Error,  
    Verbosity::Normal => LevelFilter::Info,  
    Verbosity::Verbose => LevelFilter::Debug,  
    Verbosity::Debug => LevelFilter::Trace,  
};  
env_logger::builder().filter_level(log_level).init();  

info!("Starting scan on {}", args.target.bold());  

let ip_addr: IpAddr = IpAddr::from_str(&args.target)  
    .map_err(|_| ScanError::InvalidTarget)?;  

let ports = parse_ports(&args.ports)?;  
info!("Scanning {} ports", ports.len());  

let start_time = Instant::now();  
let semaphore = Arc::new(Semaphore::new(args.concurrency));  

let pb = if !args.quiet {  
    let style = ProgressStyle::default_bar()  
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")  
        .unwrap()  
        .progress_chars("#>-");  
    Some(ProgressBar::new(ports.len() as u64).with_style(style))  
} else {  
    None  
};  

let mut tasks = vec![];  

for port in ports {  
    let permit = semaphore.clone().acquire_owned().await?;  
    let target_ip = ip_addr;  
    let conn_timeout = Duration::from_millis(args.timeout_ms);  

    let task = tokio::spawn(async move {  
        let _permit = permit;  
        let addr = SocketAddr::new(target_ip, port);  

        let connect_res = timeout(conn_timeout, TcpStream::connect(addr)).await;  

        let duration = Instant::now() - start_time;  

        match connect_res {  
            Ok(Ok(mut stream)) => {  
                let banner = grab_banner(&mut stream, Duration::from_millis(1200)).await.ok();  
                let service = detect_service(port, banner.as_deref());  
                PortResult {  
                    port,  
                    status: PortStatus::Open,  
                    banner,  
                    service,  
                    duration_ms: duration.as_millis(),  
                }  
            }  
            Ok(Err(e)) if e.kind() == std::io::ErrorKind::ConnectionRefused => PortResult {  
                port,  
                status: PortStatus::Closed,  
                banner: None,  
                service: None,  
                duration_ms: duration.as_millis(),  
            },  
            _ => PortResult {  
                port,  
                status: PortStatus::Filtered,  
                banner: None,  
                service: None,  
                duration_ms: duration.as_millis(),  
            },  
        }  
    });  
    tasks.push(task);  
}  

let mut results = vec![];  

let mut stream = stream::iter(tasks).buffer_unordered(args.concurrency * 2);  

while let Some(res) = stream.next().await {  
    match res {  
        Ok(r) => {  
            if !args.quiet || r.status == PortStatus::Open {  
                print_result(&r, args.verbose == Verbosity::Verbose);  
            }  
            results.push(r);  
        }  
        Err(e) => error!("Task failed: {}", e),  
    }  
    if let Some(pb) = &pb {  
        pb.inc(1);  
    }  
}  

if let Some(pb) = &pb {  
    pb.finish_with_message("Scan completed");  
}  

let total_time = start_time.elapsed().as_millis();  
let open_count = results.iter().filter(|r| r.status == PortStatus::Open).count();  
let closed_count = results.iter().filter(|r| r.status == PortStatus::Closed).count();  
let filtered_count = results.iter().filter(|r| r.status == PortStatus::Filtered).count();  

let summary = ScanSummary {  
    target: args.target.clone(),  
    scanned_ports: results.len(),  
    open_ports: open_count,  
    closed_ports: closed_count,  
    filtered_ports: filtered_count,  
    total_time_ms: total_time,  
    results,  
};  

info!(  
    "Done. Open: {}, Closed: {}, Filtered: {}, Time: {} ms",  
    open_count.to_string().bright_green(),  
    closed_count,  
    filtered_count.to_string().yellow(),  
    total_time  
);  

if let Some(ref path) = args.json {  
    let json = serde_json::to_string_pretty(&summary)?;  
    std::fs::write(path, json)?;  
    info!("Saved JSON: {}", path.display());  
}  

if let Some(ref path) = args.output {  
    let mut txt = format!(  
        "Scan of {} | Ports: {} | Time: {}ms\n\n",  
        args.target, summary.scanned_ports, total_time  
    );  
    for r in &summary.results {  
        txt.push_str(&format!(  
            "Port {:>5} | {} | Service: {:<12} | Banner: {}\n",  
            r.port,  
            r.status,  
            r.service.as_deref().unwrap_or("-"),  
            r.banner.as_deref().unwrap_or("-").replace('\n', " ")  
        ));  
    }  
    std::fs::write(path, txt)?;  
    info!("Saved TXT: {}", path.display());  
}  

Ok(())

}

async fn grab_banner(stream: &mut TcpStream, dur: Duration) -> Result<String, ScanError> {
let mut buffer = vec![0u8; 4096];
let read_res = timeout(dur, async {
stream.readable().await?;
stream.read(&mut buffer).await
})
.await;

match read_res {  
    Ok(Ok(n)) if n > 0 => Ok(String::from_utf8_lossy(&buffer[..n]).trim_end().to_string()),  
    _ => Err(ScanError::Io(std::io::Error::new(  
        std::io::ErrorKind::Other,  
        "Banner read failed",  
    ))),  
}

}

fn detect_service(port: u16, banner: Option<&str>) -> Option<String> {
let mut m: HashMap<u16, &str> = HashMap::new();
m.insert(22, "SSH");
m.insert(80, "HTTP");
m.insert(443, "HTTPS");
m.insert(21, "FTP");
m.insert(25, "SMTP");
m.insert(3306, "MySQL");
m.insert(5432, "PostgreSQL");
m.insert(3389, "RDP");
m.insert(5900, "VNC");

if let Some(b) = banner {  
    if b.contains("SSH-") {  
        return Some("SSH".to_string());  
    }  
    if b.contains("HTTP/") || b.contains("Server:") {  
        return Some("HTTP".to_string());  
    }  
    if b.starts_with("220 ") {  
        return Some("SMTP/FTP".to_string());  
    }  
}  
m.get(&port).map(|s| s.to_string())

}

fn print_result(r: &PortResult, verbose: bool) {
let p = format!("{:>5}", r.port).bright_blue();
let s = match r.status {
PortStatus::Open => "open".bright_green(),
PortStatus::Closed => "closed".bright_red(),
PortStatus::Filtered => "filtered".yellow(),
};
let serv = r.service.as_deref().unwrap_or("-").bright_cyan();
let ban = r.banner.as_deref().map_or("-".to_string(), |b| {
let preview: String = b.chars().take(60).collect();
if b.len() > 60 {
format!("{}...", preview)
} else {
preview
}
});

if verbose {  
    println!("{} | {} | Service: {} | Banner: {}", p, s, serv, ban);  
} else if r.status == PortStatus::Open {  
    println!("{} open   {}", p, serv);  
}

}

fn parse_ports(s: &str) -> Result<Vec<u16>, Box<dyn std::error::Error>> {
let mut v = Vec::new();
for part in s.split(',') {
let part = part.trim();
if part.is_empty() {
continue;
}
if part.contains('-') {
let nums: Vec<&str> = part.split('-').collect();
if nums.len() != 2 {
return Err("Invalid range format".into());
}
let start: u16 = nums[0].parse()?;
let end: u16 = nums[1].parse()?;
if start > end {
return Err("Start > end".into());
}
for i in start..=end {
v.push(i);
}
} else {
v.push(part.parse()?);
}
}
v.sort();
v.dedup();
v.retain(|&p| p != 0);
if v.is_empty() {
return Err("No valid ports".into());
}
Ok(v)
}
