use std::collections::HashMap;
use widget_intelligence::*;

use serde_json::{json, Value};
    use colored::*;

    fn print_separator() {
        println!("\n{}", "=".repeat(80).bright_black());
    }

    #[test]
    fn test_kyma_extractor_basic() {
        // Force enable colors for tests
        colored::control::set_override(true);
        println!("\n{}", "KYMA EXTRACTOR BASIC TEST".bold().underline());

        let mut extractor = KymaWidgetExtractor::new();

        let kyma_data = json!({
            "concreteEventID": 100,
            "label": "Master Volume",
            "minimum": 0.0,
            "maximum": 127.0,
            "displayType": "slider",
            "isGenerated": false
        });

        println!("{} {}", "→".green(), "Input Kyma data:".yellow());
        println!("{} {}", " ".repeat(4), format!("{:#}", kyma_data).cyan());

        let data_map: HashMap<String, Value> = serde_json::from_value(kyma_data).unwrap();
        extractor.cache_widget_description(data_map);

        println!("\n{} {}", "→".green(), "Creating training widget...".yellow());
        let widget = extractor.create_training_widget(100, 95.0);
        assert!(widget.is_some());

        let widget = widget.unwrap();
        println!("{} {}", "→".green(), "Extracted widget:".yellow());
        println!("{} {}", " ".repeat(4), format!("{:?}", widget).cyan());

        assert_eq!(widget.label, Some("Master Volume".to_string()));
        assert_eq!(widget.minimum, Some(0.0));
        assert_eq!(widget.maximum, Some(127.0));
        assert_eq!(widget.current_value, Some(95.0));
        assert_eq!(widget.display_type, Some("slider".to_string()));
        assert_eq!(widget.is_generated, Some(false));

        println!("\n{}", "✓ Basic extraction test passed".green());
    }

    #[test]
    fn test_widget_metadata() {
        println!("\n{}", "WIDGET METADATA TEST".bold().underline());

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

        println!("{} {}", "→".green(), "Testing metadata:".yellow());
        println!("{} {}", " ".repeat(4), format!("{:?}", metadata).cyan());

        print_separator();

        // Test value validation
        println!("{} {}", "→".green(), "Testing value validation:".yellow());
        let test_values = vec![
            (50.0, true),
            (150.0, false),
            (-10.0, false)
        ];

        for (value, expected) in test_values {
            let result = metadata.is_valid_value(value);
            println!("{} Value {}: {} ({})",
                     " ".repeat(4),
                     value,
                     if result { "valid".green() } else { "invalid".red() },
                     if result == expected { "✓".green() } else { "✗".red() }
            );
            assert_eq!(result, expected);
        }

        print_separator();

        // Test value normalization
        println!("{} {}", "→".green(), "Testing value normalization:".yellow());
        assert_eq!(metadata.normalize_value(50.0), Some(0.5));
        assert_eq!(metadata.denormalize_value(0.5), Some(50.0));
        println!("{} Normalization test passed", "✓".green());

        print_separator();

        // Test widget conversion
        println!("{} {}", "→".green(), "Testing widget conversion:".yellow());
        let widget = metadata.to_widget(75.0);
        println!("{} {}", " ".repeat(4), format!("{:?}", widget).cyan());
        assert_eq!(widget.current_value, Some(75.0));
        assert_eq!(widget.label, Some("Test Widget".to_string()));

        println!("\n{}", "✓ All metadata tests passed".green());
    }

    #[test]
    fn test_json_parsing() {
        println!("\n{}", "JSON PARSING TEST".bold().underline());

        let json_str = r#"{"concreteEventID": 13755, "label": "Amp_01", "minimum": 0, "maximum": 1.0}"#;
        println!("{} {}", "→".green(), "Testing JSON string:".yellow());
        println!("{} {}", " ".repeat(4), json_str.cyan());

        let parsed = KymaWidgetExtractor::parse_kyma_json_string(json_str);
        assert!(parsed.is_ok());

        let data = parsed.unwrap();
        let validation = KymaWidgetExtractor::validate_kyma_data(&data);
        println!("{} Validation result: {}",
                 "→".green(),
                 if validation.is_ok() { "valid ✓".green() } else { "invalid ✗".red() }
        );
        assert!(validation.is_ok());

        println!("\n{}", "✓ JSON parsing test passed".green());
    }

    #[test]
    fn test_invalid_json() {
        println!("\n{}", "INVALID JSON TEST".bold().underline());

        let json_str = r#"{"label": "Amp_01"}"#;
        println!("{} {}", "→".green(), "Testing invalid JSON:".yellow());
        println!("{} {}", " ".repeat(4), json_str.cyan());

        let parsed = KymaWidgetExtractor::parse_kyma_json_string(json_str);
        assert!(parsed.is_ok());

        let data = parsed.unwrap();
        let validation = KymaWidgetExtractor::validate_kyma_data(&data);
        println!("{} Validation result: {}",
                 "→".green(),
                 if validation.is_err() { "invalid (as expected) ✓".green() } else { "valid (unexpected) ✗".red() }
        );
        assert!(validation.is_err());

        println!("\n{}", "✓ Invalid JSON handling test passed".green());
    }

    #[test]
    fn test_extract_all_widgets() {
        println!("\n{}", "WIDGET EXTRACTION TEST".bold().underline());

        let mut extractor = KymaWidgetExtractor::new();

        // Create test data
        let test_widgets = vec![
            ("Amp_01", 0.0, 1.0, 13755),
            ("Pan", -1.0, 1.0, 13756)
        ];

        println!("{} {}", "→".green(), "Caching test widgets:".yellow());
        for (label, min, max, id) in &test_widgets {
            let kyma_data = json!({
                "concreteEventID": id,
                "label": label,
                "minimum": min,
                "maximum": max
            });
            println!("{} Widget {}: {} [{}, {}]",
                     " ".repeat(4),
                     id,
                     label.cyan(),
                     min,
                     max
            );

            let data_map: HashMap<String, Value> = serde_json::from_value(kyma_data).unwrap();
            extractor.cache_widget_description(data_map);
        }

        print_separator();

        // Test extraction
        println!("{} {}", "→".green(), "Testing widget extraction:".yellow());
        let mut values = HashMap::new();
        values.insert(13755, 0.5);  // Amp_01
        values.insert(13756, 0.0);  // Pan
        values.insert(99999, 50.0); // Non-existent widget

        let widgets = extractor.extract_all_widgets_with_values(&values);
        println!("{} Extracted {} widgets", "→".green(), widgets.len());

        for widget in &widgets {
            println!("{} {}", " ".repeat(4), format!("{:?}", widget).cyan());
        }

        assert_eq!(widgets.len(), 2);

        println!("\n{}", "✓ Widget extraction test passed".green());
    }
