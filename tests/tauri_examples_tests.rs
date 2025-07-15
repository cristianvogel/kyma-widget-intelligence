use ::widget_intelligence::*;
use colored::*;
use std::collections::HashMap;
use tempfile::tempdir;

fn print_separator() {
    println!("{}", "─".repeat(80).blue());
}

#[tokio::test]
async fn test_kyma_standalone_service() {
    control::set_override(true);

    println!("\n{}", "KYMA STANDALONE SERVICE TEST".bold().underline());

    let temp_dir = tempdir().unwrap();
    let db_path_buf = temp_dir.path().join("test_kyma_standalone");
    let db_path = db_path_buf.to_str().unwrap();

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Initializing standalone service...".yellow()
    );
    let service = StandaloneIntelligenceService::new(db_path).unwrap();

    // Test with realistic Kyma widgets
    let kyma_widgets = vec![
        (
            13755,
            r#"{"concreteEventID": 13755, "label": "Amp_01", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.75,
        ),
        (
            13756,
            r#"{"concreteEventID": 13756, "label": "cutoff", "minimum": -24.0, "maximum": 24.0, "displayType": "slider"}"#,
            8.5,
        ),
        (
            13757,
            r#"{"concreteEventID": 13757, "label": "Gate", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.6,
        ),
        (
            13758,
            r#"{"concreteEventID": 13758, "label": "morph", "minimum": -1.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.3,
        ),
        (
            13759,
            r#"{"concreteEventID": 13759, "label": "sw_00", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.9,
        ),
    ];

    println!(
        "{} {}",
        "→".green(),
        "Caching widget descriptions...".yellow()
    );
    for (event_id, kyma_json, _) in &kyma_widgets {
        service
            .cache_widget_description(*event_id, kyma_json.to_string())
            .await
            .unwrap();
    }

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Testing realistic presets...".yellow()
    );

    // Test preset: FuzzySparks
    let mut widget_values = HashMap::new();
    widget_values.insert("13755".to_string(), 0.85);
    widget_values.insert("13756".to_string(), 18.0);
    widget_values.insert("13757".to_string(), 0.7);

    let preset_data = PresetData {
        name: "FuzzySparks".to_string(),
        description: None,
        widget_values,
        created_by: None,
    };

    let stats = service.save_preset_and_learn(preset_data).await.unwrap();
    println!(
        "{} {}",
        "→".green(),
        format!(
            "Stored FuzzySparks preset: {} widgets, {} presets",
            stats.total_widgets, stats.total_presets
        )
        .cyan()
    );

    // Test preset: Default
    let mut widget_values = HashMap::new();
    widget_values.insert("13755".to_string(), 0.5);
    widget_values.insert("13756".to_string(), 0.0);
    widget_values.insert("13757".to_string(), 0.5);

    let preset_data = PresetData {
        name: "Default".to_string(),
        description: None,
        widget_values,
        created_by: None,
    };

    let stats = service.save_preset_and_learn(preset_data).await.unwrap();
    println!(
        "{} {}",
        "→".green(),
        format!(
            "Stored Default preset: {} widgets, {} presets",
            stats.total_widgets, stats.total_presets
        )
        .cyan()
    );

    // Test preset: SizzlingDrips
    let mut widget_values = HashMap::new();
    widget_values.insert("13755".to_string(), 0.95);
    widget_values.insert("13756".to_string(), -12.0);
    widget_values.insert("13757".to_string(), 0.2);
    widget_values.insert("13758".to_string(), 0.8);

    let preset_data = PresetData {
        name: "SizzlingDrips".to_string(),
        description: None,
        widget_values,
        created_by: None,
    };

    let stats = service.save_preset_and_learn(preset_data).await.unwrap();
    println!(
        "{} {}",
        "→".green(),
        format!(
            "Stored SizzlingDrips preset: {} widgets, {} presets",
            stats.total_widgets, stats.total_presets
        )
        .cyan()
    );

    assert!(stats.total_widgets > 0);
    assert!(stats.total_presets > 0);

    print_separator();
    println!("{} {}", "→".green(), "Testing suggestions...".yellow());

    // Test suggestions for Amp series
    let suggestions = service
        .get_widget_value_suggestions(
            13760,
            Some("Amp_02".to_string()),
            Some("slider".to_string()),
        )
        .await
        .unwrap();

    println!(
        "{} {}",
        "→".green(),
        format!("Got {} suggestions for Amp_02", suggestions.len()).cyan()
    );
    for suggestion in &suggestions {
        println!(
            "  • Value: {:?} (confidence: {:.2})",
            suggestion.suggested_value.unwrap_or(0.0),
            suggestion.confidence
        );
    }

    // Test suggestions for morph series
    let suggestions = service
        .get_widget_value_suggestions(
            13761,
            Some("morph2".to_string()),
            Some("slider".to_string()),
        )
        .await
        .unwrap();

    println!(
        "{} {}",
        "→".green(),
        format!("Got {} suggestions for morph2", suggestions.len()).cyan()
    );
    for suggestion in &suggestions {
        println!(
            "  • Value: {:?} (confidence: {:.2})",
            suggestion.suggested_value.unwrap_or(0.0),
            suggestion.confidence
        );
    }

    println!("\n{}", "TEST PASSED".bold().green());
}

