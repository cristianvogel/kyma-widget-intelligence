use serde_json::Value;
use std::collections::HashMap;
use crate::similarity_engine::Widget;

pub struct KymaWidgetExtractor {
    widget_descriptions: HashMap<i64, HashMap<String, Value>>,
}

impl KymaWidgetExtractor {
    pub fn new() -> Self {
        Self {
            widget_descriptions: HashMap::new(),
        }
    }

    pub fn cache_widget_description(&mut self, kyma_data: HashMap<String, Value>) {
        if let Some(Value::Number(event_id)) = kyma_data.get("concreteEventID") {
            if let Some(id) = event_id.as_i64() {
                log::trace!("Caching widget description for event ID: {}", id);
                self.widget_descriptions.insert(id, kyma_data);
            }
        }
    }

    pub fn create_training_widget(&self, event_id: i64, current_value: f64) -> Option<Widget> {
        let kyma_data = self.widget_descriptions.get(&event_id)?;
        
        let widget = Widget {
            label: self.extract_label(kyma_data),
            minimum: self.extract_float_field(kyma_data, "minimum"),
            maximum: self.extract_float_field(kyma_data, "maximum"),
            current_value: Some(current_value),
            is_generated: self.extract_bool_field(kyma_data, "isGenerated"),
            display_type: self.extract_display_type(kyma_data),
        };

        log::trace!("Created training widget for event ID {}: {:?}", event_id, widget.label);
        Some(widget)
    }

    pub fn get_cached_description(&self, event_id: i64) -> Option<&HashMap<String, Value>> {
        self.widget_descriptions.get(&event_id)
    }

    pub fn get_cached_event_ids(&self) -> Vec<i64> {
        self.widget_descriptions.keys().copied().collect()
    }

    pub fn clear_cache(&mut self) {
        self.widget_descriptions.clear();
    }

    pub fn cache_size(&self) -> usize {
        self.widget_descriptions.len()
    }

    pub fn extract_all_widgets_with_values(&self, values: &HashMap<i64, f64>) -> Vec<Widget> {
        let mut widgets = Vec::new();
        
        for (&event_id, &value) in values {
            if let Some(widget) = self.create_training_widget(event_id, value) {
                widgets.push(widget);
            }
        }
        
        widgets
    }

    fn extract_label(&self, data: &HashMap<String, Value>) -> Option<String> {
        if let Some(Value::String(label)) = data.get("label") {
            if !label.is_empty() {
                return Some(label.clone());
            }
        }
        
        if let Some(Value::String(name)) = data.get("name") {
            if !name.is_empty() {
                return Some(name.clone());
            }
        }
        
        if let Some(Value::String(title)) = data.get("title") {
            if !title.is_empty() {
                return Some(title.clone());
            }
        }
        
        if let Some(Value::Number(event_id)) = data.get("concreteEventID") {
            return Some(format!("Widget {}", event_id));
        }
        
        None
    }

    fn extract_display_type(&self, data: &HashMap<String, Value>) -> Option<String> {
        if let Some(Value::String(display_type)) = data.get("displayType") {
            return Some(display_type.clone());
        }
        
        if let Some(Value::String(widget_type)) = data.get("widgetType") {
            return Some(widget_type.clone());
        }
        
        if let Some(Value::String(control_type)) = data.get("controlType") {
            return Some(control_type.clone());
        }
        
        None
    }

    fn extract_float_field(&self, data: &HashMap<String, Value>, field_name: &str) -> Option<f64> {
        if let Some(value) = data.get(field_name) {
            match value {
                Value::Number(n) => n.as_f64(),
                Value::String(s) => s.parse::<f64>().ok(),
                _ => None,
            }
        } else {
            None
        }
    }

