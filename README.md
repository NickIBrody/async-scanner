[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
# ⚡ Port Scanner

Fast asynchronous TCP port scanner written in Rust using Tokio.

## Features

- Async TCP scanning (Tokio)
- Configurable concurrency
- Timeout control
- Banner grabbing
- Basic service detection
- JSON export
- TXT report export
- Progress bar
- Colored output

## 🚀 Installation

Clone the repository:

```bash
git clone https://github.com/YOUR_USERNAME/port-scanner.git
cd port-scanner

Build:
cargo build --release
Binary will be available at:
target/release/port-scanner
### Quick Start / Usage Examples

After building:


# Scan top 100 most common ports on a single host
./port-scanner -t 192.168.1.1 --top-ports 100

# Scan full port range (1–65535) on a subnet with banner grabbing + save to JSON
./port-scanner 10.0.0.0/24 -p 1-65535 --banner --json results.json

# Very fast scan with high concurrency and shorter timeout (example.com resolves to IP)
./port-scanner example.com --concurrency 5000 --timeout 800ms
