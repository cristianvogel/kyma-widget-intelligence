# Widget Intelligence - Comprehensive Examples Guide

This guide provides detailed examples and integration patterns for the Widget Intelligence library.

## Table of Contents

1. [Library Usage Examples](#library-usage-examples)
2. [Tauri 2 Integration](#tauri-2-integration)
3. [Advanced Patterns](#advanced-patterns)
4. [Production Deployment](#production-deployment)
5. [Troubleshooting](#troubleshooting)

## Library Usage Examples

### Basic Widget Learning

```rust
use widget_intelligence::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new suggestion engine
    let mut engine = WidgetSuggestionEngine::new();

    // Define some training widgets
    let volume_widgets = vec![
        Widget {
            label: Some("Master Volume".to_string()),
            minimum: Some(0.0),
            maximum: Some(127.0),
            current_value: Some(95.0),
            is_generated: Some(false),
            display_type: Some("slider".to_string()),
        },
        Widget {
            label: Some("Channel Volume".to_string()),
            minimum: Some(0.0),
            maximum: Some(127.0),
            current_value: Some(80.0),
            is_generated: Some(false),
            display_type: Some("slider".to_string()),
        },
        Widget {
            label: Some("Output Level".to_string()),
            minimum: Some(0.0),
            maximum: Some(100.0),
            current_value: Some(85.0),
            is_generated: Some(false),
            display_type: Some("knob".to_string()),
        },
    ];

    // Store training data
    for widget in volume_widgets {
        engine.store_widget(widget);
    }

    // Get suggestions for a new widget
    let partial_widget = Widget {
        label: Some("Volume Control".to_string()),
        minimum: Some(0.0),
        maximum: Some(127.0),
        display_type: Some("slider".to_string()),
        ..Default::default()
    };

    let suggestions = engine.get_suggestions(&partial_widget, 3);
    
    println!("Suggestions for 'Volume Control':");
    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("  {}. Value: {:?}, Confidence: {:.2}%, Reason: {}", 
                 i + 1,
                 suggestion.suggested_value,
                 suggestion.confidence * 100.0,
                 suggestion.reason);
    }

    Ok(())
}
```

### Persistent Storage with Presets

```rust
use widget_intelligence::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize persistent storage
    let mut system = init_intelligence_system("./audio_widgets.db")?;

    // Create a comprehensive preset
    let audio_preset = Preset {
        name: "Rock Mix".to_string(),
        description: Some("Balanced rock music settings".to_string()),
        widget_values: vec![
            WidgetValue {
                widget_id: "master_volume".to_string(),
                label: Some("Master Volume".to_string()),
                value: 85.0,
                confidence: 1.0,
            },
            WidgetValue {
                widget_id: "bass_eq".to_string(),
                label: Some("Bass EQ".to_string()),
                value: 65.0,
                confidence: 1.0,
            },
            WidgetValue {
                widget_id: "treble_eq".to_string(),
                label: Some("Treble EQ".to_string()),
                value: 70.0,
                confidence: 1.0,
            },
            WidgetValue {
                widget_id: "reverb_send".to_string(),
                label: Some("Reverb Send".to_string()),
                value: 25.0,
                confidence: 1.0,
            },
        ],
        created_by: Some("audio_engineer_123".to_string()),
        usage_count: 1,
        last_used: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };

    // Store the preset (this will create training widgets)
    system.store_preset(audio_preset)?;

    // Later, get suggestions for EQ controls
    let eq_widget = Widget {
        label: Some("Mid EQ".to_string()),
        minimum: Some(0.0),
        maximum: Some(100.0),
        display_type: Some("knob".to_string()),
        ..Default::default()
    };

    let suggestions = system.get_suggestions(&eq_widget, 5);
    println!("EQ suggestions based on learned patterns:");
    for suggestion in suggestions {
        println!("  Suggested: {:.1}, Confidence: {:.1}%", 
                 suggestion.suggested_value.unwrap_or(0.0),
                 suggestion.confidence * 100.0);
    }

    // Get insights about the widget
    if let Some(insights) = system.get_preset_insights(&eq_widget) {
        println!("Insights: {}", insights);
    }

    // Export all learned data for backup
    let export_data = system.export_data()?;
    let json_export = serde_json::to_string_pretty(&export_data)?;
    println!("Exported {} widgets and {} presets", 
             export_data.widgets.len(), 
             export_data.presets.len());

    Ok(())
}
```

### Kyma Integration Example

```rust
use widget_intelligence::*;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut extractor = KymaWidgetExtractor::new();

    // Simulate receiving widget descriptions from Kyma
    let kyma_descriptions = vec![
        r#"{
            "concreteEventID": 100,
            "label": "Master Volume",
            "minimum": 0.0,
            "maximum": 127.0,
            "displayType": "slider",
            "isGenerated": false,
            "category": "Audio",
            "units": "dB"
        }"#,
        r#"{
            "concreteEventID": 101,
            "label": "Pan Position",
            "minimum": -64.0,
            "maximum": 64.0,
            "displayType": "rotary",
            "isGenerated": false,
            "category": "Audio",
            "units": "position"
        }"#,
        r#"{
            "concreteEventID": 102,
            "label": "LFO Rate",
            "minimum": 0.1,
            "maximum": 20.0,
            "displayType": "slider",
            "isGenerated": true,
            "category": "Modulation",
            "units": "Hz"
        }"#,
    ];

    // Cache all widget descriptions
    for kyma_json in kyma_descriptions {
        let kyma_data: HashMap<String, serde_json::Value> = 
            serde_json::from_str(kyma_json)?;
        
        // Validate the data first
        KymaWidgetExtractor::validate_kyma_data(&kyma_data)?;
        
        extractor.cache_widget_description(kyma_data);
    }

    println!("Cached {} widget descriptions", extractor.cache_size());

    // Simulate user setting values and creating training data
    let user_values = vec![
        (100, 95.0),  // Master Volume at 95
        (101, 0.0),   // Pan centered
        (102, 2.5),   // LFO at moderate rate
    ];

    let mut training_widgets = Vec::new();
    for (event_id, value) in user_values {
        if let Some(widget) = extractor.create_training_widget(event_id, value) {
            println!("Created training widget for {}: {} = {}", 
                     event_id, 
                     widget.label.as_ref().unwrap_or(&"Unknown".to_string()),
                     value);
            training_widgets.push(widget);
        }
    }

    // Use training widgets with the suggestion engine
    let mut engine = WidgetSuggestionEngine::new();
    for widget in training_widgets {
        engine.store_widget(widget);
    }

    // Get suggestions for a new LFO widget
    let lfo_query = Widget {
        label: Some("LFO".to_string()),
        display_type: Some("slider".to_string()),
        ..Default::default()
    };

    let suggestions = engine.get_suggestions(&lfo_query, 3);
    println!("\nSuggestions for new LFO widget:");
    for suggestion in suggestions {
        println!("  Value: {:.2}, Confidence: {:.1}%", 
                 suggestion.suggested_value.unwrap_or(0.0),
                 suggestion.confidence * 100.0);
    }

    Ok(())
}
```

### Standalone Service Example

```rust
use widget_intelligence::*;
use std::collections::HashMap;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the standalone service
    let service = StandaloneIntelligenceService::new("./studio_intelligence.db")?;

    // Simulate a studio session workflow
    println!("=== Studio Session Intelligence Demo ===\n");

    // 1. Cache some widget descriptions as they're discovered
    let widget_descriptions = vec![
        (200, r#"{"concreteEventID": 200, "label": "Compressor Ratio", "minimum": 1.0, "maximum": 20.0, "displayType": "knob"}"#),
        (201, r#"{"concreteEventID": 201, "label": "Compressor Attack", "minimum": 0.1, "maximum": 100.0, "displayType": "slider"}"#),
        (202, r#"{"concreteEventID": 202, "label": "Compressor Release", "minimum": 10.0, "maximum": 1000.0, "displayType": "slider"}"#),
        (203, r#"{"concreteEventID": 203, "label": "Gate Threshold", "minimum": -60.0, "maximum": 0.0, "displayType": "slider"}"#),
    ];

    for (event_id, description) in widget_descriptions {
        service.cache_widget_description(event_id, description.to_string()).await?;
        println!("Cached description for event ID: {}", event_id);
    }

    // 2. User creates several presets during the session
    let presets = vec![
        ("Vocal Compression", vec![(200, 4.0), (201, 5.0), (202, 50.0)]),
        ("Drum Gate", vec![(203, -30.0)]),
        ("Guitar Compression", vec![(200, 6.0), (201, 2.0), (202, 30.0)]),
        ("Heavy Compression", vec![(200, 8.0), (201, 1.0), (202, 20.0)]),
    ];

    for (preset_name, values) in presets {
        let mut widget_values = HashMap::new();
        for (event_id, value) in values {
            widget_values.insert(event_id.to_string(), value);
        }

        let preset_data = PresetData {
            name: preset_name.to_string(),
            description: Some(format!("Preset created during studio session")),
            widget_values,
            created_by: Some("studio_engineer".to_string()),
        };

        let stats = service.save_preset_and_learn(preset_data).await?;
        println!("Saved preset '{}' - Total learned widgets: {}", 
                 preset_name, stats.total_widgets);
        
        // Small delay to simulate real usage
        sleep(Duration::from_millis(100)).await;
    }

    // 3. Now get intelligent suggestions for new compressor settings
    println!("\n=== Getting Smart Suggestions ===");
    
    let suggestions = service.get_widget_value_suggestions(
        999, // New widget ID
        Some("Compressor".to_string()),
        Some("knob".to_string())
    ).await?;

    println!("Suggestions for new compressor widget:");
    for (i, suggestion) in suggestions.iter().enumerate() {
        println!("  {}. Value: {:.2}, Confidence: {:.1}%", 
                 i + 1,
                 suggestion.suggested_value.unwrap_or(0.0),
                 suggestion.confidence * 100.0);
        println!("     Reason: {}", suggestion.reason);
        if !suggestion.alternative_values.is_empty() {
            println!("     Alternatives: {:?}", suggestion.alternative_values);
        }
        println!();
    }

    // 4. Show final statistics
    let final_stats = service.get_intelligence_stats().await?;
    println!("=== Session Summary ===");
    println!("Total widgets learned: {}", final_stats.total_widgets);
    println!("Total presets stored: {}", final_stats.total_presets);
    println!("Cache size: {}", final_stats.cache_size);
    println!("Last updated: {}", final_stats.last_updated);

    Ok(())
}
```

## Tauri 2 Integration

### Complete Application Structure

```
my-tauri-app/
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── intelligence.rs
│   │   └── commands/
│   │       ├── mod.rs
│   │       ├── widget_commands.rs
│   │       └── preset_commands.rs
│   └── tauri.conf.json
├── src/
│   ├── services/
│   │   └── intelligence.ts
│   ├── components/
│   │   ├── SmartWidget.tsx
│   │   ├── PresetManager.tsx
│   │   └── IntelligenceStats.tsx
│   └── types/
│       └── intelligence.ts
└── package.json
```

### Rust Backend Implementation

#### src-tauri/Cargo.toml

```toml
[package]
name = "studio-intelligence-app"
version = "0.1.0"
edition = "2021"

[dependencies]
widget_intelligence = "0.1"
tauri = { version = "2.0", features = ["macos-private-api", "devtools"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
env_logger = "0.10"
anyhow = "1.0"
chrono = { version = "0.4", features = ["serde"] }

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

#### src-tauri/src/intelligence.rs

```rust
use widget_intelligence::*;
use std::sync::Mutex;
use std::path::PathBuf;
use anyhow::Result;

pub struct IntelligenceManager {
    pub system: Mutex<PersistentWidgetSuggestionEngine>,
    pub extractor: Mutex<KymaWidgetExtractor>,
    pub db_path: PathBuf,
}

impl IntelligenceManager {
    pub fn new(app_data_dir: PathBuf) -> Result<Self> {
        let db_path = app_data_dir.join("widget_intelligence.sled");
        
        log::info!("Initializing Intelligence Manager with DB: {:?}", db_path);
        
        let system = PersistentWidgetSuggestionEngine::new(&db_path)
            .map_err(|e| anyhow::anyhow!("Failed to initialize persistence: {:?}", e))?;
        
        let extractor = KymaWidgetExtractor::new();
        
        Ok(Self {
            system: Mutex::new(system),
            extractor: Mutex::new(extractor),
            db_path,
        })
    }

    pub fn get_system_info(&self) -> Result<SystemInfo> {
        let system = self.system.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock system"))?;
        
        let extractor = self.extractor.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock extractor"))?;
        
        let stats = system.get_stats();
        let db_size = system.size_on_disk()
            .map_err(|e| anyhow::anyhow!("Failed to get DB size: {:?}", e))?;
        
        Ok(SystemInfo {
            total_widgets: stats.get("total_widgets").copied().unwrap_or(0),
            total_presets: stats.get("presets_stored").copied().unwrap_or(0),
            cache_size: extractor.cache_size(),
            db_size_bytes: db_size,
            db_path: self.db_path.to_string_lossy().to_string(),
            uptime_seconds: 0, // You could track this if needed
        })
    }

    pub fn backup_data(&self) -> Result<String> {
        let system = self.system.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock system"))?;
        
        let export_data = system.export_data()
            .map_err(|e| anyhow::anyhow!("Failed to export data: {:?}", e))?;
        
        serde_json::to_string_pretty(&export_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize data: {}", e))
    }

    pub fn restore_data(&self, json_data: &str) -> Result<()> {
        let mut system = self.system.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock system"))?;
        
        let import_data: ExportData = serde_json::from_str(json_data)
            .map_err(|e| anyhow::anyhow!("Failed to parse import data: {}", e))?;
        
        system.import_data(import_data)
            .map_err(|e| anyhow::anyhow!("Failed to import data: {:?}", e))?;
        
        log::info!("Successfully restored intelligence data from backup");
        Ok(())
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SystemInfo {
    pub total_widgets: usize,
    pub total_presets: usize,
    pub cache_size: usize,
    pub db_size_bytes: u64,
    pub db_path: String,
    pub uptime_seconds: u64,
}
```

#### src-tauri/src/commands/widget_commands.rs

```rust
use crate::intelligence::IntelligenceManager;
use widget_intelligence::*;
use std::collections::HashMap;

#[tauri::command]
pub async fn cache_widget_description(
    intelligence: tauri::State<'_, IntelligenceManager>,
    event_id: i64,
    kyma_json: String,
) -> Result<(), String> {
    log::debug!("Caching widget description for event ID: {}", event_id);
    
    // Parse and validate JSON
    let kyma_data: HashMap<String, serde_json::Value> = serde_json::from_str(&kyma_json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    KymaWidgetExtractor::validate_kyma_data(&kyma_data)
        .map_err(|e| format!("Invalid Kyma data: {}", e))?;
    
    // Cache the description
    let mut extractor = intelligence.extractor.lock()
        .map_err(|_| "Failed to acquire extractor lock")?;
    
    extractor.cache_widget_description(kyma_data);
    
    log::info!("Successfully cached widget description for event ID: {}", event_id);
    Ok(())
}

#[tauri::command]
pub async fn get_widget_suggestions(
    intelligence: tauri::State<'_, IntelligenceManager>,
    event_id: i64,
    partial_label: Option<String>,
    display_type: Option<String>,
    minimum: Option<f64>,
    maximum: Option<f64>,
    max_suggestions: Option<usize>,
) -> Result<Vec<EnhancedSuggestionResponse>, String> {
    log::debug!("Getting suggestions for event ID: {} with label: {:?}", 
                event_id, partial_label);
    
    let system = intelligence.system.lock()
        .map_err(|_| "Failed to acquire system lock")?;
    
    let partial_widget = Widget {
        label: partial_label.clone(),
        minimum,
        maximum,
        current_value: None,
        is_generated: None,
        display_type: display_type.clone(),
    };
    
    let max_suggestions = max_suggestions.unwrap_or(5).min(10); // Cap at 10
    let suggestions = system.get_suggestions(&partial_widget, max_suggestions);
    
    // Convert to enhanced response format
    let responses: Vec<EnhancedSuggestionResponse> = suggestions
        .into_iter()
        .enumerate()
        .map(|(index, suggestion)| EnhancedSuggestionResponse {
            rank: index + 1,
            suggested_value: suggestion.suggested_value,
            confidence: suggestion.confidence,
            confidence_percentage: (suggestion.confidence * 100.0).round() as u8,
            alternative_values: suggestion.alternative_values,
            reason: suggestion.reason,
            source_widget_label: suggestion.widget.label,
            source_widget_type: suggestion.widget.display_type,
            value_confidence,
            is_recommended: suggestion.confidence > 0.7,
        })
        .collect();
    
    log::info!("Generated {} suggestions for event ID: {}", responses.len(), event_id);
    Ok(responses)
}

#[tauri::command]
pub async fn get_widget_insights(
    intelligence: tauri::State<'_, IntelligenceManager>,
    partial_label: Option<String>,
    display_type: Option<String>,
) -> Result<DetailedInsightResponse, String> {
    let system = intelligence.system.lock()
        .map_err(|_| "Failed to acquire system lock")?;
    
    let partial_widget = Widget {
        label: partial_label,
        minimum: None,
        maximum: None,
        current_value: None,
        is_generated: None,
        display_type,
    };
    
    let insights = system.get_preset_insights(&partial_widget);
    let suggestions = system.get_suggestions(&partial_widget, 3);
    
    let suggested_values: Vec<f64> = suggestions.iter()
        .filter_map(|s| s.suggested_value)
        .collect();
    
    let confidence_scores: Vec<f64> = suggestions.iter()
        .map(|s| s.confidence)
        .collect();
    
    let avg_confidence = if !confidence_scores.is_empty() {
        confidence_scores.iter().sum::<f64>() / confidence_scores.len() as f64
    } else {
        0.0
    };
    
    Ok(DetailedInsightResponse {
        insights,
        suggested_values,
        confidence_scores,
        average_confidence: avg_confidence,
        data_points: suggestions.len(),
        recommendation_quality: match avg_confidence {
            c if c > 0.8 => "Excellent".to_string(),
            c if c > 0.6 => "Good".to_string(),
            c if c > 0.4 => "Fair".to_string(),
            _ => "Limited".to_string(),
        },
    })
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct EnhancedSuggestionResponse {
    pub rank: usize,
    pub suggested_value: Option<f64>,
    pub confidence: f64,
    pub confidence_percentage: u8,
    pub alternative_values: Vec<f64>,
    pub reason: String,
    pub source_widget_label: Option<String>,
    pub source_widget_type: Option<String>,
    pub value_confidence: f64,
    pub is_recommended: bool,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct DetailedInsightResponse {
    pub insights: Option<String>,
    pub suggested_values: Vec<f64>,
    pub confidence_scores: Vec<f64>,
    pub average_confidence: f64,
    pub data_points: usize,
    pub recommendation_quality: String,
}
```

### Frontend TypeScript Implementation

#### src/services/intelligence.ts

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export interface EnhancedSuggestionResponse {
  rank: number;
  suggested_value: number | null;
  confidence: number;
  confidence_percentage: number;
  alternative_values: number[];
  reason: string;
  source_widget_label: string | null;
  source_widget_type: string | null;
  value_confidence: number;
  is_recommended: boolean;
}

export interface DetailedInsightResponse {
  insights: string | null;
  suggested_values: number[];
  confidence_scores: number[];
  average_confidence: number;
  data_points: number;
  recommendation_quality: string;
}

export interface SystemInfo {
  total_widgets: number;
  total_presets: number;
  cache_size: number;
  db_size_bytes: number;
  db_path: string;
  uptime_seconds: number;
}

export class IntelligenceService {
  private static instance: IntelligenceService;
  private eventListeners: Map<string, Function[]> = new Map();

  private constructor() {
    this.setupEventListeners();
  }

  public static getInstance(): IntelligenceService {
    if (!IntelligenceService.instance) {
      IntelligenceService.instance = new IntelligenceService();
    }
    return IntelligenceService.instance;
  }

  private async setupEventListeners() {
    // Listen for intelligence system events
    await listen('intelligence-stats-updated', (event) => {
      this.emit('stats-updated', event.payload);
    });

    await listen('widget-learned', (event) => {
      this.emit('widget-learned', event.payload);
    });

    await listen('preset-saved', (event) => {
      this.emit('preset-saved', event.payload);
    });
  }

  public on(event: string, callback: Function) {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event)!.push(callback);
  }

  public off(event: string, callback: Function) {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    }
  }

  private emit(event: string, data: any) {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      listeners.forEach(callback => callback(data));
    }
  }

  async cacheWidgetDescription(
    eventId: number, 
    kymaJson: string
  ): Promise<void> {
    try {
      await invoke('cache_widget_description', { 
        eventId, 
        kymaJson 
      });
    } catch (error) {
      console.error('Failed to cache widget description:', error);
      throw new Error(`Failed to cache widget description: ${error}`);
    }
  }

  async getWidgetSuggestions(
    eventId: number,
    partialLabel?: string,
    displayType?: string,
    minimum?: number,
    maximum?: number,
    maxSuggestions?: number
  ): Promise<EnhancedSuggestionResponse[]> {
    try {
      return await invoke('get_widget_suggestions', {
        eventId,
        partialLabel,
        displayType,
        minimum,
        maximum,
        maxSuggestions: maxSuggestions || 5,
      });
    } catch (error) {
      console.error('Failed to get widget suggestions:', error);
      throw new Error(`Failed to get suggestions: ${error}`);
    }
  }

  async getWidgetInsights(
    partialLabel?: string,
    displayType?: string
  ): Promise<DetailedInsightResponse> {
    try {
      return await invoke('get_widget_insights', {
        partialLabel,
        displayType,
      });
    } catch (error) {
      console.error('Failed to get widget insights:', error);
      throw new Error(`Failed to get insights: ${error}`);
    }
  }

  async savePresetAndLearn(presetData: PresetData): Promise<IntelligenceStats> {
    try {
      const stats = await invoke('save_preset_and_learn', { presetData });
      this.emit('preset-saved', { presetData, stats });
      return stats;
    } catch (error) {
      console.error('Failed to save preset:', error);
      throw new Error(`Failed to save preset: ${error}`);
    }
  }

  async getSystemInfo(): Promise<SystemInfo> {
    try {
      return await invoke('get_system_info');
    } catch (error) {
      console.error('Failed to get system info:', error);
      throw new Error(`Failed to get system info: ${error}`);
    }
  }

  async backupData(): Promise<string> {
    try {
      return await invoke('backup_intelligence_data');
    } catch (error) {
      console.error('Failed to backup data:', error);
      throw new Error(`Failed to backup data: ${error}`);
    }
  }

  async restoreData(jsonData: string): Promise<void> {
    try {
      await invoke('restore_intelligence_data', { jsonData });
      this.emit('data-restored', {});
    } catch (error) {
      console.error('Failed to restore data:', error);
      throw new Error(`Failed to restore data: ${error}`);
    }
  }
}

export const intelligenceService = IntelligenceService.getInstance();
```

#### src/components/SmartWidget.tsx

```tsx
import React, { useState, useEffect, useMemo } from 'react';
import { intelligenceService, EnhancedSuggestionResponse } from '../services/intelligence';

interface SmartWidgetProps {
  eventId: number;
  label: string;
  value: number;
  min: number;
  max: number;
  displayType?: 'slider' | 'knob' | 'button';
  onChange: (value: number) => void;
  onLearningEvent?: (eventId: number, value: number) => void;
}

export const SmartWidget: React.FC<SmartWidgetProps> = ({
  eventId,
  label,
  value,
  min,
  max,
  displayType = 'slider',
  onChange,
  onLearningEvent,
}) => {
  const [suggestions, setSuggestions] = useState<EnhancedSuggestionResponse[]>([]);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [loading, setLoading] = useState(false);
  const [lastSuggestedValue, setLastSuggestedValue] = useState<number | null>(null);

  // Debounced label for suggestions
  const debouncedLabel = useMemo(() => {
    const timer = setTimeout(() => loadSuggestions(), 300);
    return () => clearTimeout(timer);
  }, [label, displayType]);

  useEffect(() => {
    return debouncedLabel;
  }, [debouncedLabel]);

  const loadSuggestions = async () => {
    if (!label.trim()) return;
    
    setLoading(true);
    try {
      const suggestions = await intelligenceService.getWidgetSuggestions(
        eventId,
        label,
        displayType,
        min,
        max,
        5
      );
      setSuggestions(suggestions);
    } catch (error) {
      console.error('Failed to load suggestions:', error);
    } finally {
      setLoading(false);
    }
  };

  const applySuggestion = (suggestion: EnhancedSuggestionResponse) => {
    if (suggestion.suggested_value !== null) {
      onChange(suggestion.suggested_value);
      setLastSuggestedValue(suggestion.suggested_value);
      setShowSuggestions(false);
      
      // Trigger learning event
      if (onLearningEvent) {
        onLearningEvent(eventId, suggestion.suggested_value);
      }
    }
  };

  const getConfidenceColor = (confidence: number): string => {
    if (confidence > 0.8) return 'text-green-600';
    if (confidence > 0.6) return 'text-blue-600';
    if (confidence > 0.4) return 'text-yellow-600';
    return 'text-gray-600';
  };

  const getConfidenceIcon = (confidence: number): string => {
    if (confidence > 0.8) return '🟢';
    if (confidence > 0.6) return '🔵';
    if (confidence > 0.4) return '🟡';
    return '⚪';
  };

  const highConfidenceSuggestions = suggestions.filter(s => s.is_recommended);
  const hasSuggestions = suggestions.length > 0;
  const hasHighConfidenceSuggestions = highConfidenceSuggestions.length > 0;

  return (
    <div className="smart-widget bg-white rounded-lg shadow-md p-4 border border-gray-200">
      {/* Widget Header */}
      <div className="flex items-center justify-between mb-3">
        <label className="text-sm font-medium text-gray-700">
          {label}
        </label>
        
        {hasSuggestions && (
          <div className="flex items-center space-x-2">
            {hasHighConfidenceSuggestions && (
              <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
                🤖 Smart
              </span>
            )}
            
            <button
              onClick={() => setShowSuggestions(!showSuggestions)}
              className={`inline-flex items-center px-3 py-1 rounded-md text-xs font-medium transition-colors ${
                hasHighConfidenceSuggestions
                  ? 'bg-blue-100 text-blue-700 hover:bg-blue-200'
                  : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
              }`}
              disabled={loading}
            >
              {loading ? '⏳' : '💡'} {suggestions.length} suggestions
            </button>
          </div>
        )}
      </div>

      {/* Widget Control */}
      <div className="mb-3">
        {displayType === 'slider' && (
          <div className="flex items-center space-x-3">
            <input
              type="range"
              min={min}
              max={max}
              step={0.1}
              value={value}
              onChange={(e) => onChange(Number(e.target.value))}
              className="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
            />
            <span className="min-w-[60px] text-sm font-mono text-gray-600">
              {value.toFixed(1)}
            </span>
          </div>
        )}

        {displayType === 'knob' && (
          <div className="flex items-center justify-center">
            <div className="relative">
              <input
                type="range"
                min={min}
                max={max}
                step={0.1}
                value={value}
                onChange={(e) => onChange(Number(e.target.value))}
                className="w-16 h-16 rounded-full appearance-none bg-gray-200 cursor-pointer"
                style={{
                  background: `conic-gradient(from 0deg, #3b82f6 0deg, #3b82f6 ${
                    ((value - min) / (max - min)) * 360
                  }deg, #e5e7eb ${((value - min) / (max - min)) * 360}deg, #e5e7eb 360deg)`
                }}
              />
              <div className="absolute inset-0 flex items-center justify-center">
                <span className="text-xs font-mono text-gray-700">
                  {value.toFixed(1)}
                </span>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Value Change Indicator */}
      {lastSuggestedValue !== null && Math.abs(value - lastSuggestedValue) < 0.1 && (
        <div className="mb-3 p-2 bg-green-50 rounded-md">
          <p className="text-xs text-green-700">
            ✅ Applied AI suggestion: {lastSuggestedValue.toFixed(1)}
          </p>
        </div>
      )}

      {/* Suggestions Panel */}
      {showSuggestions && (
        <div className="mt-3 p-3 bg-gray-50 rounded-md border">
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-sm font-medium text-gray-700">Smart Suggestions</h4>
            <button
              onClick={() => setShowSuggestions(false)}
              className="text-gray-400 hover:text-gray-600"
            >
              ✕
            </button>
          </div>

          {suggestions.length === 0 ? (
            <p className="text-xs text-gray-500">No suggestions available yet</p>
          ) : (
            <div className="space-y-2">
              {suggestions.slice(0, 3).map((suggestion, index) => (
                <div
                  key={index}
                  className="flex items-center justify-between p-2 bg-white rounded border hover:bg-gray-50 transition-colors"
                >
                  <div className="flex-1">
                    <div className="flex items-center space-x-2">
                      <span className="text-sm font-medium text-gray-900">
                        {suggestion.suggested_value?.toFixed(1) || 'N/A'}
                      </span>
                      <span className={`text-xs ${getConfidenceColor(suggestion.confidence)}`}>
                        {getConfidenceIcon(suggestion.confidence)} {suggestion.confidence_percentage}%
                      </span>
                      {suggestion.is_recommended && (
                        <span className="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                          Recommended
                        </span>
                      )}
                    </div>
                    <p className="text-xs text-gray-500 mt-1">
                      {suggestion.reason}
                    </p>
                  </div>
                  
                  <button
                    onClick={() => applySuggestion(suggestion)}
                    disabled={suggestion.suggested_value === null}
                    className="ml-2 px-2 py-1 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 disabled:bg-gray-300 disabled:cursor-not-allowed transition-colors"
                  >
                    Apply
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
};
```

## Advanced Patterns

### Learning Rate Adjustment

```rust
use widget_intelligence::*;

struct AdaptiveLearningEngine {
    base_engine: WidgetSuggestionEngine,
    learning_rate: f64,
    confidence_threshold: f64,
    adaptation_factor: f64,
}

impl AdaptiveLearningEngine {
    pub fn new() -> Self {
        Self {
            base_engine: WidgetSuggestionEngine::new(),
            learning_rate: 1.0,
            confidence_threshold: 0.6,
            adaptation_factor: 0.95,
        }
    }

    pub fn store_widget_with_feedback(&mut self, widget: Widget, user_accepted: bool) {
        // Adjust learning rate based on user feedback
        if user_accepted {
            self.learning_rate = (self.learning_rate * 1.05).min(2.0);
        } else {
            self.learning_rate *= self.adaptation_factor;
        }

        // Store the widget with modified frequency based on learning rate
        let mut modified_widget = widget;
        
        // Apply learning rate to the widget's importance
        self.base_engine.store_widget(modified_widget);
        
        log::info!("Learning rate adjusted to: {:.3}", self.learning_rate);
    }

    pub fn get_adaptive_suggestions(&self, partial_widget: &Widget, max_suggestions: usize) -> Vec<Suggestion> {
        let mut suggestions = self.base_engine.get_suggestions(partial_widget, max_suggestions);
        
        // Filter suggestions based on current confidence threshold
        suggestions.retain(|s| s.confidence >= self.confidence_threshold);
        
        // Adjust confidence scores based on learning rate
        for suggestion in &mut suggestions {
            suggestion.confidence = (suggestion.confidence * self.learning_rate).min(1.0);
        }
        
        suggestions
    }
}
```

### Multi-User Learning

```rust
use widget_intelligence::*;
use std::collections::HashMap;

pub struct MultiUserIntelligenceSystem {
    user_engines: HashMap<String, WidgetSuggestionEngine>,
    global_engine: WidgetSuggestionEngine,
    collaborative_weight: f64,
}

impl MultiUserIntelligenceSystem {
    pub fn new() -> Self {
        Self {
            user_engines: HashMap::new(),
            global_engine: WidgetSuggestionEngine::new(),
            collaborative_weight: 0.3, // 30% global, 70% personal
        }
    }

    pub fn store_user_widget(&mut self, user_id: &str, widget: Widget) {
        // Store in user-specific engine
        let user_engine = self.user_engines.entry(user_id.to_string())
            .or_insert_with(WidgetSuggestionEngine::new);
        user_engine.store_widget(widget.clone());

        // Also store in global engine with reduced weight
        self.global_engine.store_widget(widget);
    }

    pub fn get_personalized_suggestions(
        &self, 
        user_id: &str, 
        partial_widget: &Widget, 
        max_suggestions: usize
    ) -> Vec<Suggestion> {
        let user_suggestions = self.user_engines.get(user_id)
            .map(|engine| engine.get_suggestions(partial_widget, max_suggestions))
            .unwrap_or_default();

        let global_suggestions = self.global_engine.get_suggestions(partial_widget, max_suggestions);

        // Combine suggestions with weighted confidence
        let mut combined_suggestions = user_suggestions;
        
        for global_suggestion in global_suggestions {
            // Check if we already have a similar suggestion from user data
            let similar_exists = combined_suggestions.iter().any(|user_sugg| {
                user_sugg.widget.label == global_suggestion.widget.label
            });

            if !similar_exists {
                let mut adjusted_suggestion = global_suggestion;
                adjusted_suggestion.confidence *= self.collaborative_weight;
                combined_suggestions.push(adjusted_suggestion);
            }
        }

        // Sort by confidence and limit results
        combined_suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        combined_suggestions.truncate(max_suggestions);
        
        combined_suggestions
    }
}
```

## Production Deployment

### Production Configuration

```rust
// src-tauri/src/config.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    pub database_path: Option<PathBuf>,
    pub backup_interval_hours: u64,
    pub max_cache_size: usize,
    pub learning_rate: f64,
    pub confidence_threshold: f64,
    pub auto_backup: bool,
    pub telemetry_enabled: bool,
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            database_path: None, // Will use app data dir
            backup_interval_hours: 24,
            max_cache_size: 1000,
            learning_rate: 1.0,
            confidence_threshold: 0.3,
            auto_backup: true,
            telemetry_enabled: false,
        }
    }
}

impl IntelligenceConfig {
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

### Monitoring and Analytics

```rust
// src-tauri/src/monitoring.rs
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::interval;

#[derive(Debug, Clone)]
pub struct IntelligenceMetrics {
    pub suggestions_requested: Arc<AtomicU64>,
    pub suggestions_applied: Arc<AtomicU64>,
    pub widgets_learned: Arc<AtomicU64>,
    pub presets_saved: Arc<AtomicU64>,
    pub cache_hits: Arc<AtomicU64>,
    pub cache_misses: Arc<AtomicU64>,
    pub average_response_time_ms: Arc<AtomicU64>,
}

impl IntelligenceMetrics {
    pub fn new() -> Self {
        Self {
            suggestions_requested: Arc::new(AtomicU64::new(0)),
            suggestions_applied: Arc::new(AtomicU64::new(0)),
            widgets_learned: Arc::new(AtomicU64::new(0)),
            presets_saved: Arc::new(AtomicU64::new(0)),
            cache_hits: Arc::new(AtomicU64::new(0)),
            cache_misses: Arc::new(AtomicU64::new(0)),
            average_response_time_ms: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn record_suggestion_request(&self, response_time: Duration) {
        self.suggestions_requested.fetch_add(1, Ordering::Relaxed);
        
        let current_avg = self.average_response_time_ms.load(Ordering::Relaxed);
        let new_time = response_time.as_millis() as u64;
        let new_avg = (current_avg + new_time) / 2;
        self.average_response_time_ms.store(new_avg, Ordering::Relaxed);
    }

    pub fn record_suggestion_applied(&self) {
        self.suggestions_applied.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_widget_learned(&self) {
        self.widgets_learned.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_preset_saved(&self) {
        self.presets_saved.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_application_rate(&self) -> f64 {
        let requested = self.suggestions_requested.load(Ordering::Relaxed);
        let applied = self.suggestions_applied.load(Ordering::Relaxed);
        
        if requested == 0 {
            0.0
        } else {
            applied as f64 / requested as f64
        }
    }

    pub fn start_periodic_reporting(&self) {
        let metrics = self.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes
            
            loop {
                interval.tick().await;
                
                let report = MetricsReport {
                    timestamp: chrono::Utc::now(),
                    suggestions_requested: metrics.suggestions_requested.load(Ordering::Relaxed),
                    suggestions_applied: metrics.suggestions_applied.load(Ordering::Relaxed),
                    application_rate: metrics.get_application_rate(),
                    widgets_learned: metrics.widgets_learned.load(Ordering::Relaxed),
                    presets_saved: metrics.presets_saved.load(Ordering::Relaxed),
                    average_response_time_ms: metrics.average_response_time_ms.load(Ordering::Relaxed),
                };
                
                log::info!("Intelligence Metrics: {}", serde_json::to_string(&report).unwrap_or_default());
            }
        });
    }
}

#[derive(Debug, serde::Serialize)]
struct MetricsReport {
    timestamp: chrono::DateTime<chrono::Utc>,
    suggestions_requested: u64,
    suggestions_applied: u64,
    application_rate: f64,
    widgets_learned: u64,
    presets_saved: u64,
    average_response_time_ms: u64,
}
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Database Corruption

```rust
// Database recovery utility
pub async fn recover_database(corrupted_db_path: &str, backup_path: &str) -> Result<(), String> {
    // Try to export what we can from the corrupted database
    if let Ok(mut system) = PersistentWidgetSuggestionEngine::new(corrupted_db_path) {
        if let Ok(export_data) = system.export_data() {
            // Save recovered data
            let json_data = serde_json::to_string_pretty(&export_data)
                .map_err(|e| format!("Failed to serialize recovered data: {}", e))?;
            
            tokio::fs::write(backup_path, json_data).await
                .map_err(|e| format!("Failed to write backup: {}", e))?;
            
            log::info!("Recovered {} widgets and {} presets", 
                       export_data.widgets.len(), 
                       export_data.presets.len());
        }
    }
    
    // Create new database and restore from backup if available
    let new_db_path = format!("{}.recovered", corrupted_db_path);
    let mut new_system = PersistentWidgetSuggestionEngine::new(&new_db_path)
        .map_err(|e| format!("Failed to create new database: {:?}", e))?;
    
    if let Ok(backup_data) = tokio::fs::read_to_string(backup_path).await {
        let import_data: ExportData = serde_json::from_str(&backup_data)
            .map_err(|e| format!("Failed to parse backup data: {}", e))?;
        
        new_system.import_data(import_data)
            .map_err(|e| format!("Failed to import backup data: {:?}", e))?;
    }
    
    Ok(())
}
```

#### 2. Performance Optimization

```rust
// Optimized suggestion engine for large datasets
pub struct OptimizedSuggestionEngine {
    engine: WidgetSuggestionEngine,
    suggestion_cache: std::sync::Mutex<lru::LruCache<String, Vec<Suggestion>>>,
    last_cache_clear: std::sync::Mutex<std::time::Instant>,
}

impl OptimizedSuggestionEngine {
    pub fn new(cache_size: usize) -> Self {
        Self {
            engine: WidgetSuggestionEngine::new(),
            suggestion_cache: std::sync::Mutex::new(lru::LruCache::new(cache_size)),
            last_cache_clear: std::sync::Mutex::new(std::time::Instant::now()),
        }
    }
    
    pub fn get_cached_suggestions(&self, partial_widget: &Widget, max_suggestions: usize) -> Vec<Suggestion> {
        let cache_key = format!("{:?}:{}", partial_widget.label, max_suggestions);
        
        // Check cache first
        if let Ok(mut cache) = self.suggestion_cache.lock() {
            if let Some(cached_suggestions) = cache.get(&cache_key) {
                return cached_suggestions.clone();
            }
        }
        
        // Generate new suggestions
        let suggestions = self.engine.get_suggestions(partial_widget, max_suggestions);
        
        // Cache the results
        if let Ok(mut cache) = self.suggestion_cache.lock() {
            cache.put(cache_key, suggestions.clone());
        }
        
        suggestions
    }
    
    pub fn clear_cache_if_stale(&self, max_age: std::time::Duration) {
        if let Ok(mut last_clear) = self.last_cache_clear.lock() {
            if last_clear.elapsed() > max_age {
                if let Ok(mut cache) = self.suggestion_cache.lock() {
                    cache.clear();
                    *last_clear = std::time::Instant::now();
                    log::info!("Cleared stale suggestion cache");
                }
            }
        }
    }
}
```

#### 3. Memory Management

```rust
// Memory-efficient widget storage
pub struct CompactWidgetSuggestionEngine {
    // Use more memory-efficient storage
    widget_summaries: Vec<CompactWidgetRecord>,
    preset_index: std::collections::BTreeMap<String, usize>,
    suggestion_pool: Vec<Suggestion>,
}

#[derive(Debug, Clone)]
struct CompactWidgetRecord {
    id: u32, // Use u32 instead of u64
    label_hash: u64, // Store hash instead of full label
    min_value: f32, // Use f32 for better memory efficiency
    max_value: f32,
    frequency: u16, // Most widgets won't have frequency > 65k
    last_seen: u32, // Store as seconds since epoch (will work until 2106)
    display_type_id: u8, // Map display types to IDs
}

impl CompactWidgetSuggestionEngine {
    pub fn new() -> Self {
        Self {
            widget_summaries: Vec::new(),
            preset_index: std::collections::BTreeMap::new(),
            suggestion_pool: Vec::new(),
        }
    }
    
    // Implement space-efficient algorithms here
}
```

This comprehensive examples guide covers all major usage patterns, from basic library usage to production deployment with monitoring and troubleshooting. Each example is designed to be practical and directly applicable to real-world scenarios.