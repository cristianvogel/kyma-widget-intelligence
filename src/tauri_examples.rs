
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Response types - copy these to your Tauri app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionResponse {
    pub suggested_value: Option<f64>,
    pub confidence: f64,
    pub alternative_values: Vec<f64>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetData {
    pub name: String,
    pub description: Option<String>,
    pub widget_values: HashMap<String, f64>,
    pub created_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceStats {
    pub total_widgets: usize,
    pub total_presets: usize,
    pub last_updated: String,
    pub cache_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetInsightResponse {
    pub insights: Option<String>,
    pub suggested_values: Vec<f64>,
    pub confidence_scores: Vec<f64>,
}

// Example Tauri commands - copy these to your Tauri application:

#[cfg(doc)]
mod example_commands {
    use super::*;
    use crate::{Widget, WidgetValue, Preset, PersistentWidgetSuggestionEngine, KymaWidgetExtractor};
    use std::sync::Mutex;

    pub struct IntelligenceState {
        pub system: Mutex<PersistentWidgetSuggestionEngine>,
        pub extractor: Mutex<KymaWidgetExtractor>,
    }

    //noinspection ALL
    //noinspection GrazieInspection
    //noinspection ALL
    /// Example: Cache widget description from Kyma JSON
    /// Copy this function to your Tauri app and uncomment the #[tauri::command] attribute
    // #[tauri::command]
    pub async fn cache_widget_description(
        state: tauri::State<'_, IntelligenceState>,
        event_id: i64,
        kyma_json: String,
    ) -> Result<(), String> {
        let kyma_data: HashMap<String, serde_json::Value> = serde_json::from_str(&kyma_json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        crate::kyma_extractor::KymaWidgetExtractor::validate_kyma_data(&kyma_data)
            .map_err(|e| format!("Invalid Kyma data: {}", e))?;
        
        let mut extractor = state.extractor.lock()
            .map_err(|_| "Failed to lock extractor")?;
        
        extractor.cache_widget_description(kyma_data);
        log::debug!("Cached widget description for event ID: {}", event_id);
        Ok(())
    }

    /// Example: Save preset and learn from widget values
    /// Copy this function to your Tauri app and uncomment the #[tauri::command] attribute
    // #[tauri::command]
    pub async fn save_preset_and_learn(
        state: tauri::State<'_, IntelligenceState>,
        preset_data: PresetData,
    ) -> Result<IntelligenceStats, String> {
        let mut system = state.system.lock()
            .map_err(|_| "Failed to lock intelligence system")?;
        
        let extractor = state.extractor.lock()
            .map_err(|_| "Failed to lock extractor")?;
        
        let event_values: HashMap<i64, f64> = preset_data.widget_values
            .into_iter()
            .filter_map(|(k, v)| k.parse::<i64>().ok().map(|id| (id, v)))
            .collect();
        
        let mut widget_values = Vec::new();
        for (event_id, current_value) in &event_values {
            if let Some(training_widget) = extractor.create_training_widget(*event_id, *current_value) {
                system.store_widget(training_widget.clone())
                    .map_err(|e| format!("Failed to store widget: {:?}", e))?;
                
                widget_values.push(WidgetValue {
                    widget_id: event_id.to_string(),
                    label: training_widget.label,
                    value: *current_value,
                    confidence: 1.0,
                });
            }
        }
        
        let preset = Preset {
            name: preset_data.name,
            description: preset_data.description,
            widget_values,
            created_by: preset_data.created_by,
            usage_count: 1,
            last_used: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        system.store_preset(preset)
            .map_err(|e| format!("Failed to store preset: {:?}", e))?;
        
        let stats = system.get_stats();
        Ok(IntelligenceStats {
            total_widgets: stats.get("total_widgets").copied().unwrap_or(0),
            total_presets: stats.get("presets_stored").copied().unwrap_or(0),
            last_updated: chrono::Utc::now().to_rfc3339(),
            cache_size: extractor.cache_size(),
        })
    }

    /// Example: Get widget value suggestions
    /// Copy this function to your Tauri app and uncomment the #[tauri::command] attribute
    // #[tauri::command]
    pub async fn get_widget_value_suggestions(
        state: tauri::State<'_, IntelligenceState>,
        event_id: i64,
        partial_label: Option<String>,
        display_type: Option<String>,
    ) -> Result<Vec<SuggestionResponse>, String> {
        let system = state.system.lock()
            .map_err(|_| "Failed to lock intelligence system")?;
        
        let partial_widget = Widget {
            label: partial_label,
            minimum: None,
            maximum: None,
            current_value: None,
            is_generated: None,
            display_type,
        };
        
        let suggestions = system.get_suggestions(&partial_widget, 5);
        
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

    /// Example: Get widget insights
    /// Copy this function to your Tauri app and uncomment the #[tauri::command] attribute
    // #[tauri::command]
    pub async fn get_widget_insights(
        state: tauri::State<'_, IntelligenceState>,
        event_id: i64,
        partial_label: Option<String>,
        display_type: Option<String>,
    ) -> Result<WidgetInsightResponse, String> {
        let system = state.system.lock()
            .map_err(|_| "Failed to lock intelligence system")?;
        
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
        
        Ok(WidgetInsightResponse {
            insights,
            suggested_values,
            confidence_scores,
        })
    }

    /// Example: Get intelligence statistics
    /// Copy this function to your Tauri app and uncomment the #[tauri::command] attribute
    // #[tauri::command]
    pub async fn get_intelligence_stats(
        state: tauri::State<'_, IntelligenceState>,
    ) -> Result<IntelligenceStats, String> {
        let system = state.system.lock()
            .map_err(|_| "Failed to lock intelligence system")?;
        
        let extractor = state.extractor.lock()
            .map_err(|_| "Failed to lock extractor")?;
        
        let stats = system.get_stats();
        Ok(IntelligenceStats {
            total_widgets: stats.get("total_widgets").copied().unwrap_or(0),
            total_presets: stats.get("presets_stored").copied().unwrap_or(0),
            last_updated: chrono::Utc::now().to_rfc3339(),
            cache_size: extractor.cache_size(),
        })
    }
}

/// Standalone service for non-Tauri applications
/// 
/// This provides the same functionality as the Tauri commands but without Tauri dependencies.
/// Use this if you want to integrate the intelligence system into other types of applications.
pub struct StandaloneIntelligenceService {
    system: std::sync::Mutex<crate::PersistentWidgetSuggestionEngine>,
    extractor: std::sync::Mutex<crate::KymaWidgetExtractor>,
}

impl StandaloneIntelligenceService {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let system = crate::PersistentWidgetSuggestionEngine::new(db_path)
            .map_err(|e| format!("Failed to initialize intelligence system: {:?}", e))?;
        
        let extractor = crate::KymaWidgetExtractor::new();
        
        Ok(Self {
            system: std::sync::Mutex::new(system),
            extractor: std::sync::Mutex::new(extractor),
        })
    }
    
    pub async fn cache_widget_description(
        &self,
        event_id: i64,
        kyma_json: String,
    ) -> Result<(), String> {
        let kyma_data: HashMap<String, serde_json::Value> = serde_json::from_str(&kyma_json)
            .map_err(|e| format!("Failed to parse JSON: {}", e))?;
        
        crate::kyma_extractor::KymaWidgetExtractor::validate_kyma_data(&kyma_data)
            .map_err(|e| format!("Invalid Kyma data: {}", e))?;
        
        let mut extractor = self.extractor.lock()
            .map_err(|_| "Failed to lock extractor")?;
        
        extractor.cache_widget_description(kyma_data);
        log::debug!("Cached widget description for event ID: {}", event_id);
        Ok(())
    }
    
    pub async fn save_preset_and_learn(
        &self,
        preset_data: PresetData,
    ) -> Result<IntelligenceStats, String> {
        let mut system = self.system.lock()
            .map_err(|_| "Failed to lock intelligence system")?;
        
        let extractor = self.extractor.lock()
            .map_err(|_| "Failed to lock extractor")?;
        
        let event_values: HashMap<i64, f64> = preset_data.widget_values
            .into_iter()
            .filter_map(|(k, v)| k.parse::<i64>().ok().map(|id| (id, v)))
            .collect();
        
        let mut widget_values = Vec::new();
        for (event_id, current_value) in &event_values {
            if let Some(training_widget) = extractor.create_training_widget(*event_id, *current_value) {
                system.store_widget(training_widget.clone())
                    .map_err(|e| format!("Failed to store widget: {:?}", e))?;
                
                widget_values.push(crate::WidgetValue {
                    widget_id: event_id.to_string(),
                    label: training_widget.label,
                    value: *current_value,
                    confidence: 1.0,
                });
            }
        }
        
        let preset = crate::Preset {
            name: preset_data.name,
            description: preset_data.description,
            widget_values,
            created_by: preset_data.created_by,
            usage_count: 1,
            last_used: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        system.store_preset(preset)
            .map_err(|e| format!("Failed to store preset: {:?}", e))?;
        
        let stats = system.get_stats();
        Ok(IntelligenceStats {
            total_widgets: stats.get("total_widgets").copied().unwrap_or(0),
            total_presets: stats.get("presets_stored").copied().unwrap_or(0),
            last_updated: chrono::Utc::now().to_rfc3339(),
            cache_size: extractor.cache_size(),
        })
    }
    
    pub async fn get_widget_value_suggestions(
        &self,
        event_id: i64,
        partial_label: Option<String>,
        display_type: Option<String>,
    ) -> Result<Vec<SuggestionResponse>, String> {
        let system = self.system.lock()
            .map_err(|_| "Failed to lock intelligence system")?;
        
        let partial_widget = crate::Widget {
            label: partial_label,
            minimum: None,
            maximum: None,
            current_value: None,
            is_generated: None,
            display_type,
        };
        
        let suggestions = system.get_suggestions(&partial_widget, 5);
        
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
    
    pub async fn get_intelligence_stats(&self) -> Result<IntelligenceStats, String> {
        let system = self.system.lock()
            .map_err(|_| "Failed to lock intelligence system")?;
        
        let extractor = self.extractor.lock()
            .map_err(|_| "Failed to lock extractor")?;
        
        let stats = system.get_stats();
        Ok(IntelligenceStats {
            total_widgets: stats.get("total_widgets").copied().unwrap_or(0),
            total_presets: stats.get("presets_stored").copied().unwrap_or(0),
            last_updated: chrono::Utc::now().to_rfc3339(),
            cache_size: extractor.cache_size(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_standalone_service() {
        let temp_dir = tempdir().unwrap();
        let db_path_buf = temp_dir.path().join("test_standalone");
        let db_path = db_path_buf.to_str().unwrap();
        
        let service = StandaloneIntelligenceService::new(db_path).unwrap();
        
        let kyma_json = r#"{"concreteEventID": 100, "label": "Master Volume", "minimum": 0.0, "maximum": 127.0, "displayType": "slider"}"#;
        
        service.cache_widget_description(100, kyma_json.to_string()).await.unwrap();
        
        let mut widget_values = HashMap::new();
        widget_values.insert("100".to_string(), 95.0);
        
        let preset_data = PresetData {
            name: "My Mix".to_string(),
            description: Some("Perfect audio setup".to_string()),
            widget_values,
            created_by: Some("test_user".to_string()),
        };
        
        let stats = service.save_preset_and_learn(preset_data).await.unwrap();
        assert_eq!(stats.total_widgets, 1);
        assert_eq!(stats.total_presets, 1);
        
        let suggestions = service.get_widget_value_suggestions(
            100,
            Some("Volume".to_string()),
            Some("slider".to_string()),
        ).await.unwrap();
        
        assert!(!suggestions.is_empty());
    }
}