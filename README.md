# ghost-tunnel

![Rust](https://img.shields.io/badge/rust-1.75%2B-black?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey)

**A fault-tolerant, userspace UDP tunneling protocol optimized for high-latency and lossy network environments.**

## Motivation

Traditional VPN protocols (such as OpenVPN over TCP) often suffer from **head-of-line blocking** when packet loss exceeds 5-10%. This results in connection stalls and frequent distinct handshakes, making them unusable on unstable mobile networks (e.g., rural 4G/3G).

**GhostTunnel** implements a custom reliable-UDP transport layer designed to prioritize connectivity over strict ordering. It decouples congestion control from the tunnel logic, allowing traffic to persist through packet loss rates as high as 30%.

## Core Capabilities

* **Resilient Transport:** UDP-based encapsulation prevents TCP meltdown during network degradation.
* **Adaptive Compression:** Implements **Zstd** with content-type heuristics. Detects and skips compression for high-entropy payloads (images, encrypted archives) to minimize CPU cycles.
* **Traffic Obfuscation:** Mitigates Deep Packet Inspection (DPI) by implementing randomized packet sizing and timing jitter (0-15ms) to disrupt traffic analysis signatures.
* **Headless Monitoring:** Integrated TUI (Terminal User Interface) via `ratatui` for real-time throughput analysis on servers without window managers.

## Architecture

Data flows through a user-space TUN interface, is processed by the optimization pipeline, and transmitted via raw UDP sockets.

```mermaid
graph TD
    subgraph "Client Host"
        App[Application Traffic] --> |IP Packet| TUN[TUN Interface]
        TUN --> |Raw Bytes| GT[GhostTunnel Process]
        GT --> |Optimization| Zstd[Zstd Compression]
        Zstd --> |Encryption| ChaCha[ChaCha20-Poly1305]
        ChaCha --> |Obfuscation| UDP[UDP Socket]
    end

    UDP --> |Encrypted Frame| Internet((Public Network))