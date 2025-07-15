use crate::similarity_engine::Widget;
use serde_json::Value;
use std::collections::HashMap;

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
                log::trace!("Caching widget description for event ID: {id}");
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

        log::trace!(
            "Created training widget for event ID {}: {:?}",
            event_id,
            widget.label
        );
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
            return Some(format!("Widget {event_id}"));
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
                Value::Number(n) => n.as_i64().map(|num| num != 0),
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
            default_value: self
                .extract_float_field(kyma_data, "defaultValue")
                .or_else(|| self.extract_float_field(kyma_data, "default")),
            is_generated: self.extract_bool_field(kyma_data, "isGenerated"),
            units: self.extract_string_field(kyma_data, "units"),
            category: self.extract_string_field(kyma_data, "category"),
            description: self.extract_string_field(kyma_data, "description"),
        })
    }

    fn extract_string_field(
        &self,
        data: &HashMap<String, Value>,
        field_name: &str,
    ) -> Option<String> {
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
        serde_json::from_str(json_str).map_err(|e| format!("Failed to parse JSON: {e}"))
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
            (Some(min), Some(max)) if max > min => Some((value - min) / (max - min)),
            _ => None,
        }
    }

    pub fn denormalize_value(&self, normalized_value: f64) -> Option<f64> {
        match (self.minimum, self.maximum) {
            (Some(min), Some(max)) if max > min => Some(min + normalized_value * (max - min)),
            _ => None,
        }
    }
}
