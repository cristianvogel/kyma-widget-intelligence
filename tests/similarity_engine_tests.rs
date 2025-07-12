use widget_intelligence::*;
use colored::*;
use std::collections::HashMap;

fn create_kyma_widget(label: &str, min: f64, max: f64, current: f64) -> Widget {
    Widget {
        label: Some(label.to_string()),
        minimum: Some(min),
        maximum: Some(max),
        current_value: Some(current),
        is_generated: Some(false),
        display_type: Some("slider".to_string()),
    }
}

fn create_preset_data(name: &str, widget_values: HashMap<String, f64>) -> Preset {
    let widget_values: Vec<WidgetValue> = widget_values
        .into_iter()
        .map(|(id, value)| WidgetValue {
            widget_id: id,
            label: None,
            value,
            confidence: 1.0,
        })
        .collect();

    Preset {
        name: name.to_string(),
        description: None,
        widget_values,
        created_by: None,
        usage_count: 1,
        last_used: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    }
}

#[test]
fn test_kyma_style_widgets() {
    colored::control::set_override(true);

    println!("\n{}", "KYMA STYLE WIDGETS TEST".bold().underline());

    let mut engine = WidgetSuggestionEngine::new();

    // Common Kyma-style widgets
    let widgets = vec![
        create_kyma_widget("Amp_01", 0.0, 1.0, 0.75),
        create_kyma_widget("Amp_02", -1.0, 1.0, 0.2),
        create_kyma_widget("sw_00", 0.0, 1.0, 0.9),
        create_kyma_widget("Gate", 0.0, 1.0, 0.5),
        create_kyma_widget("cutoff", -24.0, 24.0, 5.0),
        create_kyma_widget("rate", 0.0, 1.0, 0.3),
        create_kyma_widget("morph", -1.0, 1.0, -0.4),
        create_kyma_widget("Amp_03", 0.0, 1.0, 0.8),
        create_kyma_widget("sw_01", 0.0, 1.0, 0.1),
        create_kyma_widget("morph2", -1.0, 1.0, 0.6),
    ];

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Storing Kyma-style widgets...".yellow()
    );

    for widget in widgets {
        engine.store_widget(widget);
    }

    println!(
        "{} {}",
        "→".green(),
        format!("Stored {} widget records", engine.records.len()).cyan()
    );

    // Test suggestions for similar widgets
    let test_widget = Widget {
        label: Some("Amp_04".to_string()),
        minimum: Some(0.0),
        maximum: Some(1.0),
        ..Default::default()
    };

    let suggestions = engine.get_suggestions(&test_widget, 3);
    println!(
        "{} {}",
        "→".green(),
        format!("Got {} suggestions for 'Amp_04'", suggestions.len()).cyan()
    );

    for suggestion in &suggestions {
        println!(
            "  • {} (confidence: {:.4})",
            suggestion
                .widget
                .label
                .as_deref()
                .unwrap_or("Unknown")
                .cyan(),
            suggestion.confidence.to_string().yellow()
        );
    }

    assert!(!suggestions.is_empty());
    println!("\n{}", "TEST PASSED".bold().green());
}

#[test]
fn test_realistic_presets() {
    colored::control::set_override(true);

    println!("\n{}", "REALISTIC PRESETS TEST".bold().underline());

    let mut engine = WidgetSuggestionEngine::new();

    // Store some widgets first
    engine.store_widget(create_kyma_widget("Amp_01", 0.0, 1.0, 0.8));
    engine.store_widget(create_kyma_widget("cutoff", 0.0, 3_500.0, 630.0));
    engine.store_widget(create_kyma_widget("Gate", 0.0, 1.0, 0.7));
    engine.store_widget(create_kyma_widget("rate", 0.0, 1.0, 0.4));
    engine.store_widget(create_kyma_widget("index", 0.0, 1.0, 0.22));
    engine.store_widget(create_kyma_widget("index", 0.0, 1.0, 0.82));

    // Create realistic presets
    let presets = vec![
        create_preset_data("FuzzySparks", {
            let mut values = HashMap::new();
            values.insert("13755".to_string(), 0.85);
            values.insert("13756".to_string(), 18.0);
            values.insert("13757".to_string(), 0.9);
            values
        }),
        create_preset_data("Default", {
            let mut values = HashMap::new();
            values.insert("13755".to_string(), 0.5);
            values.insert("13756".to_string(), 0.0);
            values.insert("13757".to_string(), 0.5);
            values
        }),
        create_preset_data("Default01", {
            let mut values = HashMap::new();
            values.insert("13755".to_string(), 0.6);
            values.insert("13756".to_string(), 2.0);
            values.insert("13757".to_string(), 0.4);
            values
        }),
        create_preset_data("SizzlingDrips", {
            let mut values = HashMap::new();
            values.insert("13755".to_string(), 0.95);
            values.insert("13756".to_string(), -8.0);
            values.insert("13757".to_string(), 0.1);
            values.insert("13758".to_string(), 0.75);
            values
        }),
    ];

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Storing realistic presets...".yellow()
    );

    for preset in presets {
        engine.store_preset(preset);
    }

    println!(
        "{} {}",
        "→".green(),
        format!("Stored {} presets", engine.presets.len()).cyan()
    );

    // Test preset insights
    let test_widget = Widget {
        label: Some("Amp_01".to_string()),
        ..Default::default()
    };

    if let Some(insight) = engine.get_preset_insights(&test_widget) {
        println!(
            "{} {}",
            "→".green(),
            format!("Preset insight: {}", insight).cyan()
        );
    }

    let stats = engine.get_stats();
    println!(
        "{} {}",
        "→".green(),
        format!("Final stats: {:?}", stats).cyan()
    );

    assert_eq!(stats.get("total_presets"), Some(&4));
    println!("\n{}", "TEST PASSED".bold().green());
}

