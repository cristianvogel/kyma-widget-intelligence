# Widget Intelligence

A Rust library for intelligent widget suggestion and learning based on user behavior patterns.

## Features

- **Smart Suggestions**: Learn from user behavior to suggest widget values
- **Similarity Engine**: Advanced algorithm for finding similar widgets based on multiple features
- **Persistent Storage**: Sled-based database for long-term learning
- **Kyma Integration**: Extract and process widget data from Kyma JSON format
- **Zero Dependencies**: Pure Rust library without UI framework dependencies
- **Tauri Ready**: Example integrations for Tauri applications

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
widget_intelligence = "0.1"
```

## Quick Start

### Basic Usage

```rust
use widget_intelligence::*;

// Create a new suggestion engine
let mut engine = WidgetSuggestionEngine::new();

// Define a widget
let widget = Widget {
    label: Some("Master Volume".to_string()),
    minimum: Some(0.0),
    maximum: Some(127.0),
    current_value: Some(95.0),
    is_generated: Some(false),
    display_type: Some("slider".to_string()),
};

// Store the widget to learn from it
engine.store_widget(widget);

// Get suggestions for similar widgets
let partial_widget = Widget {
    label: Some("Volume".to_string()),
    ..Default::default()
};

let suggestions = engine.get_suggestions(&partial_widget, 5);
for suggestion in suggestions {
    println!("Suggested value: {:?} (confidence: {:.2})", 
             suggestion.suggested_value, suggestion.confidence);
}
```

### Persistent Storage

```rust
use widget_intelligence::*;

// Initialize with persistent storage
let mut system = init_intelligence_system("./widget_data.db")?;

// Store widgets - they'll be persisted automatically
system.store_widget(create_test_widget("Bass Level", 0.0, 100.0, 75.0))?;

// Create and store presets
let preset = Preset {
    name: "My Audio Setup".to_string(),
    description: Some("Perfect for podcasts".to_string()),
    widget_values: vec![
        WidgetValue {
            widget_id: "1".to_string(),
            label: Some("Bass Level".to_string()),
            value: 75.0,
            confidence: 1.0,
        }
    ],
    created_by: Some("user123".to_string()),
    usage_count: 1,
    last_used: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs(),
};

system.store_preset(preset)?;

// Get statistics
let stats = system.get_stats();
println!("Total widgets: {}", stats.get("total_widgets").unwrap_or(&0));
```

### Kyma Integration

```rust
use widget_intelligence::*;
use std::collections::HashMap;

let mut extractor = KymaWidgetExtractor::new();

// Cache widget description from Kyma JSON
let kyma_json = r#"{
    "concreteEventID": 100,
    "label": "Master Volume",
    "minimum": 0.0,
    "maximum": 127.0,
    "displayType": "slider",
    "isGenerated": false
}"#;

let kyma_data: HashMap<String, serde_json::Value> = 
    serde_json::from_str(kyma_json)?;

extractor.cache_widget_description(kyma_data);

// Create training widget from cached data
let training_widget = extractor
    .create_training_widget(100, 95.0)
    .expect("Widget should be created from cached data");

println!("Created widget: {:?}", training_widget.label);
```

### Standalone Service

For non-Tauri applications, use the standalone service:

```rust
use widget_intelligence::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the service
    let service = StandaloneIntelligenceService::new("./intelligence.db")?;
    
    // Cache widget description
    let kyma_json = r#"{"concreteEventID": 5355, "label": "Amp_01", "minimum": 0.0, "maximum": 1.0 }"#;
    service.cache_widget_description(100, kyma_json.to_string()).await?;
    
    // Save preset and learn
    let mut widget_values = std::collections::HashMap::new();
    widget_values.insert("100".to_string(), 75.0);
    
    let preset_data = PresetData {
        name: "My Setup".to_string(),
        description: Some("Great for mixing".to_string()),
        widget_values,
        created_by: Some("user123".to_string()),
    };
    
    let stats = service.save_preset_and_learn(preset_data).await?;
    println!("Learned from preset. Total widgets: {}", stats.total_widgets);
    
    // Get suggestions
    let suggestions = service.get_widget_value_suggestions(
        100,
        Some("Vol".to_string()),
        Some("slider".to_string())
    ).await?;
    
    for suggestion in suggestions {
        println!("Suggestion: {:?} (confidence: {:.2})", 
                 suggestion.suggested_value, suggestion.confidence);
    }
    
    Ok(())
}
```

## API Reference

### Core Types

- `Widget`: Represents a UI widget with label, range, and current value
- `Preset`: Collection of widget values with metadata
- `Suggestion`: AI-generated suggestion with confidence score
- `WidgetRecord`: Internal storage format for learned widgets

### Main Classes

- `WidgetSuggestionEngine`: Core suggestion algorithm
- `PersistentWidgetSuggestionEngine`: Engine with database persistence
- `KymaWidgetExtractor`: Kyma JSON format processor
- `StandaloneIntelligenceService`: High-level service interface



## License

This project is licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.