    fn extract_bool_field(&self, data: &HashMap<String, Value>, field_name: &str) -> Option<bool> {
        if let Some(value) = data.get(field_name) {
            match value {
                Value::Bool(b) => Some(*b),
                Value::String(s) => match s.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => Some(true),
                    "false" | "0" | "no" | "off" => Some(false),
                    _ => None,
                },
                Value::Number(n) => {
                    if let Some(num) = n.as_i64() {
                        Some(num != 0)
                    } else {
                        None
                    }
                },
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn extract_widget_metadata(&self, event_id: i64) -> Option<WidgetMetadata> {
        let kyma_data = self.widget_descriptions.get(&event_id)?;
        
        Some(WidgetMetadata {
            event_id,
            label: self.extract_label(kyma_data),
            display_type: self.extract_display_type(kyma_data),
            minimum: self.extract_float_field(kyma_data, "minimum"),
            maximum: self.extract_float_field(kyma_data, "maximum"),
            default_value: self.extract_float_field(kyma_data, "defaultValue")
                .or_else(|| self.extract_float_field(kyma_data, "default")),
            is_generated: self.extract_bool_field(kyma_data, "isGenerated"),
            units: self.extract_string_field(kyma_data, "units"),
            category: self.extract_string_field(kyma_data, "category"),
            description: self.extract_string_field(kyma_data, "description"),
        })
    }

    fn extract_string_field(&self, data: &HashMap<String, Value>, field_name: &str) -> Option<String> {
        if let Some(Value::String(s)) = data.get(field_name) {
            if !s.is_empty() {
                Some(s.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn parse_kyma_json_string(json_str: &str) -> Result<HashMap<String, Value>, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    pub fn validate_kyma_data(data: &HashMap<String, Value>) -> Result<(), String> {
        if !data.contains_key("concreteEventID") {
            return Err("Missing required field: concreteEventID".to_string());
        }
        
        if let Some(Value::Number(event_id)) = data.get("concreteEventID") {
            if event_id.as_i64().is_none() {
                return Err("concreteEventID must be a valid integer".to_string());
            }
        } else {
            return Err("concreteEventID must be a number".to_string());
        }
        
        Ok(())
    }
}

impl Default for KymaWidgetExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WidgetMetadata {
    pub event_id: i64,
    pub label: Option<String>,
    pub display_type: Option<String>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    pub default_value: Option<f64>,
    pub is_generated: Option<bool>,
    pub units: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
}

impl WidgetMetadata {
    pub fn to_widget(&self, current_value: f64) -> Widget {
        Widget {
            label: self.label.clone(),
            minimum: self.minimum,
            maximum: self.maximum,
            current_value: Some(current_value),
            is_generated: self.is_generated,
            display_type: self.display_type.clone(),
        }
    }

    pub fn is_valid_value(&self, value: f64) -> bool {
        match (self.minimum, self.maximum) {
            (Some(min), Some(max)) => value >= min && value <= max,
            (Some(min), None) => value >= min,
            (None, Some(max)) => value <= max,
            (None, None) => true,
        }
    }

    pub fn normalize_value(&self, value: f64) -> Option<f64> {
        match (self.minimum, self.maximum) {
            (Some(min), Some(max)) if max > min => {
                Some((value - min) / (max - min))
            },
            _ => None,
        }
    }

    pub fn denormalize_value(&self, normalized_value: f64) -> Option<f64> {
        match (self.minimum, self.maximum) {
            (Some(min), Some(max)) if max > min => {
                Some(min + normalized_value * (max - min))
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_kyma_extractor_basic() {
        let mut extractor = KymaWidgetExtractor::new();
        
        let kyma_data = json!({
            "concreteEventID": 100,
            "label": "Master Volume",
            "minimum": 0.0,
            "maximum": 127.0,
            "displayType": "slider",
            "isGenerated": false
        });
        
        let data_map: HashMap<String, Value> = serde_json::from_value(kyma_data).unwrap();
        extractor.cache_widget_description(data_map);
        
        let widget = extractor.create_training_widget(100, 95.0);
        assert!(widget.is_some());
        
        let widget = widget.unwrap();
        assert_eq!(widget.label, Some("Master Volume".to_string()));
        assert_eq!(widget.minimum, Some(0.0));
        assert_eq!(widget.maximum, Some(127.0));
        assert_eq!(widget.current_value, Some(95.0));
        assert_eq!(widget.display_type, Some("slider".to_string()));
        assert_eq!(widget.is_generated, Some(false));
    }

    #[test]
    fn test_widget_metadata() {
        let metadata = WidgetMetadata {
            event_id: 100,
            label: Some("Test Widget".to_string()),
            display_type: Some("knob".to_string()),
            minimum: Some(0.0),
            maximum: Some(100.0),
            default_value: Some(50.0),
            is_generated: Some(false),
            units: Some("dB".to_string()),
            category: Some("Audio".to_string()),
            description: Some("Test widget description".to_string()),
        };
        
        assert!(metadata.is_valid_value(50.0));
        assert!(!metadata.is_valid_value(150.0));
        assert!(!metadata.is_valid_value(-10.0));
        
        assert_eq!(metadata.normalize_value(50.0), Some(0.5));
        assert_eq!(metadata.denormalize_value(0.5), Some(50.0));
        
        let widget = metadata.to_widget(75.0);
        assert_eq!(widget.current_value, Some(75.0));
        assert_eq!(widget.label, Some("Test Widget".to_string()));
    }

    #[test]
    fn test_json_parsing() {
        let json_str = r#"{"concreteEventID": 123, "label": "Test", "minimum": 0, "maximum": 100}"#;
        let parsed = KymaWidgetExtractor::parse_kyma_json_string(json_str);
        assert!(parsed.is_ok());
        
        let data = parsed.unwrap();
        assert!(KymaWidgetExtractor::validate_kyma_data(&data).is_ok());
    }

    #[test]
    fn test_invalid_json() {
        let json_str = r#"{"label": "Test"}"#;
        let parsed = KymaWidgetExtractor::parse_kyma_json_string(json_str);
        assert!(parsed.is_ok());
        
        let data = parsed.unwrap();
        assert!(KymaWidgetExtractor::validate_kyma_data(&data).is_err());
    }

    #[test]
    fn test_extract_all_widgets() {
        let mut extractor = KymaWidgetExtractor::new();
        
        let kyma_data1 = json!({
            "concreteEventID": 100,
            "label": "Volume",
            "minimum": 0.0,
            "maximum": 127.0
        });
        
        let kyma_data2 = json!({
            "concreteEventID": 101,
            "label": "Pan",
            "minimum": -64.0,
            "maximum": 64.0
        });
        
        let data_map1: HashMap<String, Value> = serde_json::from_value(kyma_data1).unwrap();
        let data_map2: HashMap<String, Value> = serde_json::from_value(kyma_data2).unwrap();
        
        extractor.cache_widget_description(data_map1);
        extractor.cache_widget_description(data_map2);
        
        let mut values = HashMap::new();
        values.insert(100, 95.0);
        values.insert(101, 0.0);
        values.insert(102, 50.0); // This one doesn't have cached description
        
        let widgets = extractor.extract_all_widgets_with_values(&values);
        assert_eq!(widgets.len(), 2); // Only the first two should be extracted
        
        // The widgets may be returned in any order, just check they're both present
        let labels: Vec<_> = widgets.iter().filter_map(|w| w.label.as_ref()).collect();
        assert!(labels.contains(&&"Volume".to_string()));
        assert!(labels.contains(&&"Pan".to_string()));
    }
}