#[test]
fn test_value_ranges() {
    colored::control::set_override(true);

    println!("\n{}", "VALUE RANGES TEST".bold().underline());

    let mut engine = WidgetSuggestionEngine::new();

    // Test different common ranges
    let range_tests = vec![
        ("sw_00",   0.0, 1.0, 1.0),
        ("sw_01", 0.0, 1.0, 0.0),
        ("cutoff", 0.0, 8_000.0 , 650.0),
        ("rate", -1.0, 1.0, 0.666),
        ("morph", 0.0, 1.0, 0.3),
        ("Gate", 0.0, 1.0, 1.0),
        ("Amp_01", 0.0, 1.0, 0.7),
        ("Amp_02", -1.0, 1.0, -0.4),
    ];

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Testing various value ranges...".yellow()
    );

    for (label, min, max, current) in range_tests {
        let widget = create_kyma_widget(label, min, max, current);
        engine.store_widget(widget);

        println!(
            "  • {} [{:.1}, {:.1}] = {:.1}",
            label.cyan(),
            min,
            max,
            current
        );
    }

    // Test similarity within the same ranges
    let test_widget = Widget {
        label: Some("index".to_string()),
        minimum: Some(0.0),
        maximum: Some(1.0),
        ..Default::default()
    };

    let suggestions = engine.get_suggestions(&test_widget, 3);
    println!(
        "\n{} {}",
        "→".green(),
        format!("Suggestions for {}", test_widget.label.as_deref().unwrap_or("Unknown")).cyan()
    );

    for suggestion in &suggestions {
        println!(
            "  • {} (confidence: {:.4})",
            suggestion
                .widget
                .label
                .as_deref()
                .unwrap_or("Unknown")
                .cyan(),
            suggestion.confidence.to_string().yellow()
        );
    }

    assert!(!suggestions.is_empty());
    println!("\n{}", "TEST PASSED".bold().green());
}

#[test]
fn test_morph_variants() {
    colored::control::set_override(true);

    println!("\n{}", "MORPH VARIANTS TEST".bold().underline());

    let mut engine = WidgetSuggestionEngine::new();

    // Test various morph-style widgets
    let morph_widgets = vec![
        create_kyma_widget("morph", -1.0, 1.0, 0.3),
        create_kyma_widget("morph2", -1.0, 1.0, -0.7),
        create_kyma_widget("morph3", -1.0, 1.0, 0.1),
        create_kyma_widget("morph_x", -1.0, 1.0, 0.8),
        create_kyma_widget("morph_y", -1.0, 1.0, -0.2),
    ];

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Testing morph-style widgets...".yellow()
    );

    for widget in morph_widgets {
        engine.store_widget(widget);
    }

    // Test suggestions for new morph widget
    let test_widget = Widget {
        label: Some("morph4".to_string()),
        minimum: Some(-1.0),
        maximum: Some(1.0),
        ..Default::default()
    };

    let suggestions = engine.get_suggestions(&test_widget, 5);
    println!(
        "{} {}",
        "→".green(),
        format!("Got {} suggestions for 'morph4'", suggestions.len()).cyan()
    );

    for suggestion in &suggestions {
        println!(
            "  • {} (confidence: {:.4})",
            suggestion
                .widget
                .label
                .as_deref()
                .unwrap_or("Unknown")
                .cyan(),
            suggestion.confidence.to_string().yellow()
        );
    }

    // Should find high similarity with other morph widgets
    assert!(suggestions.len() >= 3);
    assert!(suggestions[0].confidence > 0.7);

    println!("\n{}", "TEST PASSED".bold().green());
}

#[test]
fn test_amp_series() {
    colored::control::set_override(true);

    println!("\n{}", "AMP SERIES TEST".bold().underline());

    let mut engine = WidgetSuggestionEngine::new();

    // Test Amp series widgets
    let amp_widgets = vec![
        create_kyma_widget("Amp_01", 0.0, 1.0, 0.8),
        create_kyma_widget("Amp_02", 0.0, 1.0, 0.6),
        create_kyma_widget("Amp_03", 0.0, 1.0, 0.9),
        create_kyma_widget("Amp_04", 0.0, 1.0, 0.7),
        create_kyma_widget("Amp_05", 0.0, 1.0, 0.5),
    ];

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Testing Amp series widgets...".yellow()
    );

    for widget in amp_widgets {
        engine.store_widget(widget);
    }

    // Test suggestions for new Amp widget
    let test_widget = Widget {
        label: Some("Amp_06".to_string()),
        minimum: Some(0.0),
        maximum: Some(1.0),
        ..Default::default()
    };

    let suggestions = engine.get_suggestions(&test_widget, 5);
    println!(
        "{} {}",
        "→".green(),
        format!("Got {} suggestions for 'Amp_06'", suggestions.len()).cyan()
    );

    for suggestion in &suggestions {
        println!(
            "  • {} (confidence: {:.4})",
            suggestion
                .widget
                .label
                .as_deref()
                .unwrap_or("Unknown")
                .cyan(),
            suggestion.confidence.to_string().yellow()
        );
    }

    // Should find high similarity with other Amp widgets
    assert!(!suggestions.is_empty());
    assert!(suggestions[0].confidence > 0.8);

    println!("\n{}", "TEST PASSED".bold().green());
}

fn print_separator() {
    println!("{}", "─".repeat(80).blue());
}
