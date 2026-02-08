[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)

 # Async Scanner

A fast, asynchronous TCP port scanner written in Rust using Tokio.

Async Scanner is designed to perform high-speed TCP port scanning with configurable concurrency, timeout control, and optional banner grabbing. The tool is lightweight, simple to use, and intended for network diagnostics and security research in authorized environments.

---

# Features

- ⚡ Asynchronous TCP scanning powered by Tokio
- 🎛 Configurable concurrency level
- ⏱ Customizable connection timeout
- 📡 Optional service banner grabbing
- 📊 Output in human-readable or JSON format
- 📈 Progress indicator during scanning

---

# Installation

Clone the repository:

git clone https://github.com/NickIBrody/async-scanner.git
cd async-scanner

Build the project:

cargo build --release

The compiled binary will be located in:

target/release/async-scanner

---

# Usage

Basic example:

async-scanner <target> [options]

Example:

async-scanner 192.168.1.1 --ports 1-1000 --concurrency 500 --timeout 3

# Available Options

- "--ports" — Port range to scan (e.g. "1-65535")
- "--concurrency" — Number of simultaneous connection attempts
- "--timeout" — Connection timeout in seconds
- "--json" — Output results in JSON format
- "--banner" — Attempt to grab service banners

---

# Legal Disclaimer

This tool is intended strictly for authorized testing and educational purposes.

Do not use this software to scan networks, hosts, or systems without explicit permission from the owner. Unauthorized scanning may violate local laws, terms of service, or network policies.

The author assumes no responsibility for misuse or damage caused by this tool.

---

# Contributing

Contributions, improvements, and suggestions are welcome.
Feel free to open an issue or submit a pull request.

---

# License
 MIT
