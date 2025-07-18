use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use strsim::jaro_winkler;

/// Type alias for filtered widget description from JSON
pub type FilteredWidgetDescription = HashMap<String, serde_json::Value>;

/// Represents a widget with its properties and normalized current value (0.0-1.0 or -1.0-1.0)
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Widget {
    pub label: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub is_generated: Option<bool>,
    pub display_type: Option<String>,
    pub current_value: Option<f64>,
}

/// Represents a widget value with metadata
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct WidgetValue {
    pub widget_id: String,
    pub label: Option<String>,
    pub value: f64,
    pub confidence: f64,
}

/// Represents a preset collection of widget values
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: Option<String>,
    pub widget_values: Vec<WidgetValue>,
    pub created_by: Option<String>,
    pub usage_count: u32,
    pub last_used: u64,
}

/// Features extracted from a widget for similarity calculation
/// value_patterns stores normalized values (0.0-1.0 or -1.0-1.0) from observed widgets
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct WidgetFeatures {
    pub label_tokens: Vec<String>,
    pub min_value: f64,
    pub max_value: f64,
    pub range: f64,
    pub is_generated: f64,
    pub display_type_hash: u64,
    pub value_patterns: Vec<f64>,
    pub normalized_position: f64,
}

impl Default for WidgetFeatures {
    fn default() -> Self {
        Self {
            label_tokens: Vec::new(),
            min_value: 0.0,
            max_value: 100.0,
            range: 100.0,
            is_generated: 0.0,
            display_type_hash: 0,
            value_patterns: Vec::new(),
            normalized_position: 0.0,
        }
    }
}

/// Statistical information about widget values
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ValueStats {
    pub common_values: Vec<f64>,
    pub frequency_map: HashMap<String, u32>,
    pub mean: f64,
    pub std_dev: f64,
    pub percentiles: Vec<f64>,
}

/// A stored widget record with features and usage statistics
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct WidgetRecord {
    pub id: u64,
    pub widget: Widget,
    pub features: WidgetFeatures,
    pub frequency: u32,
    pub last_seen: u64,
    pub value_stats: Option<ValueStats>,
}

impl From<FilteredWidgetDescription> for WidgetRecord {
    fn from(filtered: FilteredWidgetDescription) -> Self {
        // Helper function to extract string values from JSON
        fn extract_string(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<String> {
            map.get(key).and_then(|v| v.as_str().map(|s| s.to_string()))
        }

        // Helper function to extract f64 values from JSON
        fn extract_f64(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<f64> {
            map.get(key).and_then(|v| v.as_f64())
        }

        // Helper function to extract bool values from JSON
        fn extract_bool(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<bool> {
            map.get(key).and_then(|v| v.as_bool())
        }

        // Helper function to extract u64 values from JSON
        fn extract_u64(map: &HashMap<String, serde_json::Value>, key: &str) -> Option<u64> {
            map.get(key).and_then(|v| v.as_u64())
        }

        // Extract widget data from the filtered description
        let widget = Widget {
            label: extract_string(&filtered, "label"),
            minimum: extract_f64(&filtered, "minimum"),
            maximum: extract_f64(&filtered, "maximum"),
            current_value: extract_f64(&filtered, "current_value"),
            is_generated: extract_bool(&filtered, "isGenerated"),
            display_type: extract_string(&filtered, "displayType"),
        };

        // Create basic features from the widget data
        let label_tokens = if let Some(ref label) = widget.label {
            label
                .to_lowercase()
                .split_whitespace()
                .map(|s| s.chars().filter(|c| c.is_alphanumeric()).collect())
                .filter(|s: &String| !s.is_empty())
                .collect()
        } else {
            Vec::new()
        };

        let min_value = widget.minimum.unwrap_or(0.0);
        let max_value = widget.maximum.unwrap_or(100.0);
        let range = max_value - min_value;

        // Calculate display type hash
        let display_type_hash = if let Some(ref display_type) = widget.display_type {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(display_type, &mut hasher);
            std::hash::Hasher::finish(&hasher)
        } else {
            0
        };

        let features = WidgetFeatures {
            label_tokens,
            min_value,
            max_value,
            range,
            is_generated: if widget.is_generated.unwrap_or(false) {
                1.0
            } else {
                0.0
            },
            display_type_hash,
            value_patterns: if let Some(current) = widget.current_value {
                vec![current]
            } else {
                Vec::new()
            },
            normalized_position: widget.current_value.unwrap_or(0.5)
        };

        // Get current timestamp
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0))
            .as_secs();

        // Extract ID from concreteEventID if available, otherwise use 0
        let id = extract_u64(&filtered, "concreteEventID").unwrap_or(0);

        WidgetRecord {
            id,
            widget,
            features,
            frequency: 1,
            last_seen: current_time,
            value_stats: None,
        }
    }
}

/// A suggestion for a widget value with confidence and reasoning
/// All suggested values are normalized (0.0-1.0 or -1.0-1.0)
#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct Suggestion {
    pub widget: Widget,
    pub confidence: f64,
    pub reason: String,
    pub suggested_value: Option<f64>,
    pub value_confidence: f64,
    pub alternative_values: Vec<f64>,
}

