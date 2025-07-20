use widget_intelligence::*;
use colored::*;

#[test]
fn test_simplified_use_case() {
    colored::control::set_override(true);
    println!("\n{}", "SIMPLIFIED USE CASE TEST".bold().underline());
    
    let mut engine = WidgetSuggestionEngine::new();
    
    // Register widgets with Label, EventId, and values
    println!("{} {}", "→".green(), "Registering widgets...".yellow());
    
    // Widget 1: Register by EventId 101
    let widget1 = Widget::simplified(
        Some("Volume".to_string()),
        Some(101),
        vec![0.7, 0.8, 0.75]
    );
    engine.store_widget(widget1);
    println!("  • Registered widget with EventId 101 (Volume)");
    
    // Widget 2: Register by EventId 102
    let widget2 = Widget::simplified(
        Some("Cutoff".to_string()),
        Some(102),
        vec![0.3, 0.4, 0.35]
    );
    engine.store_widget(widget2);
    println!("  • Registered widget with EventId 102 (Cutoff)");
    
    // Widget 3: Register by Label only
    let widget3 = Widget::simplified(
        Some("Pan".to_string()),
        None,
        vec![0.5, 0.6, 0.55]
    );
    engine.store_widget(widget3);
    println!("  • Registered widget with Label 'Pan'");
    
    // Add more values to existing widgets
    let widget1_update = Widget::simplified(
        Some("Volume".to_string()),
        Some(101),
        vec![0.65, 0.85]
    );
    engine.store_widget(widget1_update);
    println!("  • Added more values to EventId 101 (Volume)");
    
    // Test querying by EventId
    println!("\n{} {}", "→".green(), "Querying by EventId...".yellow());
    
    let suggestions_by_event_id = engine.get_suggestions_by_event_id(101, 1);
    assert!(!suggestions_by_event_id.is_empty(), "Should get suggestions by EventId");
    
    let suggestion = &suggestions_by_event_id[0];
    println!(
        "  • Suggested value for EventId 101: {} (confidence: {:.2})",
        suggestion.suggested_value.unwrap_or(0.0).to_string().cyan(),
        suggestion.confidence
    );
    println!(
        "  • Alternative values: {:?}",
        suggestion.alternative_values
    );
    
    // Test querying by Label
    println!("\n{} {}", "→".green(), "Querying by Label...".yellow());
    
    let query_widget = Widget::simplified(
        Some("Pan".to_string()),
        None,
        vec![]
    );
    
    let suggestions_by_label = engine.get_suggestions(&query_widget, 1);
    assert!(!suggestions_by_label.is_empty(), "Should get suggestions by Label");
    
    let suggestion = &suggestions_by_label[0];
    println!(
        "  • Suggested value for Label 'Pan': {} (confidence: {:.2})",
        suggestion.suggested_value.unwrap_or(0.0).to_string().cyan(),
        suggestion.confidence
    );
    println!(
        "  • Alternative values: {:?}",
        suggestion.alternative_values
    );
    
    println!("\n{}", "TEST PASSED".bold().green());
}