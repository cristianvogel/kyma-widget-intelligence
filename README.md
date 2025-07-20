# Widget Intelligence

A Rust library for intelligent widget suggestion and learning based on user behavior patterns.

Used in production https://neverenginelabs.com/products/nel-vcs-player 

## Features

- **Simple API**: Register widgets with Label, EventId, and values, then get suggestions
- **Smart Suggestions**: Query by EventID or Label to get suggested values based on training
- **Persistent Storage**: Sled-based database for long-term learning
- **Pure Rust**: Pure Rust library without UI framework dependencies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
widget_intelligence = "0.1"
```

## Usage

### Simplified Use Case

The Widget Intelligence system has been simplified to focus on the following use case:

1. Register widgets with Label, EventId, and a vector of values
2. Query for suggested values by EventId or Label

```rust
use widget_intelligence::*;

// Create a new suggestion engine
let mut engine = WidgetSuggestionEngine::new();

// Register a widget with Label, EventId, and values
let widget = Widget::simplified(
    Some("Volume".to_string()),
    Some(101),
    vec![0.7, 0.8, 0.75]
);
engine.store_widget(widget);

// Add more values to an existing widget
let widget_update = Widget::simplified(
    Some("Volume".to_string()),
    Some(101),
    vec![0.65, 0.85]
);
engine.store_widget(widget_update);

// Query for a suggested value by EventId
let suggestions = engine.get_suggestions_by_event_id(101, 1);
if let Some(suggestion) = suggestions.first() {
    println!("Suggested value: {:?}", suggestion.suggested_value);
    println!("Alternative values: {:?}", suggestion.alternative_values);
}

// Query for a suggested value by Label
let query_widget = Widget::simplified(
    Some("Volume".to_string()),
    None,
    vec![]
);
let suggestions = engine.get_suggestions(&query_widget, 1);
if let Some(suggestion) = suggestions.first() {
    println!("Suggested value: {:?}", suggestion.suggested_value);
    println!("Alternative values: {:?}", suggestion.alternative_values);
}
```

### Persistent Storage

For persistent storage, use the `PersistentWidgetSuggestionEngine`:

```rust
// Initialize with a database path
let mut system = init_intelligence_system("widget_db")?;

// Register widgets and get suggestions as above
let widget = Widget::simplified(
    Some("Volume".to_string()),
    Some(101),
    vec![0.7, 0.8, 0.75]
);
system.store_widget(widget)?;

// Flush changes to disk
system.flush()?;
```