/// The main engine for widget suggestions and learning
pub struct WidgetSuggestionEngine {
    pub records: Vec<WidgetRecord>,
    pub presets: Vec<Preset>,
    pub display_types: HashMap<String, u64>,
    pub next_id: u64,
}

impl WidgetSuggestionEngine {
    pub fn new() -> Self {
        Self {
            records: Vec::new(),
            presets: Vec::new(),
            display_types: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn store_widget(&mut self, widget: Widget) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Extract features
        let features = self.extract_features(&widget);

        // Check if a similar widget already exists
        let mut found_similar = false;

        for i in 0..self.records.len() {
            let similarity = self.calculate_similarity(&features, &self.records[i].features);

            if similarity > 0.85 {
                self.records[i].frequency += 1;
                self.records[i].last_seen = current_time;

                // Update widget if new one has more complete information
                if widget.label.is_some() && self.records[i].widget.label.is_none() {
                    self.records[i].widget.label = widget.label.clone();
                }
                if widget.display_type.is_some() && self.records[i].widget.display_type.is_none() {
                    self.records[i].widget.display_type = widget.display_type.clone();
                }
                if let Some(current) = widget.current_value {
                    self.records[i].widget.current_value = Some(current);
                    // Add normalized value to the feature's value_patterns
                    self.records[i].features.value_patterns.push(current);
                }

                found_similar = true;
                break;
            }
        }

        if !found_similar {
            let record = WidgetRecord {
                id: self.next_id,
                widget,
                features,
                frequency: 1,
                last_seen: current_time,
                value_stats: None,
            };
            self.records.push(record);
            self.next_id += 1;
        }

        // Recompute statistics periodically
        if self.records.len() % 10 == 0 {
            self.recompute_value_statistics();
        }
    }

    pub fn store_preset(&mut self, preset: Preset) {
        // Store or update preset
        if let Some(existing) = self.presets.iter_mut().find(|p| p.name == preset.name) {
            existing.usage_count += 1;
            existing.last_used = preset.last_used;
            existing.widget_values = preset.widget_values;
            existing.description = preset.description;
        } else {
            self.presets.push(preset);
        }
    }

    pub fn get_suggestions(
        &self,
        partial_widget: &Widget,
        max_suggestions: usize,
    ) -> Vec<Suggestion> {
        let features = self.extract_features_partial(partial_widget);
        let mut suggestions = Vec::new();

        for record in &self.records {
            let similarity = self.calculate_similarity(&features, &record.features);

            if similarity > 0.3 {
                let (suggested_value, value_confidence, alternative_values) =
                    self.suggest_values(partial_widget, &record.features);

                let reason = format!(
                    "Similar to {} (similarity: {:.2}, frequency: {})",
                    record.widget.label.as_deref().unwrap_or("unnamed widget"),
                    similarity,
                    record.frequency
                );

                suggestions.push(Suggestion {
                    widget: record.widget.clone(),
                    confidence: similarity,
                    reason,
                    suggested_value,
                    value_confidence,
                    alternative_values,
                });
            }
        }

        suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());
        suggestions.truncate(max_suggestions);
        suggestions
    }