#[tokio::test]
async fn test_kyma_widget_patterns() {
    control::set_override(true);

    println!("\n{}", "KYMA WIDGET PATTERNS TEST".bold().underline());

    let temp_dir = tempdir().unwrap();
    let db_path_buf = temp_dir.path().join("test_kyma_patterns");
    let db_path = db_path_buf.to_str().unwrap();

    let service = StandaloneIntelligenceService::new(db_path).unwrap();

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Testing common value ranges...".yellow()
    );

    // Test (-1, 1) range widgets
    let bipolar_widgets = vec![
        (
            14001,
            r#"{"concreteEventID": 14001, "label": "morph", "minimum": -1.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.3,
        ),
        (
            14002,
            r#"{"concreteEventID": 14002, "label": "morph2", "minimum": -1.0, "maximum": 1.0, "displayType": "slider"}"#,
            -0.7,
        ),
        (
            14003,
            r#"{"concreteEventID": 14003, "label": "morph3", "minimum": -1.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.1,
        ),
    ];

    for (event_id, kyma_json, _) in &bipolar_widgets {
        service
            .cache_widget_description(*event_id, kyma_json.to_string())
            .await
            .unwrap();
    }

    // Test (0, 1) range widgets
    let unipolar_widgets = vec![
        (
            14004,
            r#"{"concreteEventID": 14004, "label": "Amp_01", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.8,
        ),
        (
            14005,
            r#"{"concreteEventID": 14005, "label": "Amp_02", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.6,
        ),
        (
            14006,
            r#"{"concreteEventID": 14006, "label": "sw_00", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
            0.9,
        ),
    ];

    for (event_id, kyma_json, _) in &unipolar_widgets {
        service
            .cache_widget_description(*event_id, kyma_json.to_string())
            .await
            .unwrap();
    }

    // Test (-24, 24) range widgets
    let audio_widgets = vec![
        (
            14007,
            r#"{"concreteEventID": 14007, "label": "cutoff", "minimum": -24.0, "maximum": 24.0, "displayType": "slider"}"#,
            12.0,
        ),
        (
            14008,
            r#"{"concreteEventID": 14008, "label": "rate", "minimum": -24.0, "maximum": 24.0, "displayType": "slider"}"#,
            -5.0,
        ),
    ];

    for (event_id, kyma_json, _) in &audio_widgets {
        service
            .cache_widget_description(*event_id, kyma_json.to_string())
            .await
            .unwrap();
    }

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Creating pattern-based preset...".yellow()
    );

    let mut widget_values = HashMap::new();
    widget_values.insert("14001".to_string(), 0.3); // morph
    widget_values.insert("14002".to_string(), -0.7); // morph2
    widget_values.insert("14004".to_string(), 0.8); // Amp_01
    widget_values.insert("14005".to_string(), 0.6); // Amp_02
    widget_values.insert("14007".to_string(), 12.0); // cutoff

    let preset_data = PresetData {
        name: "Default01".to_string(),
        description: None,
        widget_values,
        created_by: None,
    };

    let stats = service.save_preset_and_learn(preset_data).await.unwrap();
    println!(
        "{} {}",
        "→".green(),
        format!("Pattern learning complete: {} widgets", stats.total_widgets).cyan()
    );

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Testing pattern recognition...".yellow()
    );

    // Test suggestions for the new morph widget
    let suggestions = service
        .get_widget_value_suggestions(
            14009,
            Some("morph4".to_string()),
            Some("slider".to_string()),
        )
        .await
        .unwrap();

    println!(
        "{} {}",
        "→".green(),
        format!("Morph pattern suggestions: {}", suggestions.len()).cyan()
    );
    assert!(!suggestions.is_empty());

    // Test suggestions for the new Amp widget
    let suggestions = service
        .get_widget_value_suggestions(
            14010,
            Some("Amp_03".to_string()),
            Some("slider".to_string()),
        )
        .await
        .unwrap();

    println!(
        "{} {}",
        "→".green(),
        format!("Amp pattern suggestions: {}", suggestions.len()).cyan()
    );
    assert!(!suggestions.is_empty());

    println!("\n{}", "TEST PASSED".bold().green());
}

