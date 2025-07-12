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
impl Default for WidgetFeatures {
    fn default() -> Self {
        Self {
            label_tokens: Vec::new(),
            min_value: 0.0,
            max_value: 1.0,
            range: 1.0,
            is_generated: 0.0,
            display_type_hash: 0,
            value_patterns: Vec::new(),
            normalized_position: 0.5,
        }
    }
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
    use colored::*;

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

    fn print_separator() {
        println!("\n{}", "=".repeat(80).bright_black());
    }

    #[test]
    fn test_widget_storage_and_retrieval() {
        println!("\n{}", "WIDGET STORAGE TEST".bold().underline());
        
        let mut engine = WidgetSuggestionEngine::new();
        let widget = create_test_widget("Amp_01", 0.0, 1.0, 0.1, "slider");
        
        println!("{} {}", "→".green(), "Storing widget:".yellow());
        println!("{} {}", " ".repeat(4), format!("{:?}", widget).cyan());
        
        engine.store_widget(widget.clone());
        
        let stats = engine.get_stats();
        println!("{} {}", "→".green(), "Engine stats:".yellow());
        println!("{} {}", " ".repeat(4), format!("{:?}", stats).cyan());
        
        assert_eq!(stats.get("total_widgets"), Some(&1));
        println!("{}", "✓ Storage test passed".green());
    }

    #[test]
    fn test_widget_suggestions() {
        println!("\n{}", "WIDGET SUGGESTIONS TEST".bold().underline());
        
        let mut engine = WidgetSuggestionEngine::new();
        
        // Store some test widgets
        let widgets = vec![
            create_test_widget("Amp", 0.0, 1.0, 0.3, "slider"),
            create_test_widget("Frequency", 0.0, 20_000.0, 800.0, "slider"),
            create_test_widget("FreqLow", 0.0, 650.0, 125.0, "slider"),
        ];

        println!("{}", "Training widgets:".yellow());
        for widget in &widgets {
            println!("{} {}", "→".green(), format!("{:?}", widget).cyan());
            engine.store_widget(widget.clone());
        }

        print_separator();
        
        // Test partial widget
        let partial = Widget {
            label: Some("Volume".to_string()),
            ..Default::default()
        };

        println!("{}", "Testing suggestions for:".yellow());
        println!("{} {}", "→".green(), format!("{:?}", partial).cyan());

        let suggestions = engine.get_suggestions(&partial, 5);
        
        println!("\n{}", "Suggestions:".yellow().bold());
        for (i, suggestion) in suggestions.iter().enumerate() {
            println!("{} {} {}", 
                "→".green(),
                format!("#{}", i + 1).yellow(),
                format!("(confidence: {:.2})", suggestion.confidence).cyan()
            );
            println!("{} Widget: {:?}", " ".repeat(4), suggestion.widget);
            if let Some(val) = suggestion.suggested_value {
                println!("{} Suggested value: {}", " ".repeat(4), val);
            }
            println!("{} Reason: {}", " ".repeat(4), suggestion.reason.italic());
        }

        assert!(!suggestions.is_empty());
        println!("{}", "✓ Suggestion test passed".green());
    }

    #[test]
    fn test_label_similarity_calculation() {
        println!("\n{}", "LABEL SIMILARITY TEST".bold().underline());
        
        let engine = WidgetSuggestionEngine::new();
        
        let test_cases = vec![
            ("Output", "Volume", 0.5),
            ("Amp_01", "Amp_02", 0.5),
            ("Gain", "InputGain", 0.5),
        ];

        for (label1, label2, expected_min) in test_cases {
            println!("\n{}", "Testing pair:".yellow());
            println!("{} Label 1: {}", "→".green(), label1.cyan());
            println!("{} Label 2: {}", "→".green(), label2.cyan());

            let tokens1 = engine.tokenize_label(label1);
            let tokens2 = engine.tokenize_label(label2);
            
            let similarity = engine.calculate_label_similarity(&tokens1, &tokens2);
            
            println!("{} Similarity: {:.2}", "→".green(), similarity.to_string().cyan());
            assert!(similarity >= expected_min);
        }
        
        println!("{}", "✓ Label similarity test passed".green());
    }




    #[test]
    fn test_range_similarity_calculation() {
        // Force enable colors for tests
        colored::control::set_override(true);
        println!("\n{}", "RANGE SIMILARITY TEST".bold().underline());

        let engine = WidgetSuggestionEngine::new();

        let test_cases = vec![
            // (range1, range2, expected_min) - with exact calculated values
            ((0.0, 1.0), (-1.0, 1.0), 0.625),    // Unit range vs symmetric range
            ((0.0, 20_000.0), (0.0, 650.0), 0.274), // Large vs small range, same min
            ((0.0, 24.0), (-24.0, 24.0), 0.625),  // One-sided vs symmetric range
        ];


        for ((min1, max1), (min2, max2), expected_min) in test_cases {
            println!("\n{}", "Testing ranges:".yellow());
            println!("{} Range 1: [{:.1}, {:.1}] (span: {:.1})",
                     "→".green(),
                     min1,
                     max1,
                     (max1 - min1)
            );
            println!("{} Range 2: [{:.1}, {:.1}] (span: {:.1})",
                     "→".green(),
                     min2,
                     max2,
                     (max2 - min2)
            );

            let features1 = WidgetFeatures {
                min_value: min1,
                max_value: max1,
                range: max1 - min1,
                ..Default::default()
            };

            let features2 = WidgetFeatures {
                min_value: min2,
                max_value: max2,
                range: max2 - min2,
                ..Default::default()
            };

            let similarity = engine.calculate_range_similarity(&features1, &features2);

            println!("{} Similarity: {:.4}", "→".green(), similarity);

            if similarity >= expected_min {
                println!("{} {}", "✓".green(), "Pass".green());
            } else {
                println!("{} {} (expected >= {:.4}, got {:.4})",
                         "✗".red(),
                         "Failed".red(),
                         expected_min,
                         similarity
                );
            }

            assert!(
                similarity >= expected_min,
                "Similarity {:.4} is less than expected minimum {:.4} for ranges [{:.1},{:.1}] and [{:.1},{:.1}]",
                similarity, expected_min, min1, max1, min2, max2
            );
        }

        println!("\n{}", "✓ All range similarity tests passed".green());
    }
}