    pub fn get_preset_insights(&self, widget: &Widget) -> Option<String> {
        for preset in &self.presets {
            for widget_value in &preset.widget_values {
                if let Some(label) = &widget.label {
                    if let Some(preset_label) = &widget_value.label {
                        if jaro_winkler(label, preset_label) > 0.8 {
                            return Some(format!(
                                "This widget is often set to {} in the '{}' preset",
                                widget_value.value, preset.name
                            ));
                        }
                    }
                }
            }
        }
        None
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_widgets".to_string(), self.records.len());
        stats.insert("total_presets".to_string(), self.presets.len());
        stats.insert("display_types".to_string(), self.display_types.len());
        stats
    }

    fn extract_features(&mut self, widget: &Widget) -> WidgetFeatures {
        let label_tokens = if let Some(label) = &widget.label {
            self.tokenize_label(label)
        } else {
            Vec::new()
        };

        let min_value = widget.minimum.unwrap_or(0.0);
        let max_value = widget.maximum.unwrap_or(100.0);
        let range = max_value - min_value;

        let display_type_hash = if let Some(display_type) = &widget.display_type {
            let mut hasher = DefaultHasher::new();
            display_type.hash(&mut hasher);
            let hash = hasher.finish();

            // Store display type for future reference
            self.display_types.insert(display_type.clone(), hash);
            hash
        } else {
            0
        };

        let is_generated = if widget.is_generated.unwrap_or(false) {
            1.0
        } else {
            0.0
        };

        let mut value_patterns = self.extract_value_patterns(&label_tokens, &widget.display_type);
        
        // Add the normalized current_value to value_patterns if available
        if let Some(current) = widget.current_value {
            value_patterns.push(current);
        }

        // current_value is already normalized, use it directly
        let normalized_position = widget.current_value.unwrap_or(0.5);

        WidgetFeatures {
            label_tokens,
            min_value,
            max_value,
            range,
            is_generated,
            display_type_hash,
            value_patterns,
            normalized_position,
        }
    }

    fn extract_features_partial(&self, widget: &Widget) -> WidgetFeatures {
        let label_tokens = if let Some(label) = &widget.label {
            self.tokenize_label(label)
        } else {
            Vec::new()
        };

        let min_value = widget.minimum.unwrap_or(0.0);
        let max_value = widget.maximum.unwrap_or(100.0);
        let range = max_value - min_value;

        let display_type_hash = if let Some(display_type) = &widget.display_type {
            let mut hasher = DefaultHasher::new();
            display_type.hash(&mut hasher);
            hasher.finish()
        } else {
            0
        };

        let is_generated = if widget.is_generated.unwrap_or(false) {
            1.0
        } else {
            0.0
        };

        let mut value_patterns = self.extract_value_patterns(&label_tokens, &widget.display_type);
        
        // Add the normalized current_value to value_patterns if available
        if let Some(current) = widget.current_value {
            value_patterns.push(current);
        }

        // current_value is already normalized, use it directly
        let normalized_position = widget.current_value.unwrap_or(0.5);

        WidgetFeatures {
            label_tokens,
            min_value,
            max_value,
            range,
            is_generated,
            display_type_hash,
            value_patterns,
            normalized_position,
        }
    }

    fn tokenize_label(&self, label: &str) -> Vec<String> {
        label
            .to_lowercase()
            .split_whitespace()
            .filter(|word| !word.is_empty())
            .map(|word| word.to_string())
            .collect()
    }

    fn calculate_similarity(&self, features1: &WidgetFeatures, features2: &WidgetFeatures) -> f64 {
        let label_similarity =
            self.calculate_label_similarity(&features1.label_tokens, &features2.label_tokens);
        let range_similarity = self.calculate_range_similarity(features1, features2);
        let display_type_similarity = if features1.display_type_hash == features2.display_type_hash
            && features1.display_type_hash != 0
        {
            1.0
        } else {
            0.0
        };
        let generated_similarity = 1.0 - (features1.is_generated - features2.is_generated).abs();

        // Weighted combination
        let similarity = (label_similarity * 0.4)
            + (range_similarity * 0.3)
            + (display_type_similarity * 0.2)
            + (generated_similarity * 0.1);

        similarity.clamp(0.0, 1.0)
    }

    fn calculate_label_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        if tokens1.is_empty() || tokens2.is_empty() {
            return if tokens1.is_empty() && tokens2.is_empty() {
                1.0
            } else {
                0.0
            };
        }

        let mut total_similarity = 0.0;
        let mut matches = 0;

