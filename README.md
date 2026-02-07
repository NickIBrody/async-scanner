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
