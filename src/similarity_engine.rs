use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strsim::jaro_winkler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub label: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub is_generated: Option<bool>,
    pub display_type: Option<String>,
    pub current_value: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetValue {
    pub widget_id: String,
    pub label: Option<String>,
    pub value: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    pub description: Option<String>,
    pub widget_values: Vec<WidgetValue>,
    pub created_by: Option<String>,
    pub usage_count: u32,
    pub last_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueStats {
    pub common_values: Vec<f64>,
    pub frequency_map: HashMap<String, u32>,
    pub mean: f64,
    pub std_dev: f64,
    pub percentiles: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetRecord {
    pub id: u64,
    pub widget: Widget,
    pub features: WidgetFeatures,
    pub frequency: u32,
    pub last_seen: u64,
    pub value_stats: Option<ValueStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub widget: Widget,
    pub confidence: f64,
    pub reason: String,
    pub suggested_value: Option<f64>,
    pub value_confidence: f64,
    pub alternative_values: Vec<f64>,
}

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
        let features = self.extract_features(&widget);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let similar_index = self.records.iter().position(|record| {
            self.calculate_similarity(&features, &record.features) > 0.95
        });

        if let Some(idx) = similar_index {
            let record = &mut self.records[idx];
            record.frequency += 1;
            record.last_seen = now;
        } else {
            self.records.push(WidgetRecord {
                id: self.next_id,
                widget,
                features,
                frequency: 1,
                last_seen: now,
                value_stats: None,
            });
            self.next_id += 1;
        }
    }

    pub fn store_preset(&mut self, preset: Preset) {
        if let Some(existing) = self.presets.iter_mut().find(|p| p.name == preset.name) {
            existing.usage_count += 1;
            existing.last_used = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        } else {
            self.presets.push(preset);
        }
        self.recompute_value_statistics();
    }

    pub fn get_suggestions(&self, partial_widget: &Widget, max_suggestions: usize) -> Vec<Suggestion> {
        let partial_features = self.extract_features_partial(partial_widget);
        let mut similarities: Vec<(f64, &WidgetRecord)> = Vec::new();

        for record in &self.records {
            let similarity = self.calculate_similarity(&partial_features, &record.features);
            similarities.push((similarity, record));
        }

        similarities.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        similarities
            .into_iter()
            .take(max_suggestions)
            .filter(|(sim, _)| *sim > 0.1)
            .map(|(similarity, record)| {
                let confidence = similarity * 0.7 + (record.frequency as f64 / 10.0).min(0.3);
                let (suggested_value, value_confidence, alternatives) = 
                    self.suggest_values(partial_widget, &record.features);
                
                Suggestion {
                    widget: record.widget.clone(),
                    confidence,
                    reason: format!("Similarity: {:.2}, Frequency: {}", similarity, record.frequency),
                    suggested_value,
                    value_confidence,
                    alternative_values: alternatives,
                }
            })
            .collect()
    }

    pub fn get_preset_insights(&self, widget: &Widget) -> Option<String> {
        let features = self.extract_features_partial(widget);
        let values = self.extract_value_patterns(&features.label_tokens, &widget.display_type);
        
        if values.is_empty() {
            return None;
        }
        
        let stats = self.compute_value_stats(&values);
        Some(format!(
            "Based on {} presets: Mean={:.2}, StdDev={:.2}, Median={:.2}",
            values.len(),
            stats.mean,
            stats.std_dev,
            stats.percentiles.get(2).unwrap_or(&0.0)
        ))
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("total_widgets".to_string(), self.records.len());
        stats.insert("presets_stored".to_string(), self.presets.len());
        stats.insert("unique_display_types".to_string(), self.display_types.len());
        stats
    }

    fn extract_features(&mut self, widget: &Widget) -> WidgetFeatures {
        let label_tokens = widget.label
            .as_ref()
            .map(|s| self.tokenize_label(s))
            .unwrap_or_default();

        let min_value = widget.minimum.unwrap_or(0.0);
        let max_value = widget.maximum.unwrap_or(1.0);
        let range = max_value - min_value;
        let is_generated = widget.is_generated.unwrap_or(false) as u8 as f64;

        let display_type_hash = widget.display_type
            .as_ref()
            .map(|dt| {
                let hash = self.display_types.len() as u64;
                *self.display_types.entry(dt.clone()).or_insert(hash)
            })
            .unwrap_or(0);

        let value_patterns = self.extract_value_patterns(&label_tokens, &widget.display_type);
        let normalized_position = widget.current_value
            .map(|val| {
                if range > 0.0 {
                    ((val - min_value) / range).clamp(0.0, 1.0)
                } else {
                    0.5
                }
            })
            .unwrap_or(0.5);

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
        let label_tokens = widget.label
            .as_ref()
            .map(|s| self.tokenize_label(s))
            .unwrap_or_default();

        let min_value = widget.minimum.unwrap_or(0.0);
        let max_value = widget.maximum.unwrap_or(1.0);
        let range = max_value - min_value;
        let is_generated = widget.is_generated.unwrap_or(false) as u8 as f64;

        let display_type_hash = widget.display_type
            .as_ref()
            .and_then(|dt| self.display_types.get(dt).copied())
            .unwrap_or(0);

        let value_patterns = self.extract_value_patterns(&label_tokens, &widget.display_type);
        let normalized_position = widget.current_value
            .map(|val| {
                if range > 0.0 {
                    ((val - min_value) / range).clamp(0.0, 1.0)
                } else {
                    0.5
                }
            })
            .unwrap_or(0.5);

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
            .map(|s| s.chars().filter(|c| c.is_alphanumeric()).collect())
            .filter(|s: &String| !s.is_empty())
            .collect()
    }

    fn calculate_similarity(&self, features1: &WidgetFeatures, features2: &WidgetFeatures) -> f64 {
        let mut total_score = 0.0;
        let mut weight_sum = 0.0;

        let label_sim = self.calculate_label_similarity(&features1.label_tokens, &features2.label_tokens);
        total_score += label_sim * 0.4;
        weight_sum += 0.4;

        let range_sim = self.calculate_range_similarity(features1, features2);
        total_score += range_sim * 0.3;
        weight_sum += 0.3;

        let display_sim = if features1.display_type_hash == features2.display_type_hash { 1.0 } else { 0.0 };
        total_score += display_sim * 0.2;
        weight_sum += 0.2;

        let gen_sim = if features1.is_generated == features2.is_generated { 1.0 } else { 0.0 };
        total_score += gen_sim * 0.1;
        weight_sum += 0.1;

        total_score / weight_sum
    }

    fn calculate_label_similarity(&self, tokens1: &[String], tokens2: &[String]) -> f64 {
        if tokens1.is_empty() && tokens2.is_empty() {
            return 1.0;
        }
        if tokens1.is_empty() || tokens2.is_empty() {
            return 0.0;
        }

        let mut max_similarity = 0.0_f64;
        for token1 in tokens1 {
            for token2 in tokens2 {
                let sim = jaro_winkler(token1, token2);
                max_similarity = max_similarity.max(sim);
            }
        }

        let full_label1 = tokens1.join(" ");
        let full_label2 = tokens2.join(" ");
        let full_sim = jaro_winkler(&full_label1, &full_label2);
        
        max_similarity.max(full_sim)
    }

    fn calculate_range_similarity(&self, features1: &WidgetFeatures, features2: &WidgetFeatures) -> f64 {
        let range_diff = (features1.range - features2.range).abs();
        let max_range = features1.range.max(features2.range);
        let range_sim = if max_range > 0.0 {
            1.0 - (range_diff / max_range).min(1.0)
        } else {
            1.0
        };

        let min_diff = (features1.min_value - features2.min_value).abs();
        let max_diff = (features1.max_value - features2.max_value).abs();
        let value_range = (features1.max_value - features1.min_value)
            .max(features2.max_value - features2.min_value);
        
        let value_sim = if value_range > 0.0 {
            1.0 - ((min_diff + max_diff) / (2.0 * value_range)).min(1.0)
        } else {
            1.0
        };

        (range_sim + value_sim) / 2.0
    }

    fn extract_value_patterns(&self, label_tokens: &[String], _display_type: &Option<String>) -> Vec<f64> {
        let mut values = Vec::new();
        
        for preset in &self.presets {
            for widget_value in &preset.widget_values {
                if let Some(label) = &widget_value.label {
                    let other_tokens = self.tokenize_label(label);
                    let label_similarity = self.calculate_label_similarity(label_tokens, &other_tokens);
                    
                    if label_similarity > 0.7 {
                        values.push(widget_value.value);
                    }
                }
            }
        }
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        values.dedup_by(|a, b| (*a - *b).abs() < 0.001);
        values
    }

    fn suggest_values(&self, widget: &Widget, _features: &WidgetFeatures) -> (Option<f64>, f64, Vec<f64>) {
        let values = self.extract_value_patterns(
            &widget.label.as_ref().map(|l| self.tokenize_label(l)).unwrap_or_default(),
            &widget.display_type
        );
        
        if values.is_empty() {
            return (None, 0.0, vec![]);
        }
        
        let suggested_value = values.first().copied();
        let alternatives = values.into_iter().skip(1).take(3).collect();
        
        (suggested_value, 0.8, alternatives)
    }

    fn recompute_value_statistics(&mut self) {
        let mut updates = Vec::new();
        
        for (index, record) in self.records.iter().enumerate() {
            let values = self.extract_value_patterns(&record.features.label_tokens, &record.widget.display_type);
            if !values.is_empty() {
                let stats = self.compute_value_stats(&values);
                updates.push((index, stats));
            }
        }
        
        for (index, stats) in updates {
            self.records[index].value_stats = Some(stats);
        }
    }

    fn compute_value_stats(&self, values: &[f64]) -> ValueStats {
        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mean = sorted_values.iter().sum::<f64>() / sorted_values.len() as f64;
        let variance = sorted_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / sorted_values.len() as f64;
        let std_dev = variance.sqrt();
        
        let percentiles = vec![0.1, 0.25, 0.5, 0.75, 0.9]
            .iter()
            .map(|&p| {
                let idx = (p * (sorted_values.len() - 1) as f64).round() as usize;
                sorted_values[idx.min(sorted_values.len() - 1)]
            })
            .collect();
        
        let mut frequency_map = HashMap::new();
        for &value in &sorted_values {
            let bucket = format!("{:.2}", value);
            *frequency_map.entry(bucket).or_insert(0) += 1;
        }
        
        ValueStats {
            common_values: sorted_values.into_iter().take(10).collect(),
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
mod tests {
    use super::*;

    fn create_test_widget(label: &str, min: f64, max: f64, current: f64, display_type: &str) -> Widget {
        Widget {
            label: Some(label.to_string()),
            minimum: Some(min),
            maximum: Some(max),
            current_value: Some(current),
            is_generated: Some(false),
            display_type: Some(display_type.to_string()),
        }
    }

    #[test]
    fn test_widget_storage_and_retrieval() {
        let mut engine = WidgetSuggestionEngine::new();
        
        let widget = create_test_widget("Master Volume", 0.0, 127.0, 95.0, "slider");
        engine.store_widget(widget);
        
        assert_eq!(engine.records.len(), 1);
        assert_eq!(engine.next_id, 2);
        
        let record = &engine.records[0];
        assert_eq!(record.widget.label, Some("Master Volume".to_string()));
        assert_eq!(record.frequency, 1);
    }

    #[test]
    fn test_duplicate_widget_frequency() {
        let mut engine = WidgetSuggestionEngine::new();
        
        let widget1 = create_test_widget("Volume", 0.0, 100.0, 50.0, "slider");
        let widget2 = create_test_widget("Volume", 0.0, 100.0, 75.0, "slider");
        
        engine.store_widget(widget1);
        engine.store_widget(widget2);
        
        // Should only have one record due to similarity
        assert_eq!(engine.records.len(), 1);
        assert_eq!(engine.records[0].frequency, 2);
    }

    #[test]
    fn test_label_tokenization() {
        let engine = WidgetSuggestionEngine::new();
        
        let tokens = engine.tokenize_label("Master Volume Control");
        assert_eq!(tokens, vec!["master", "volume", "control"]);
        
        let tokens = engine.tokenize_label("LFO-Rate_01");
        assert_eq!(tokens, vec!["lforate01"]);
        
        let tokens = engine.tokenize_label("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_label_similarity_calculation() {
        let engine = WidgetSuggestionEngine::new();
        
        let tokens1 = vec!["master".to_string(), "volume".to_string()];
        let tokens2 = vec!["volume".to_string(), "control".to_string()];
        
        let similarity = engine.calculate_label_similarity(&tokens1, &tokens2);
        assert!(similarity > 0.0);
        assert!(similarity <= 1.0);
        
        // Identical tokens should have high similarity
        let similarity_identical = engine.calculate_label_similarity(&tokens1, &tokens1);
        assert_eq!(similarity_identical, 1.0);
        
        // Empty tokens
        let empty_tokens = vec![];
        let similarity_empty = engine.calculate_label_similarity(&empty_tokens, &empty_tokens);
        assert_eq!(similarity_empty, 1.0);
        
        let similarity_one_empty = engine.calculate_label_similarity(&tokens1, &empty_tokens);
        assert_eq!(similarity_one_empty, 0.0);
    }

    #[test]
    fn test_range_similarity_calculation() {
        let engine = WidgetSuggestionEngine::new();
        
        let features1 = WidgetFeatures {
            label_tokens: vec![],
            min_value: 0.0,
            max_value: 100.0,
            range: 100.0,
            is_generated: 0.0,
            display_type_hash: 0,
            value_patterns: vec![],
            normalized_position: 0.5,
        };
        
        let features2 = WidgetFeatures {
            label_tokens: vec![],
            min_value: 0.0,
            max_value: 100.0,
            range: 100.0,
            is_generated: 0.0,
            display_type_hash: 0,
            value_patterns: vec![],
            normalized_position: 0.5,
        };
        
        let similarity = engine.calculate_range_similarity(&features1, &features2);
        assert_eq!(similarity, 1.0);
        
        // Different ranges
        let features3 = WidgetFeatures {
            range: 50.0,
            max_value: 50.0,
            ..features2.clone()
        };
        
        let similarity_diff = engine.calculate_range_similarity(&features1, &features3);
        assert!(similarity_diff < 1.0);
        assert!(similarity_diff > 0.0);
    }

    #[test]
    fn test_widget_suggestions() {
        let mut engine = WidgetSuggestionEngine::new();
        
        // Store some training widgets
        engine.store_widget(create_test_widget("Master Volume", 0.0, 127.0, 95.0, "slider"));
        engine.store_widget(create_test_widget("Channel Volume", 0.0, 127.0, 80.0, "slider"));
        engine.store_widget(create_test_widget("Bass Level", 0.0, 100.0, 50.0, "knob"));
        
        // Test suggestions for similar widget
        let partial_widget = Widget {
            label: Some("Volume".to_string()),
            minimum: None,
            maximum: None,
            current_value: None,
            is_generated: None,
            display_type: None,
        };
        
        let suggestions = engine.get_suggestions(&partial_widget, 5);
        assert!(!suggestions.is_empty());
        
        // Should suggest widgets with "Volume" in the name with higher confidence
        let volume_suggestions: Vec<_> = suggestions.iter()
            .filter(|s| s.widget.label.as_ref().map_or(false, |l| l.contains("Volume")))
            .collect();
        
        assert!(!volume_suggestions.is_empty());
        
        // All suggestions should have positive confidence
        for suggestion in &suggestions {
            assert!(suggestion.confidence > 0.0);
            assert!(suggestion.confidence <= 1.0);
        }
    }

    #[test]
    fn test_preset_storage_and_statistics() {
        let mut engine = WidgetSuggestionEngine::new();
        
        // Store some widgets first
        engine.store_widget(create_test_widget("Volume", 0.0, 100.0, 75.0, "slider"));
        engine.store_widget(create_test_widget("Pan", -50.0, 50.0, 0.0, "slider"));
        
        let preset = Preset {
            name: "My Setup".to_string(),
            description: Some("Test preset".to_string()),
            widget_values: vec![
                WidgetValue {
                    widget_id: "1".to_string(),
                    label: Some("Volume".to_string()),
                    value: 75.0,
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
        assert_eq!(stats.get("total_widgets"), Some(&2));
        assert_eq!(stats.get("presets_stored"), Some(&1));
        
        // Test duplicate preset (should increment usage count)
        let preset2 = Preset {
            name: "My Setup".to_string(), // Same name
            description: Some("Updated preset".to_string()),
            widget_values: vec![],
            created_by: Some("test_user".to_string()),
            usage_count: 1,
            last_used: 1234567891,
        };
        
        engine.store_preset(preset2);
        
        // Should still have 1 preset but with incremented usage count
        let stats = engine.get_stats();
        assert_eq!(stats.get("presets_stored"), Some(&1));
        assert_eq!(engine.presets[0].usage_count, 2);
    }

    #[test]
    fn test_preset_insights() {
        let mut engine = WidgetSuggestionEngine::new();
        
        // Create preset with volume-related values
        let preset = Preset {
            name: "Audio Setup".to_string(),
            description: None,
            widget_values: vec![
                WidgetValue {
                    widget_id: "1".to_string(),
                    label: Some("Master Volume".to_string()),
                    value: 85.0,
                    confidence: 1.0,
                },
                WidgetValue {
                    widget_id: "2".to_string(),
                    label: Some("Volume Control".to_string()),
                    value: 75.0,
                    confidence: 1.0,
                },
                WidgetValue {
                    widget_id: "3".to_string(),
                    label: Some("Output Volume".to_string()),
                    value: 90.0,
                    confidence: 1.0,
                },
            ],
            created_by: Some("test_user".to_string()),
            usage_count: 1,
            last_used: 1234567890,
        };
        
        engine.store_preset(preset);
        
        let widget = Widget {
            label: Some("Volume".to_string()),
            minimum: Some(0.0),
            maximum: Some(100.0),
            current_value: None,
            is_generated: None,
            display_type: Some("slider".to_string()),
        };
        
        let insights = engine.get_preset_insights(&widget);
        assert!(insights.is_some());
        
        let insights_text = insights.unwrap();
        assert!(insights_text.contains("Mean="));
        assert!(insights_text.contains("StdDev="));
        assert!(insights_text.contains("Median="));
    }

    #[test]
    fn test_value_stats_computation() {
        let engine = WidgetSuggestionEngine::new();
        
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let stats = engine.compute_value_stats(&values);
        
        assert_eq!(stats.mean, 30.0);
        assert_eq!(stats.common_values, values);
        assert_eq!(stats.percentiles.len(), 5);
        assert!(!stats.frequency_map.is_empty());
        
        // Test single value
        let single_value = vec![42.0];
        let stats_single = engine.compute_value_stats(&single_value);
        assert_eq!(stats_single.mean, 42.0);
        assert_eq!(stats_single.std_dev, 0.0);
    }

    #[test]
    fn test_display_type_handling() {
        let mut engine = WidgetSuggestionEngine::new();
        
        // Store widgets with different display types
        engine.store_widget(create_test_widget("Volume 1", 0.0, 100.0, 50.0, "slider"));
        engine.store_widget(create_test_widget("Volume 2", 0.0, 100.0, 60.0, "knob"));
        engine.store_widget(create_test_widget("Volume 3", 0.0, 100.0, 70.0, "slider"));
        
        // Check display types are tracked
        assert_eq!(engine.display_types.len(), 2);
        assert!(engine.display_types.contains_key("slider"));
        assert!(engine.display_types.contains_key("knob"));
        
        // Test suggestions prefer same display type
        let partial_widget = Widget {
            label: Some("Volume".to_string()),
            display_type: Some("slider".to_string()),
            ..Default::default()
        };
        
        let suggestions = engine.get_suggestions(&partial_widget, 5);
        
        // Slider widgets should have higher confidence due to display type match
        let slider_suggestions: Vec<_> = suggestions.iter()
            .filter(|s| s.widget.display_type.as_ref() == Some(&"slider".to_string()))
            .collect();
        
        let knob_suggestions: Vec<_> = suggestions.iter()
            .filter(|s| s.widget.display_type.as_ref() == Some(&"knob".to_string()))
            .collect();
        
        if !slider_suggestions.is_empty() && !knob_suggestions.is_empty() {
            assert!(slider_suggestions[0].confidence >= knob_suggestions[0].confidence);
        }
    }

    #[test]
    fn test_confidence_calculation() {
        let mut engine = WidgetSuggestionEngine::new();
        
        // Store widget with high frequency
        for _ in 0..10 {
            engine.store_widget(create_test_widget("Master Volume", 0.0, 127.0, 95.0, "slider"));
        }
        
        // Store widget with low frequency
        engine.store_widget(create_test_widget("Rare Control", 0.0, 100.0, 50.0, "knob"));
        
        let partial_widget = Widget {
            label: Some("Volume".to_string()),
            ..Default::default()
        };
        
        let suggestions = engine.get_suggestions(&partial_widget, 5);
        
        // The Master Volume should have higher confidence due to higher frequency
        let master_volume_suggestion = suggestions.iter()
            .find(|s| s.widget.label.as_ref().map_or(false, |l| l.contains("Master")));
        
        let rare_control_suggestion = suggestions.iter()
            .find(|s| s.widget.label.as_ref().map_or(false, |l| l.contains("Rare")));
        
        if let (Some(master), Some(rare)) = (master_volume_suggestion, rare_control_suggestion) {
            assert!(master.confidence > rare.confidence);
        }
    }

    #[test]
    fn test_suggestion_filtering() {
        let mut engine = WidgetSuggestionEngine::new();
        
        // Store completely unrelated widget
        engine.store_widget(create_test_widget("Unrelated Control", 0.0, 1.0, 0.5, "button"));
        
        let partial_widget = Widget {
            label: Some("Volume Control".to_string()),
            display_type: Some("slider".to_string()),
            ..Default::default()
        };
        
        let suggestions = engine.get_suggestions(&partial_widget, 5);
        
        // Should filter out suggestions with very low similarity (< 0.1)
        for suggestion in suggestions {
            assert!(suggestion.confidence > 0.1);
        }
    }

    #[test]
    fn test_value_suggestion() {
        let mut engine = WidgetSuggestionEngine::new();
        
        // Create preset with consistent values for a widget type
        let preset = Preset {
            name: "Test Setup".to_string(),
            description: None,
            widget_values: vec![
                WidgetValue {
                    widget_id: "1".to_string(),
                    label: Some("Volume Control".to_string()),
                    value: 80.0,
                    confidence: 1.0,
                },
            ],
            created_by: None,
            usage_count: 1,
            last_used: 1234567890,
        };
        
        engine.store_preset(preset);
        
        let widget = Widget {
            label: Some("Volume Control".to_string()),
            ..Default::default()
        };
        
        let suggestions = engine.get_suggestions(&widget, 5);
        
        // Should have suggestions with actual values
        if !suggestions.is_empty() {
            let suggestion = &suggestions[0];
            assert!(suggestion.suggested_value.is_some());
            assert!(suggestion.value_confidence > 0.0);
        }
    }

    #[test]
    fn test_edge_cases() {
        let mut engine = WidgetSuggestionEngine::new();
        
        // Test with empty/None values
        let empty_widget = Widget {
            label: None,
            minimum: None,
            maximum: None,
            current_value: None,
            is_generated: None,
            display_type: None,
        };
        
        engine.store_widget(empty_widget.clone());
        
        let suggestions = engine.get_suggestions(&empty_widget, 5);
        // Should handle gracefully without panicking
        assert!(suggestions.len() <= 1);
        
        // Test with zero range
        let zero_range_widget = Widget {
            label: Some("Fixed Value".to_string()),
            minimum: Some(50.0),
            maximum: Some(50.0), // Same as minimum
            current_value: Some(50.0),
            is_generated: Some(false),
            display_type: Some("display".to_string()),
        };
        
        engine.store_widget(zero_range_widget);
        // Should not panic or cause issues
        
        // Test with very large numbers
        let large_widget = Widget {
            label: Some("Large Range".to_string()),
            minimum: Some(0.0),
            maximum: Some(f64::MAX / 2.0),
            current_value: Some(1000000.0),
            is_generated: Some(false),
            display_type: Some("slider".to_string()),
        };
        
        engine.store_widget(large_widget);
        // Should handle without overflow
    }

    #[test]
    fn test_widget_features_extraction() {
        let mut engine = WidgetSuggestionEngine::new();
        
        let widget = Widget {
            label: Some("Test Widget Control".to_string()),
            minimum: Some(10.0),
            maximum: Some(90.0),
            current_value: Some(50.0),
            is_generated: Some(true),
            display_type: Some("rotary".to_string()),
        };
        
        let features = engine.extract_features(&widget);
        
        assert_eq!(features.label_tokens, vec!["test", "widget", "control"]);
        assert_eq!(features.min_value, 10.0);
        assert_eq!(features.max_value, 90.0);
        assert_eq!(features.range, 80.0);
        assert_eq!(features.is_generated, 1.0);
        assert_eq!(features.normalized_position, 0.5); // (50-10)/(90-10) = 0.5
        assert!(engine.display_types.contains_key("rotary"));
    }
}