        for token1 in tokens1 {
            let mut best_match = 0.0;
            for token2 in tokens2 {
                let similarity = jaro_winkler(token1, token2);
                if similarity > best_match {
                    best_match = similarity;
                }
            }
            if best_match > 0.7 {
                total_similarity += best_match;
                matches += 1;
            }
        }

        if matches > 0 {
            total_similarity / matches as f64
        } else {
            0.0
        }
    }

    fn calculate_range_similarity(
        &self,
        features1: &WidgetFeatures,
        features2: &WidgetFeatures,
    ) -> f64 {
        let min_diff = (features1.min_value - features2.min_value).abs();
        let max_diff = (features1.max_value - features2.max_value).abs();
        let range_diff = (features1.range - features2.range).abs();

        let max_range = features1.range.max(features2.range);
        if max_range == 0.0 {
            return 1.0;
        }

        let normalized_diff = (min_diff + max_diff + range_diff) / (3.0 * max_range);
        1.0 - normalized_diff.min(1.0)
    }

    fn extract_value_patterns(
        &self,
        label_tokens: &[String],
        _display_type: &Option<String>,
    ) -> Vec<f64> {
        let mut patterns = Vec::new();

        // Common value patterns based on label tokens
        for token in label_tokens {
            match token.as_str() {
                "volume" | "level" | "gain" => patterns.push(0.75),
                "bass" | "low" => patterns.push(0.6),
                "treble" | "high" => patterns.push(0.7),
                "mid" | "middle" => patterns.push(0.5),
                "pan" => patterns.push(0.5),
                "reverb" | "delay" => patterns.push(0.3),
                _ => {}
            }
        }

        if patterns.is_empty() {
            patterns.push(0.5); // Default middle value
        }

        patterns
    }

    fn suggest_values(
        &self,
        _widget: &Widget,
        features: &WidgetFeatures,
    ) -> (Option<f64>, f64, Vec<f64>) {
        // Return normalized values directly - consumer will handle denormalization
        let mut suggested_values: Vec<f64> = if !features.value_patterns.is_empty() {
            // Use the accumulated normalized values directly
            features.value_patterns.clone()
        } else {
            // Fallback to reasonable normalized defaults if no patterns available
            vec![0.5, 0.3, 0.7]  // Middle, lower third, upper third
        };

        // Sort and remove duplicates
        suggested_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        suggested_values.dedup();

        // Calculate confidence based on number of observed patterns
        let confidence = match features.value_patterns.len() {
            0 => 0.3,
            1..=2 => 0.5,
            3..=5 => 0.7,
            _ => 0.9,
        };

        let primary_suggestion = suggested_values.first().copied();

        (primary_suggestion, confidence, suggested_values)
    }

    fn recompute_value_statistics(&mut self) {
        for i in 0..self.records.len() {
            let values: Vec<f64> = self
                .records
                .iter()
                .filter_map(|r| r.widget.current_value)
                .collect();

            if !values.is_empty() {
                self.records[i].value_stats = Some(self.compute_value_stats(&values));
            }
        }
    }

    fn compute_value_stats(&self, values: &[f64]) -> ValueStats {
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mean = sorted_values.iter().sum::<f64>() / sorted_values.len() as f64;

        let variance = sorted_values
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>()
            / sorted_values.len() as f64;
        let std_dev = variance.sqrt();

        let percentiles = vec![
            sorted_values[(sorted_values.len() * 25 / 100).min(sorted_values.len() - 1)],
            sorted_values[(sorted_values.len() * 50 / 100).min(sorted_values.len() - 1)],
            sorted_values[(sorted_values.len() * 75 / 100).min(sorted_values.len() - 1)],
        ];

        // Find most common values
        let mut frequency_map = HashMap::new();
        for &value in values {
            let key = format!("{value:.2}");
            *frequency_map.entry(key).or_insert(0) += 1;
        }

        let mut common_values: Vec<f64> = frequency_map
            .iter()
            .filter(|(_, &count)| count > 1)
            .map(|(key, _)| key.parse::<f64>().unwrap_or(0.0))
            .collect();
        common_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        ValueStats {
            common_values,
            frequency_map,
            mean,
            std_dev,
            percentiles,
        }
    }
}

impl Default for WidgetSuggestionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod conversion_tests {
    use super::*;

