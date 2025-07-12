//! # Widget Intelligence
//!
//! A Rust library for intelligent widget suggestion and learning based on user behavior patterns.
//! 
//! This crate provides functionality for:
//! - Learning from widget usage patterns
//! - Suggesting widget values based on similarity
//! - Persistent storage using Sled database
//! - Integration with Tauri applications
//! - Extracting widget information from Kyma JSON data
//!
//! ## Features
//! 
//! - **Similarity Engine**: Core algorithm for finding similar widgets based on multiple features
//! - **Persistence**: Sled-based storage for long-term learning
//! - **Kyma Integration**: Extract widget data from Kyma JSON format
//! - **Tauri Commands**: Ready-to-use Tauri commands for frontend integration
//!
//! ## Example
//!
//! ```rust
//! use widget_intelligence::{WidgetSuggestionEngine, Widget};
//! 
//! let mut engine = WidgetSuggestionEngine::new();
//! 
//! let widget = Widget {
//!     label: Some("Master Volume".to_string()),
//!     minimum: Some(0.0),
//!     maximum: Some(127.0),
//!     current_value: Some(95.0),
//!     is_generated: Some(false),
//!     display_type: Some("slider".to_string()),
//! };
//! 
//! engine.store_widget(widget);
//! 
//! let suggestions = engine.get_suggestions(&Widget {
//!     label: Some("Volume".to_string()),
//!     ..Default::default()
//! }, 5);
//! ```

pub mod similarity_engine;
pub mod persistence;
pub mod kyma_extractor;
pub mod tauri_examples;

// Re-export main types for convenience
pub use similarity_engine::{
    Widget, WidgetValue, Preset, WidgetSuggestionEngine, 
    Suggestion, WidgetRecord, WidgetFeatures, ValueStats
};

pub use persistence::{
    PersistentWidgetSuggestionEngine, SledPersistenceManager, 
    SledPersistenceError, ExportData
};

pub use kyma_extractor::{KymaWidgetExtractor, WidgetMetadata};

pub use tauri_examples::{
    SuggestionResponse, PresetData, IntelligenceStats, 
    WidgetInsightResponse, StandaloneIntelligenceService
};

impl Default for Widget {
    fn default() -> Self {
        Self {
            label: None,
            minimum: None,
            maximum: None,
            current_value: None,
            is_generated: None,
            display_type: None,
        }
    }
}

/// Initialize the widget intelligence system with a database path
pub fn init_intelligence_system<P: AsRef<std::path::Path>>(
    db_path: P
) -> Result<PersistentWidgetSuggestionEngine, persistence::SledPersistenceError> {
    PersistentWidgetSuggestionEngine::new(db_path)
}

/// Initialize the standalone intelligence service
pub fn init_standalone_service(db_path: &str) -> Result<StandaloneIntelligenceService, String> {
    StandaloneIntelligenceService::new(db_path)
}

/// Utility function to validate widget data
pub fn validate_widget(widget: &Widget) -> Result<(), String> {
    if let (Some(min), Some(max)) = (widget.minimum, widget.maximum) {
        if min >= max {
            return Err("Minimum value must be less than maximum value".to_string());
        }
        
        if let Some(current) = widget.current_value {
            if current < min || current > max {
                return Err("Current value must be within minimum and maximum bounds".to_string());
            }
        }
    }
    
    Ok(())
}

/// Utility function to create a simple widget for testing
pub fn create_test_widget(label: &str, min: f64, max: f64, current: f64) -> Widget {
    Widget {
        label: Some(label.to_string()),
        minimum: Some(min),
        maximum: Some(max),
        current_value: Some(current),
        is_generated: Some(false),
        display_type: Some("slider".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_basic_functionality() {
        let mut engine = WidgetSuggestionEngine::new();
        
        let widget = create_test_widget("Master Volume", 0.0, 127.0, 95.0);
        assert!(validate_widget(&widget).is_ok());
        
        engine.store_widget(widget);
        
        let partial_widget = Widget {
            label: Some("Volume".to_string()),
            ..Default::default()
        };
        
        let suggestions = engine.get_suggestions(&partial_widget, 5);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].confidence > 0.0);
    }

    #[test]
    fn test_persistent_system() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_persistent_lib");
        
        let mut system = init_intelligence_system(&db_path)?;
        
        let widget = create_test_widget("Test Widget", 0.0, 100.0, 50.0);
        system.store_widget(widget)?;
        
        let stats = system.get_stats();
        assert_eq!(stats.get("total_widgets"), Some(&1));
        
        system.flush()?;
        
        let system2 = init_intelligence_system(&db_path)?;
        let stats2 = system2.get_stats();
        assert_eq!(stats2.get("total_widgets"), Some(&1));
        
        Ok(())
    }

    #[test]
    fn test_widget_validation() {
        let valid_widget = create_test_widget("Valid", 0.0, 100.0, 50.0);
        assert!(validate_widget(&valid_widget).is_ok());
        
        let invalid_widget = Widget {
            label: Some("Invalid".to_string()),
            minimum: Some(100.0),
            maximum: Some(0.0), // max < min
            current_value: Some(50.0),
            ..Default::default()
        };
        assert!(validate_widget(&invalid_widget).is_err());
        
        let out_of_bounds_widget = Widget {
            label: Some("Out of bounds".to_string()),
            minimum: Some(0.0),
            maximum: Some(100.0),
            current_value: Some(150.0), // current > max
            ..Default::default()
        };
        assert!(validate_widget(&out_of_bounds_widget).is_err());
    }

    #[test]
    fn test_kyma_extractor() {
        let mut extractor = KymaWidgetExtractor::new();
        
        let kyma_data = serde_json::json!({
            "concreteEventID": 100,
            "label": "Test Widget",
            "minimum": 0.0,
            "maximum": 100.0,
            "displayType": "knob"
        });
        
        let data_map: std::collections::HashMap<String, serde_json::Value> = 
            serde_json::from_value(kyma_data).unwrap();
        
        extractor.cache_widget_description(data_map);
        
        let widget = extractor.create_training_widget(100, 75.0);
        assert!(widget.is_some());
        
        let widget = widget.unwrap();
        assert_eq!(widget.label, Some("Test Widget".to_string()));
        assert_eq!(widget.current_value, Some(75.0));
    }

    #[test]
    fn test_preset_functionality() {
        let mut engine = WidgetSuggestionEngine::new();
        
        let widget1 = create_test_widget("Volume", 0.0, 127.0, 95.0);
        let widget2 = create_test_widget("Pan", -64.0, 64.0, 0.0);
        
        engine.store_widget(widget1);
        engine.store_widget(widget2);
        
        let preset = Preset {
            name: "My Mix".to_string(),
            description: Some("Perfect audio setup".to_string()),
            widget_values: vec![
                WidgetValue {
                    widget_id: "1".to_string(),
                    label: Some("Volume".to_string()),
                    value: 95.0,
                    confidence: 1.0,
                },
                WidgetValue {
                    widget_id: "2".to_string(),
                    label: Some("Pan".to_string()),
                    value: 0.0,
                    confidence: 1.0,
                },
            ],
            created_by: Some("test_user".to_string()),
            usage_count: 1,
            last_used: 1234567890,
        };
        
        engine.store_preset(preset);
        
        let stats = engine.get_stats();
        assert_eq!(stats.get("presets_stored"), Some(&1));
    }
}
