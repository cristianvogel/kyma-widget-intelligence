# Widget Intelligence

A Rust library for intelligent widget suggestion and learning based on user behavior patterns.

## Features

- **Smart Suggestions**: Learn from user behavior to suggest widget values from training on captured Kyma presets
- **Similarity Engine**: Advanced algorithm for finding similar widgets based on multiple features
- **Persistent Storage**: Sled-based database for long-term learning
- **Kyma Integration**: Extract and process widget data from Kyma JSON format
- **Pure Rust**: Pure Rust library without UI framework dependencies
- **Tauri Ready**: Example integrations for Tauri applications

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
widget_intelligence = "0.1"
```