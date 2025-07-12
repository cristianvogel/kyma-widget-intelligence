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
    let kyma_json = r#"{"concreteEventID": 100, "label": "Volume", "minimum": 0.0, "maximum": 100.0}"#;
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

## Tauri Integration

### Complete Tauri 2 Application Example

Here's a complete example of integrating Widget Intelligence with a Tauri 2 application:

#### 1. Cargo.toml for your Tauri app

```toml
[package]
name = "my-tauri-app"
version = "0.1.0"
edition = "2021"

[dependencies]
widget_intelligence = "0.1"
tauri = { version = "2.0", features = ["devtools"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
env_logger = "0.10"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }
```

#### 2. src-tauri/src/main.rs

```rust
use widget_intelligence::*;
use std::sync::Mutex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// State management for the intelligence system
pub struct AppState {
    pub intelligence: Mutex<PersistentWidgetSuggestionEngine>,
    pub extractor: Mutex<KymaWidgetExtractor>,
}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let intelligence = PersistentWidgetSuggestionEngine::new(db_path)
            .map_err(|e| format!("Failed to initialize intelligence system: {:?}", e))?;
        
        let extractor = KymaWidgetExtractor::new();
        
        Ok(Self {
            intelligence: Mutex::new(intelligence),
            extractor: Mutex::new(extractor),
        })
    }
}

// Tauri Commands
#[tauri::command]
async fn cache_widget_description(
    state: tauri::State<'_, AppState>,
    event_id: i64,
    kyma_json: String,
) -> Result<(), String> {
    log::info!("Caching widget description for event ID: {}", event_id);
    
    let kyma_data: HashMap<String, serde_json::Value> = serde_json::from_str(&kyma_json)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    KymaWidgetExtractor::validate_kyma_data(&kyma_data)
        .map_err(|e| format!("Invalid Kyma data: {}", e))?;
    
    let mut extractor = state.extractor.lock()
        .map_err(|_| "Failed to lock extractor")?;
    
    extractor.cache_widget_description(kyma_data);
    log::debug!("Successfully cached widget description for event ID: {}", event_id);
    Ok(())
}

#[tauri::command]
async fn save_preset_and_learn(
    state: tauri::State<'_, AppState>,
    preset_data: PresetData,
) -> Result<IntelligenceStats, String> {
    log::info!("Saving preset '{}' and learning from {} widgets", 
               preset_data.name, preset_data.widget_values.len());
    
    let mut intelligence = state.intelligence.lock()
        .map_err(|_| "Failed to lock intelligence system")?;
    
    let extractor = state.extractor.lock()
        .map_err(|_| "Failed to lock extractor")?;
    
    // Convert string keys to event IDs
    let event_values: HashMap<i64, f64> = preset_data.widget_values
        .into_iter()
        .filter_map(|(k, v)| {
            k.parse::<i64>().ok().map(|id| (id, v))
        })
        .collect();
    
    log::debug!("Processing {} valid event IDs", event_values.len());
    
    // Create and store training widgets
    let mut widget_values = Vec::new();
    for (event_id, current_value) in &event_values {
        if let Some(training_widget) = extractor.create_training_widget(*event_id, *current_value) {
            intelligence.store_widget(training_widget.clone())
                .map_err(|e| format!("Failed to store widget {}: {:?}", event_id, e))?;
            
            widget_values.push(WidgetValue {
                widget_id: event_id.to_string(),
                label: training_widget.label,
                value: *current_value,
                confidence: 1.0,
            });
            
            log::trace!("Stored training widget for event ID: {}", event_id);
        } else {
            log::warn!("No cached description found for event ID: {}", event_id);
        }
    }
    
    // Create and store the preset
    let preset = Preset {
        name: preset_data.name.clone(),
        description: preset_data.description,
        widget_values,
        created_by: preset_data.created_by,
        usage_count: 1,
        last_used: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };
    
    intelligence.store_preset(preset)
        .map_err(|e| format!("Failed to store preset: {:?}", e))?;
    
    // Return updated statistics
    let stats = intelligence.get_stats();
    let result = IntelligenceStats {
        total_widgets: stats.get("total_widgets").copied().unwrap_or(0),
        total_presets: stats.get("presets_stored").copied().unwrap_or(0),
        last_updated: chrono::Utc::now().to_rfc3339(),
        cache_size: extractor.cache_size(),
    };
    
    log::info!("Successfully saved preset '{}'. Total widgets: {}, Total presets: {}", 
               preset_data.name, result.total_widgets, result.total_presets);
    
    Ok(result)
}

#[tauri::command]
async fn get_widget_suggestions(
    state: tauri::State<'_, AppState>,
    event_id: i64,
    partial_label: Option<String>,
    display_type: Option<String>,
    max_suggestions: Option<usize>,
) -> Result<Vec<SuggestionResponse>, String> {
    log::debug!("Getting suggestions for event ID: {} with label: {:?}", 
                event_id, partial_label);
    
    let intelligence = state.intelligence.lock()
        .map_err(|_| "Failed to lock intelligence system")?;
    
    let partial_widget = Widget {
        label: partial_label,
        minimum: None,
        maximum: None,
        current_value: None,
        is_generated: None,
        display_type,
    };
    
    let max_suggestions = max_suggestions.unwrap_or(5);
    let suggestions = intelligence.get_suggestions(&partial_widget, max_suggestions);
    
    let responses: Vec<SuggestionResponse> = suggestions
        .into_iter()
        .map(|suggestion| SuggestionResponse {
            suggested_value: suggestion.suggested_value,
            confidence: suggestion.confidence,
            alternative_values: suggestion.alternative_values,
            reason: suggestion.reason,
        })
        .collect();
    
    log::debug!("Generated {} suggestions for event ID: {}", responses.len(), event_id);
    Ok(responses)
}

fn main() {
    // Initialize logging
    env_logger::init();
    
    // Get app data directory for database
    let app_data_dir = tauri::api::path::app_data_dir(&tauri::Config::default())
        .expect("Failed to get app data directory");
    
    std::fs::create_dir_all(&app_data_dir)
        .expect("Failed to create app data directory");
    
    let db_path = app_data_dir.join("widget_intelligence.db");
    
    log::info!("Initializing Widget Intelligence with database at: {:?}", db_path);
    
    // Initialize the app state
    let app_state = AppState::new(db_path.to_str().unwrap())
        .expect("Failed to initialize app state");
    
    log::info!("Widget Intelligence initialized successfully!");
    
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            cache_widget_description,
            save_preset_and_learn,
            get_widget_suggestions,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### 3. Frontend Integration (TypeScript)

```typescript
// src/types/intelligence.ts
export interface SuggestionResponse {
  suggested_value: number | null;
  confidence: number;
  alternative_values: number[];
  reason: string;
}