#[tokio::test]
async fn test_kyma_intelligence_stats() {
    control::set_override(true);

    println!("\n{}", "KYMA INTELLIGENCE STATS TEST".bold().underline());

    let temp_dir = tempdir().unwrap();
    let db_path_buf = temp_dir.path().join("test_kyma_stats");
    let db_path = db_path_buf.to_str().unwrap();

    let service = StandaloneIntelligenceService::new(db_path).unwrap();

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Building intelligence database...".yellow()
    );

    // Add multiple widgets and presets
    let widgets = vec![
        (
            15001,
            r#"{"concreteEventID": 15001, "label": "Amp_01", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
        ),
        (
            15002,
            r#"{"concreteEventID": 15002, "label": "Amp_02", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
        ),
        (
            15003,
            r#"{"concreteEventID": 15003, "label": "Gate", "minimum": 0.0, "maximum": 1.0, "displayType": "slider"}"#,
        ),
        (
            15004,
            r#"{"concreteEventID": 15004, "label": "cutoff", "minimum": -24.0, "maximum": 24.0, "displayType": "slider"}"#,
        ),
        (
            15005,
            r#"{"concreteEventID": 15005, "label": "morph", "minimum": -1.0, "maximum": 1.0, "displayType": "slider"}"#,
        ),
    ];

    for (event_id, kyma_json) in &widgets {
        service
            .cache_widget_description(*event_id, kyma_json.to_string())
            .await
            .unwrap();
    }

    // Create multiple presets
    let presets = vec![
        (
            "FuzzySparks",
            vec![("15001", 0.85), ("15002", 0.7), ("15003", 0.9)],
        ),
        (
            "Default",
            vec![("15001", 0.5), ("15002", 0.5), ("15003", 0.5)],
        ),
        (
            "SizzlingDrips",
            vec![("15001", 0.95), ("15004", -12.0), ("15005", 0.3)],
        ),
        (
            "Default01",
            vec![("15001", 0.6), ("15002", 0.4), ("15004", 3.0)],
        ),
        (
            "MassiveSparks",
            vec![("15001", 0.9), ("15003", 0.8), ("15005", -0.2)],
        ),
    ];

    for (name, widget_values) in presets {
        let mut values = HashMap::new();
        for (id, value) in widget_values {
            values.insert(id.to_string(), value);
        }

        let preset_data = PresetData {
            name: name.to_string(),
            description: None,
            widget_values: values,
            created_by: None,
        };

        service.save_preset_and_learn(preset_data).await.unwrap();
    }

    print_separator();
    println!(
        "{} {}",
        "→".green(),
        "Getting intelligence statistics...".yellow()
    );

    let stats = service.get_intelligence_stats().await.unwrap();

    println!(
        "{} {}",
        "→".green(),
        format!("Total widgets: {}", stats.total_widgets).cyan()
    );
    println!(
        "{} {}",
        "→".green(),
        format!("Total presets: {}", stats.total_presets).cyan()
    );
    println!(
        "{} {}",
        "→".green(),
        format!("Cache size: {}", stats.cache_size).cyan()
    );
    println!(
        "{} {}",
        "→".green(),
        format!("Last updated: {}", stats.last_updated).cyan()
    );

    assert!(stats.total_widgets > 0);
    assert!(stats.total_presets > 0);
    assert!(stats.cache_size > 0);
    assert!(!stats.last_updated.is_empty());

    println!("\n{}", "TEST PASSED".bold().green());
}
