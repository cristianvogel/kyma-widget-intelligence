use widget_intelligence::*;
use colored::*;
use std::collections::HashMap;

fn create_kyma_widget(id: u64, label: &str, min: f64, max: f64, current: f64) -> FilteredWidgetDescription {
    let mut filtered = HashMap::new();

    filtered.insert(
        "concreteEventID".to_string(),
        serde_json::Value::Number(serde_json::Number::from(id)),
    );
    filtered.insert(
        "label".to_string(),
        serde_json::Value::String(label.to_string()),
    );
    filtered.insert(
        "minimum".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(min).unwrap()),
    );
    filtered.insert(
        "maximum".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(max).unwrap()),
    );
    filtered.insert(
        "current_value".to_string(),
        serde_json::Value::Number(serde_json::Number::from_f64(current).unwrap()),
    );
    filtered.insert(
        "displayType".to_string(),
        serde_json::Value::String("slider".to_string()),
    );
    filtered.insert("isGenerated".to_string(), serde_json::Value::Bool(false));

    filtered
}

fn print_separator() {
    println!("{}", "─".repeat(80).blue());
}

#[test]
fn test_event_id_suggestions() {
    colored::control::set_override(true);

    println!("\n{}", "EVENT ID SUGGESTIONS TEST".bold().underline());

    let mut engine = WidgetSuggestionEngine::new();

    // Create widgets with different event IDs
    let event_id_1 = 42;
    let event_id_2 = 43;

    // Create multiple widgets with the same event ID but different values
    // This simulates observing the same widget with different values over time
    let widget1a: WidgetRecord = create_kyma_widget(event_id_1, "Volume", 0.0, 1.0, 0.7).into();
    let widget1b: WidgetRecord = create_kyma_widget(event_id_1, "Volume", 0.0, 1.0, 0.8).into();
    let widget1c: WidgetRecord = create_kyma_widget(event_id_1, "Volume", 0.0, 1.0, 0.7).into();

    // Create widgets with a different event ID
    let widget2a: WidgetRecord = create_kyma_widget(event_id_2, "Cutoff", -24.0, 24.0, 12.0).into();
    let widget2b: WidgetRecord = create_kyma_widget(event_id_2, "Cutoff", -24.0, 24.0, 6.0).into();

    print_separator();
    println!("{} {}", "→".green(), "Storing widgets with event IDs...".yellow());

    // Store the widgets directly to avoid merging
    engine.records.push(widget1a);
    engine.records.push(widget1b);
    engine.records.push(widget1c);
    engine.records.push(widget2a);
    engine.records.push(widget2b);

    // Test getting suggestions by event ID
    print_separator();
    println!("{} {}", "→".green(), "Testing suggestions by event ID...".yellow());

    let suggestions = engine.get_suggestions_by_event_id(event_id_1, 3);

    println!(
        "{} {}",
        "→".green(),
        format!("Got {} suggestions for event ID {}", suggestions.len(), event_id_1).cyan()
    );

    for suggestion in &suggestions {
        println!(
            "  • {} (confidence: {:.2}, suggested value: {:?})",
            suggestion
                .widget
                .label
                .as_deref()
                .unwrap_or("Unknown")
                .cyan(),
            suggestion.confidence.to_string().yellow(),
            suggestion.suggested_value
        );
    }

    // Verify that we got suggestions and that the most common value (0.7) is suggested
    assert!(!suggestions.is_empty());
    assert_eq!(suggestions[0].suggested_value, Some(0.7));

    // Test getting suggestions for a different event ID
    let suggestions2 = engine.get_suggestions_by_event_id(event_id_2, 3);

    println!(
        "{} {}",
        "→".green(),
        format!("Got {} suggestions for event ID {}", suggestions2.len(), event_id_2).cyan()
    );

    for suggestion in &suggestions2 {
        println!(
            "  • {} (confidence: {:.2}, suggested value: {:?})",
            suggestion
                .widget
                .label
                .as_deref()
                .unwrap_or("Unknown")
                .cyan(),
            suggestion.confidence.to_string().yellow(),
            suggestion.suggested_value
        );
    }

    // Verify that we got suggestions for the second event ID
    assert!(!suggestions2.is_empty());

    println!("\n{}", "TEST PASSED".bold().green());
}