export interface PresetData {
  name: string;
  description?: string;
  widget_values: Record<string, number>;
  created_by?: string;
}

export interface IntelligenceStats {
  total_widgets: number;
  total_presets: number;
  last_updated: string;
  cache_size: number;
}
```

```typescript
// src/services/intelligence.ts
import { invoke } from '@tauri-apps/api/core';

export class IntelligenceService {
  
  async cacheWidgetDescription(eventId: number, kymaJson: string): Promise<void> {
    await invoke('cache_widget_description', { eventId, kymaJson });
  }
  
  async savePresetAndLearn(presetData: PresetData): Promise<IntelligenceStats> {
    return await invoke('save_preset_and_learn', { presetData });
  }
  
  async getWidgetSuggestions(
    eventId: number, 
    partialLabel?: string, 
    displayType?: string,
    maxSuggestions?: number
  ): Promise<SuggestionResponse[]> {
    return await invoke('get_widget_suggestions', { 
      eventId, 
      partialLabel, 
      displayType, 
      maxSuggestions 
    });
  }
}

export const intelligenceService = new IntelligenceService();
```

#### 4. Smart Widget Component (React)

```typescript
// src/components/SmartWidget.tsx
import React, { useState, useEffect } from 'react';
import { intelligenceService } from '../services/intelligence';

interface SmartWidgetProps {
  eventId: number;
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
}

export const SmartWidget: React.FC<SmartWidgetProps> = ({
  eventId,
  label,
  value,
  min,
  max,
  onChange
}) => {
  const [suggestions, setSuggestions] = useState([]);
  const [showSuggestions, setShowSuggestions] = useState(false);

  useEffect(() => {
    loadSuggestions();
  }, [label]);

  const loadSuggestions = async () => {
    try {
      const suggestions = await intelligenceService.getWidgetSuggestions(
        eventId, 
        label, 
        'slider'
      );
      setSuggestions(suggestions);
    } catch (error) {
      console.error('Failed to load suggestions:', error);
    }
  };

  const applySuggestion = (suggestedValue: number) => {
    onChange(suggestedValue);
    setShowSuggestions(false);
  };

  return (
    <div className="smart-widget">
      <label>{label}</label>
      <div className="widget-controls">
        <input
          type="range"
          min={min}
          max={max}
          value={value}
          onChange={(e) => onChange(Number(e.target.value))}
        />
        <span>{value}</span>
        
        {suggestions.length > 0 && (
          <button 
            onClick={() => setShowSuggestions(!showSuggestions)}
            className="suggestions-button"
          >
            💡 {suggestions.length} suggestions
          </button>
        )}
      </div>
      
      {showSuggestions && (
        <div className="suggestions-panel">
          <h4>Smart Suggestions</h4>
          {suggestions.map((suggestion, index) => (
            <div key={index} className="suggestion-item">
              <button
                onClick={() => applySuggestion(suggestion.suggested_value!)}
                disabled={suggestion.suggested_value === null}
              >
                {suggestion.suggested_value?.toFixed(1) || 'N/A'}
              </button>
              <span className="confidence">
                {(suggestion.confidence * 100).toFixed(0)}% confidence
              </span>
              <span className="reason">{suggestion.reason}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
```

This comprehensive example shows how to integrate Widget Intelligence with a Tauri 2 application, including:

- Complete Rust backend with all Tauri commands
- TypeScript service layer for frontend communication
- React components demonstrating smart widget suggestions
- Preset management with learning capabilities
- Error handling and logging
- Database persistence and statistics

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

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under either of:

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.