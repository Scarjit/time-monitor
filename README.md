# 🕒 Time-Monitor

[![Rust](https://github.com/scarjit/time-monitor/actions/workflows/rust.yml/badge.svg)](https://github.com/scarjit/time-monitor/actions/workflows/rust.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A comprehensive, Docker-based monitoring solution for NTP and GPS status, using Grafana, Prometheus, and a custom Rust-based GPS exporter.

## ✨ Features

- **Real-time GPS Monitoring**: Track satellite fixes, signal strength, and precision metrics via `gpsd`.
- **NTP Performance Tracking**: Monitor Chrony offset, delay, and synchronization status.
- **Network Health**: Includes Smokeping-style latency tracking and Blackbox ICMP/HTTP probing.
- **Pre-configured Dashboards**: Comes with ready-to-use Grafana dashboards for immediate visualization.

## 🏗️ Architecture

- **Prometheus**: Time-series database for metrics storage.
- **Grafana**: Visualization platform with pre-provisioned data sources and dashboards.
- **gps_exporter (Rust)**: Custom exporter that interfaces with `gpsd`.
- **Chrony Exporter**: Collects metrics from the Chrony NTP daemon.
- **Smokeping Prober**: Monitors network latency and packet loss.
- **Blackbox Exporter**: Performs endpoint probing.

## 🚀 Getting Started

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)
- A running `gpsd` instance (if monitoring local GPS hardware)
- A running `chrony` instance (exposed on port `323` for the exporter)

### Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/scarjit/time-monitor.git
   cd time-monitor
   ```

2. **Start the stack**:
   ```bash
   docker-compose up -d
   ```

3. **Access the Dashboards**:
   - Open your browser at `http://localhost:3000`
   - Login with:
     - **Username**: `admin`
     - **Password**: `admin`

## 📊 Dashboards

The project automatically provisions the following dashboards in Grafana:
- **GPS Dashboard**: Satellite status, coordinates, and precision.
- **Chrony Dashboard**: NTP synchronization metrics.
- **Network Monitoring**: Latency and connectivity status.

## 🛠️ Configuration

Most settings can be adjusted in the `docker-compose.yaml` file.

### Grafana Admin Credentials

You can change the default Grafana admin credentials by modifying the following environment variables in `docker-compose.yaml`:

```yaml
grafana:
  environment:
    - GF_SECURITY_ADMIN_USER=admin
    - GF_SECURITY_ADMIN_PASSWORD=admin
```

### GPS Exporter Settings

The `gps_exporter` configuration can also be customized:

```yaml
gps_exporter:
  environment:
    - GPSD_HOST=127.0.0.1
    - GPSD_PORT=2947
    - EXPORTER_PORT=9015
```

### Chrony Exporter

By default, the `chrony_exporter` connects to `127.0.0.1:323`. Ensure your `chrony` daemon is configured to allow connections from the exporter (usually via the `bindcmdaddress` and `cmdallow` directives in `chrony.conf`).

## 📄 License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.