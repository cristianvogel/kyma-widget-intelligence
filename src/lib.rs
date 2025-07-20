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

pub mod kyma_extractor;
pub mod persistence;
pub mod similarity_engine;
pub mod tauri_examples;

// Re-export main types for convenience
pub use similarity_engine::{
    FilteredWidgetDescription, Preset, Suggestion, ValueStats, Widget, WidgetFeatures,
    WidgetRecord, WidgetSuggestionEngine, WidgetValue,
};

pub use persistence::{
    ExportData, PersistentWidgetSuggestionEngine, SledPersistenceError, SledPersistenceManager,
};

pub use kyma_extractor::{KymaWidgetExtractor, WidgetMetadata};

pub use tauri_examples::{
    IntelligenceStats, PresetData, StandaloneIntelligenceService, SuggestionResponse,
    WidgetInsightResponse,
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
            event_id: None,
            values: Vec::new(),
        }
    }
}

/// Initialize the widget intelligence system with a database path
pub fn init_intelligence_system<P: AsRef<std::path::Path>>(
    db_path: P,
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
        event_id: None,
        values: vec![current],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};
    use tempfile::tempdir;

    #[test]
    fn test_persistent_system() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_persistent_lib");

        // Add a small delay to ensure any previous test's file handles are released
        thread::sleep(Duration::from_millis(100));

        let mut system = init_intelligence_system(&db_path)?;

        let widget = create_test_widget("Test Widget", 0.0, 100.0, 50.0);
        system.store_widget(widget)?;

        let stats = system.get_stats();
        assert_eq!(stats.get("total_widgets"), Some(&1));

        // Ensure data is flushed and file handles are released
        system.flush()?;
        drop(system);

        thread::sleep(Duration::from_millis(100));

        let system2 = init_intelligence_system(&db_path)?;
        let stats2 = system2.get_stats();
        assert_eq!(stats2.get("total_widgets"), Some(&1));

        // Clean up
        drop(system2);
        temp_dir.close()?;

        Ok(())
    }
}
