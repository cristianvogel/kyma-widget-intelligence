
use widget_intelligence::*;

    use tempfile::tempdir;
    use std::fs;
    use colored::*;
    use crate::similarity_engine::{Widget, WidgetValue, Preset};
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

    fn create_kyma_preset(name: &str, widget_values: HashMap<String, f64>) -> Preset {
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

    fn print_separator() {
        println!("{}", "─".repeat(80).blue());
    }

    #[test]
    fn test_kyma_widget_persistence() -> Result<(), Box<dyn std::error::Error>> {
        control::set_override(true);

        println!("\n{}", "KYMA WIDGET PERSISTENCE TEST".bold().underline());

        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_kyma_persistence");

        println!("{} {}", "→".green(), format!("Using database path: {:?}", db_path).cyan());
        fs::create_dir_all(&db_path)?;

        print_separator();
        println!("{} {}", "→".green(), "Initializing system...".yellow());
        let mut system = PersistentWidgetSuggestionEngine::new(&db_path)?;

        // Store realistic Kyma widgets
        let kyma_widgets = vec![
            create_kyma_widget("Amp_01", 0.0, 1.0, 0.75),
            create_kyma_widget("cutoff", -24.0, 24.0, 8.5),
            create_kyma_widget("Gate", 0.0, 1.0, 0.6),
            create_kyma_widget("morph", -1.0, 1.0, 0.3),
            create_kyma_widget("sw_00", 0.0, 1.0, 0.9),
            create_kyma_widget("rate", 30.0, 90.0, 65.0),
        ];

        println!("{} {}", "→".green(), "Storing Kyma widgets...".yellow());
        for widget in kyma_widgets {
            system.store_widget(widget)?;
        }

        let stats = system.get_stats();
        println!("{} {}", "→".green(), format!("Stored {} widgets", stats.get("total_widgets").unwrap_or(&0)).cyan());

        print_separator();
        println!("{} {}", "→".green(), "Testing persistence...".yellow());
        system.flush()?;
        drop(system);

        println!("{} {}", "→".green(), "Reloading system...".yellow());
        let system2 = PersistentWidgetSuggestionEngine::new(&db_path)?;
        let stats2 = system2.get_stats();

        println!("{} {}", "→".green(), format!("Reloaded {} widgets", stats2.get("total_widgets").unwrap_or(&0)).cyan());
        assert_eq!(stats.get("total_widgets"), stats2.get("total_widgets"));

        println!("\n{}", "TEST PASSED".bold().green());
        Ok(())
    }

    #[test]
    fn test_kyma_preset_persistence() -> Result<(), Box<dyn std::error::Error>> {
        control::set_override(true);

        println!("\n{}", "KYMA PRESET PERSISTENCE TEST".bold().underline());

        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_kyma_presets");

        fs::create_dir_all(&db_path)?;
        let mut system = PersistentWidgetSuggestionEngine::new(&db_path)?;

        // Create realistic presets
        let presets = vec![
            create_kyma_preset("FuzzySparks", {
                let mut values = HashMap::new();
                values.insert("13755".to_string(), 0.85);
                values.insert("13756".to_string(), 18.0);
                values.insert("13757".to_string(), 0.7);
                values
            }),
            create_kyma_preset("Default", {
                let mut values = HashMap::new();
                values.insert("13755".to_string(), 0.5);
                values.insert("13756".to_string(), 0.0);
                values.insert("13757".to_string(), 0.5);
                values
            }),
            create_kyma_preset("SizzlingDrips", {
                let mut values = HashMap::new();
                values.insert("13755".to_string(), 0.95);
                values.insert("13756".to_string(), -12.0);
                values.insert("13757".to_string(), 0.2);
                values.insert("13758".to_string(), 0.8);
                values
            }),
            create_kyma_preset("Default01", {
                let mut values = HashMap::new();
                values.insert("13755".to_string(), 0.6);
                values.insert("13756".to_string(), 3.0);
                values.insert("13757".to_string(), 0.4);
                values
            }),
        ];

        print_separator();
        println!("{} {}", "→".green(), "Storing Kyma presets...".yellow());
        for preset in presets {
            system.store_preset(preset)?;
        }

        let stats = system.get_stats();
        println!("{} {}", "→".green(), format!("Stored {} presets", stats.get("presets_stored").unwrap_or(&0)).cyan());

        print_separator();
        println!("{} {}", "→".green(), "Testing preset persistence...".yellow());
        system.flush()?;
        drop(system);

        println!("{} {}", "→".green(), "Reloading system...".yellow());
        let system2 = PersistentWidgetSuggestionEngine::new(&db_path)?;
        let stats2 = system2.get_stats();

        println!("{} {}", "→".green(), format!("Reloaded {} presets", stats2.get("presets_stored").unwrap_or(&0)).cyan());
        assert_eq!(stats.get("presets_stored"), stats2.get("presets_stored"));

        println!("\n{}", "TEST PASSED".bold().green());
        Ok(())
    }

    #[test]
    fn test_kyma_export_import() -> Result<(), Box<dyn std::error::Error>> {
        control::set_override(true);

        println!("\n{}", "KYMA EXPORT/IMPORT TEST".bold().underline());

        let temp_dir = tempdir()?;
        let db_path1 = temp_dir.path().join("test_kyma_export");
        let db_path2 = temp_dir.path().join("test_kyma_import");

        fs::create_dir_all(&db_path1)?;
        fs::create_dir_all(&db_path2)?;

        print_separator();
        println!("{} {}", "→".green(), "Creating source database...".yellow());
        let mut system1 = PersistentWidgetSuggestionEngine::new(&db_path1)?;

        // Store realistic Kyma data
        let widgets = vec![
            create_kyma_widget("Amp_01", 0.0, 1.0, 0.8),
            create_kyma_widget("Amp_02", 0.0, 1.0, 0.6),
            create_kyma_widget("morph", -1.0, 1.0, 0.2),
            create_kyma_widget("cutoff", -24.0, 24.0, 12.0),
        ];

        for widget in widgets {
            system1.store_widget(widget)?;
        }

        let preset = create_kyma_preset("MassiveSparks", {
            let mut values = HashMap::new();
            values.insert("13755".to_string(), 0.9);
            values.insert("13756".to_string(), 15.0);
            values.insert("13757".to_string(), 0.3);
            values
        });
        system1.store_preset(preset)?;

        print_separator();
        println!("{} {}", "→".green(), "Exporting data...".yellow());
        let export_data = system1.export_data()?;
        println!("{} {}", "→".green(), format!("Exported {} widgets, {} presets",
                                               export_data.widgets.len(), export_data.presets.len()).cyan());

        print_separator();
        println!("{} {}", "→".green(), "Importing to new database...".yellow());
        let mut system2 = PersistentWidgetSuggestionEngine::new(&db_path2)?;
        system2.import_data(export_data)?;

        let stats1 = system1.get_stats();
        let stats2 = system2.get_stats();

        println!("{} {}", "→".green(), format!("Source: {} widgets, {} presets",
                                               stats1.get("total_widgets").unwrap_or(&0),
                                               stats1.get("presets_stored").unwrap_or(&0)).cyan());

        println!("{} {}", "→".green(), format!("Destination: {} widgets, {} presets",
                                               stats2.get("total_widgets").unwrap_or(&0),
                                               stats2.get("presets_stored").unwrap_or(&0)).cyan());

        assert_eq!(stats1.get("total_widgets"), stats2.get("total_widgets"));
        assert_eq!(stats1.get("presets_stored"), stats2.get("presets_stored"));

        println!("\n{}", "TEST PASSED".bold().green());
        Ok(())
    }

    #[test]
    fn test_kyma_suggestions_persistence() -> Result<(), Box<dyn std::error::Error>> {
        control::set_override(true);

        println!("\n{}", "KYMA SUGGESTIONS PERSISTENCE TEST".bold().underline());

        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_kyma_suggestions");

        fs::create_dir_all(&db_path)?;
        let mut system = PersistentWidgetSuggestionEngine::new(&db_path)?;

        // Store training data
        let training_widgets = vec![
            create_kyma_widget("Amp_01", 0.0, 1.0, 0.8),
            create_kyma_widget("Amp_02", 0.0, 1.0, 0.7),
            create_kyma_widget("Amp_03", 0.0, 1.0, 0.9),
            create_kyma_widget("sw_00", 0.0, 1.0, 0.6),
            create_kyma_widget("sw_01", 0.0, 1.0, 0.4),
            create_kyma_widget("morph", -1.0, 1.0, 0.3),
            create_kyma_widget("morph2", -1.0, 1.0, -0.2),
        ];

        print_separator();
        println!("{} {}", "→".green(), "Storing training widgets...".yellow());
        for widget in training_widgets {
            system.store_widget(widget)?;
        }

        print_separator();
        println!("{} {}", "→".green(), "Testing suggestions...".yellow());

        // Test Amp series suggestions
        let test_widget = Widget {
            label: Some("Amp_04".to_string()),
            minimum: Some(0.0),
            maximum: Some(1.0),
            ..Default::default()
        };

        let suggestions = system.get_suggestions(&test_widget, 3);
        println!("{} {}", "→".green(), format!("Got {} suggestions for Amp_04", suggestions.len()).cyan());

        for suggestion in &suggestions {
            println!("  • {} (confidence: {:.2})",
                     suggestion.widget.label.as_deref().unwrap_or("Unknown").cyan(),
                     suggestion.confidence.to_string().yellow());
        }

        print_separator();
        println!("{} {}", "→".green(), "Testing persistence of suggestions...".yellow());
        system.flush()?;
        drop(system);

        let system2 = PersistentWidgetSuggestionEngine::new(&db_path)?;
        let suggestions2 = system2.get_suggestions(&test_widget, 3);

        println!("{} {}", "→".green(), format!("After reload: {} suggestions", suggestions2.len()).cyan());
        assert_eq!(suggestions.len(), suggestions2.len());

        println!("\n{}", "TEST PASSED".bold().green());
        Ok(())
    }