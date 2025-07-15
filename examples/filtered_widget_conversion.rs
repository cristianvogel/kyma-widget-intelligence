// Example demonstrating how to use the From<FilteredWidgetDescription> trait
use widget_intelligence::{FilteredWidgetDescription, WidgetRecord};
use serde_json::{Value, Number};
use std::collections::HashMap;

fn main() {
    // Simulate getting a filtered widget description from your filter_widget_json function
    let filtered_widget = create_sample_filtered_widget();
    
    // Convert using the idiomatic .into() method
    let widget_record: WidgetRecord = filtered_widget.into();
    
    println!("Created WidgetRecord from FilteredWidgetDescription:");
    println!("  ID: {}", widget_record.id);
    println!("  Label: {:?}", widget_record.widget.label);
    println!("  Min: {:?}", widget_record.widget.minimum);
    println!("  Max: {:?}", widget_record.widget.maximum);
    println!("  Display Type: {:?}", widget_record.widget.display_type);
    println!("  Is Generated: {:?}", widget_record.widget.is_generated);
    println!("  Frequency: {}", widget_record.frequency);
    println!("  Features:");
    println!("    Label Tokens: {:?}", widget_record.features.label_tokens);
    println!("    Range: {}", widget_record.features.range);
    println!("    Display Type Hash: {}", widget_record.features.display_type_hash);
    
    // Example of how you would use this in your code:
    // let filtered_widget_description = filter_widget_json(&mut deserialized_kyma_widget);
    // let widget_record: WidgetRecord = filtered_widget_description.into();
    // sled.store_widget(&widget_record).unwrap();
}

fn create_sample_filtered_widget() -> FilteredWidgetDescription {
    let mut filtered = HashMap::new();
    
    // Add the fields that your filter_widget_json function would populate
    filtered.insert("concreteEventID".to_string(), Value::Number(Number::from(42)));
    filtered.insert("label".to_string(), Value::String("Master Volume Control".to_string()));
    filtered.insert("minimum".to_string(), Value::Number(Number::from_f64(0.0).unwrap()));
    filtered.insert("maximum".to_string(), Value::Number(Number::from_f64(127.0).unwrap()));
    filtered.insert("displayType".to_string(), Value::String("slider".to_string()));
    filtered.insert("isGenerated".to_string(), Value::Bool(false));
    filtered.insert("isBoolean".to_string(), Value::Bool(false));
    filtered.insert("isAggregate".to_string(), Value::Bool(false));
    filtered.insert("isFullRange".to_string(), Value::Bool(false));
    filtered.insert("isEventSource".to_string(), Value::Bool(true));
    
    filtered
}