    #[test]
    fn test_filtered_widget_description_conversion() {
        let mut filtered = FilteredWidgetDescription::new();
        filtered.insert(
            "concreteEventID".to_string(),
            serde_json::Value::Number(serde_json::Number::from(42)),
        );
        filtered.insert(
            "label".to_string(),
            serde_json::Value::String("Master Volume".to_string()),
        );
        filtered.insert(
            "minimum".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
        );
        filtered.insert(
            "maximum".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(127.0).unwrap()),
        );
        filtered.insert(
            "current_value".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.75).unwrap()),
        );
        filtered.insert(
            "displayType".to_string(),
            serde_json::Value::String("slider".to_string()),
        );
        filtered.insert("isGenerated".to_string(), serde_json::Value::Bool(false));

        // Test the conversion using the From trait - this is the idiomatic way
        let widget_record: WidgetRecord = filtered.into();

        assert_eq!(widget_record.id, 42);
        assert_eq!(
            widget_record.widget.label,
            Some("Master Volume".to_string())
        );
        assert_eq!(widget_record.widget.minimum, Some(0.0));
        assert_eq!(widget_record.widget.maximum, Some(127.0));
        assert_eq!(
            widget_record.widget.display_type,
            Some("slider".to_string())
        );
        assert_eq!(widget_record.widget.is_generated, Some(false));
        assert_eq!(widget_record.frequency, 1);
        assert_eq!(
            widget_record.features.label_tokens,
            vec!["master", "volume"]
        );
        assert_eq!(widget_record.features.min_value, 0.0);
        assert_eq!(widget_record.features.max_value, 127.0);
        assert_eq!(widget_record.features.range, 127.0);
        assert_eq!(widget_record.features.is_generated, 0.0);
        assert!(widget_record.features.display_type_hash != 0);
        assert_eq!(widget_record.widget.current_value, Some(0.75));
        // Check that current_value is added to value_patterns
        assert!(widget_record.features.value_patterns.contains(&0.75));
    }

    #[test]
    fn test_conversion_with_missing_fields() {
        // Test conversion when some fields are missing
        let mut filtered = FilteredWidgetDescription::new();
        filtered.insert(
            "concreteEventID".to_string(),
            serde_json::Value::Number(serde_json::Number::from(100)),
        );
        filtered.insert(
            "label".to_string(),
            serde_json::Value::String("Test Control".to_string()),
        );
        // Note: missing minimum, maximum, displayType, isGenerated

        let widget_record: WidgetRecord = filtered.into();

        assert_eq!(widget_record.id, 100);
        assert_eq!(widget_record.widget.label, Some("Test Control".to_string()));
        assert_eq!(widget_record.widget.minimum, None);
        assert_eq!(widget_record.widget.maximum, None);
        assert_eq!(widget_record.widget.display_type, None);
        assert_eq!(widget_record.widget.is_generated, None);
        assert_eq!(widget_record.features.min_value, 0.0); // Default value
        assert_eq!(widget_record.features.max_value, 100.0); // Default value
        assert_eq!(widget_record.features.range, 100.0);
        assert_eq!(widget_record.features.display_type_hash, 0);
    }

    #[test]
    fn test_normalized_value_accumulation() {
        let mut engine = WidgetSuggestionEngine::new();

        // Create a widget with normalized value
        let widget1 = Widget {
            label: Some("Volume".to_string()),
            minimum: Some(0.0),
            maximum: Some(100.0),
            current_value: Some(0.7), // Normalized value
            is_generated: Some(false),
            display_type: Some("slider".to_string()),
        };

        // Store first widget
        engine.store_widget(widget1.clone());

        // Store similar widget with different normalized value
        let mut widget2 = widget1.clone();
        widget2.current_value = Some(0.8);
        engine.store_widget(widget2);

        // Store another similar widget
        let mut widget3 = widget1.clone();
        widget3.current_value = Some(0.75);
        engine.store_widget(widget3);

        // Check that values are accumulated in value_patterns
        assert_eq!(engine.records.len(), 1); // Should be merged as similar
        assert_eq!(engine.records[0].frequency, 3);
        
        let patterns = &engine.records[0].features.value_patterns;
        assert!(patterns.contains(&0.7));
        assert!(patterns.contains(&0.8));
        assert!(patterns.contains(&0.75));
        // Also contains default pattern from extract_value_patterns
        assert!(patterns.len() >= 3);
    }
}
