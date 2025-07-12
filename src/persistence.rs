use sled::{Db, Tree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::similarity_engine::{WidgetRecord, Preset, WidgetSuggestionEngine, Widget, Suggestion};

#[derive(Debug)]
pub enum SledPersistenceError {
    DatabaseError(sled::Error),
    SerializationError(String),
    DeserializationError(String),
}

impl From<sled::Error> for SledPersistenceError {
    fn from(err: sled::Error) -> Self {
        SledPersistenceError::DatabaseError(err)
    }
}

impl std::fmt::Display for SledPersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SledPersistenceError::DatabaseError(e) => write!(f, "Database error: {}", e),
            SledPersistenceError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            SledPersistenceError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
        }
    }
}

impl std::error::Error for SledPersistenceError {}

pub struct SledPersistenceManager {
    db: Db,
    widgets_tree: Tree,
    presets_tree: Tree,
    metadata_tree: Tree,
}

impl SledPersistenceManager {
    pub fn new<P: AsRef<std::path::Path>>(db_path: P) -> Result<Self, SledPersistenceError> {
        let db = sled::open(db_path)?;
        let widgets_tree = db.open_tree("widgets")?;
        let presets_tree = db.open_tree("presets")?;
        let metadata_tree = db.open_tree("metadata")?;

        Ok(Self {
            db,
            widgets_tree,
            presets_tree,
            metadata_tree,
        })
    }

    pub fn store_widget(&self, record: &WidgetRecord) -> Result<(), SledPersistenceError> {
        let key = record.id.to_be_bytes();
        let value = bincode::serialize(record)
            .map_err(|e| SledPersistenceError::SerializationError(e.to_string()))?;
        
        self.widgets_tree.insert(key, value)?;
        Ok(())
    }

    pub fn load_all_widgets(&self) -> Result<Vec<WidgetRecord>, SledPersistenceError> {
        let mut records = Vec::new();
        
        for result in self.widgets_tree.iter() {
            let (_key, value) = result?;
            let record: WidgetRecord = bincode::deserialize(&value)
                .map_err(|e| SledPersistenceError::DeserializationError(e.to_string()))?;
            records.push(record);
        }
        
        Ok(records)
    }

    pub fn store_preset(&self, preset: &Preset) -> Result<(), SledPersistenceError> {
        let key = preset.name.as_bytes();
        let value = bincode::serialize(preset)
            .map_err(|e| SledPersistenceError::SerializationError(e.to_string()))?;
        
        self.presets_tree.insert(key, value)?;
        Ok(())
    }

    pub fn load_all_presets(&self) -> Result<Vec<Preset>, SledPersistenceError> {
        let mut presets = Vec::new();
        
        for result in self.presets_tree.iter() {
            let (_key, value) = result?;
            let preset: Preset = bincode::deserialize(&value)
                .map_err(|e| SledPersistenceError::DeserializationError(e.to_string()))?;
            presets.push(preset);
        }
        
        Ok(presets)
    }

    pub fn store_metadata(&self, key: &str, value: &str) -> Result<(), SledPersistenceError> {
        self.metadata_tree.insert(key.as_bytes(), value.as_bytes())?;
        Ok(())
    }

    pub fn load_metadata(&self, key: &str) -> Result<Option<String>, SledPersistenceError> {
        if let Some(value) = self.metadata_tree.get(key.as_bytes())? {
            let string_value = String::from_utf8_lossy(&value).to_string();
            Ok(Some(string_value))
        } else {
            Ok(None)
        }
    }

    pub fn flush(&self) -> Result<(), SledPersistenceError> {
        self.db.flush()?;
        Ok(())
    }

    pub fn compact(&self) -> Result<(), SledPersistenceError> {
        self.db.clear()?;
        Ok(())
    }

    pub fn size_on_disk(&self) -> Result<u64, SledPersistenceError> {
        Ok(self.db.size_on_disk()?)
    }
}

pub struct PersistentWidgetSuggestionEngine {
    pub engine: WidgetSuggestionEngine,
    pub persistence: SledPersistenceManager,
}

impl PersistentWidgetSuggestionEngine {
    pub fn new<P: AsRef<std::path::Path>>(db_path: P) -> Result<Self, SledPersistenceError> {
        let persistence = SledPersistenceManager::new(db_path)?;
        let mut engine = WidgetSuggestionEngine::new();
        
        match persistence.load_all_widgets() {
            Ok(widgets) => {
                engine.records = widgets;
                log::info!("Loaded {} widget records from database", engine.records.len());
            }
            Err(e) => {
                log::warn!("Failed to load widgets from database: {}", e);
            }
        }
        
        match persistence.load_all_presets() {
            Ok(presets) => {
                engine.presets = presets;
                log::info!("Loaded {} presets from database", engine.presets.len());
            }
            Err(e) => {
                log::warn!("Failed to load presets from database: {}", e);
            }
        }

        if let Some(next_id) = persistence.load_metadata("next_id").ok().flatten() {
            if let Ok(id) = next_id.parse::<u64>() {
                engine.next_id = id;
            }
        }
        
        Ok(Self { engine, persistence })
    }

    pub fn store_widget(&mut self, widget: Widget) -> Result<(), SledPersistenceError> {
        let initial_count = self.engine.records.len();
        self.engine.store_widget(widget);
        
        if self.engine.records.len() > initial_count {
            if let Some(record) = self.engine.records.last() {
                self.persistence.store_widget(record)?;
                self.persistence.store_metadata("next_id", &self.engine.next_id.to_string())?;
            }
        } else if let Some(record) = self.engine.records.iter().find(|r| r.frequency > 1) {
            self.persistence.store_widget(record)?;
        }
        
        Ok(())
    }

    pub fn store_preset(&mut self, preset: Preset) -> Result<(), SledPersistenceError> {
        self.engine.store_preset(preset.clone());
        self.persistence.store_preset(&preset)?;
        Ok(())
    }

    pub fn get_suggestions(&self, partial_widget: &Widget, max_suggestions: usize) -> Vec<Suggestion> {
        self.engine.get_suggestions(partial_widget, max_suggestions)
    }

    pub fn get_preset_insights(&self, widget: &Widget) -> Option<String> {
        self.engine.get_preset_insights(widget)
    }

    pub fn get_stats(&self) -> HashMap<String, usize> {
        self.engine.get_stats()
    }

    pub fn flush(&self) -> Result<(), SledPersistenceError> {
        self.persistence.flush()
    }

    pub fn compact(&self) -> Result<(), SledPersistenceError> {
        self.persistence.compact()
    }

    pub fn size_on_disk(&self) -> Result<u64, SledPersistenceError> {
        self.persistence.size_on_disk()
    }

    pub fn export_data(&self) -> Result<ExportData, SledPersistenceError> {
        Ok(ExportData {
            widgets: self.engine.records.clone(),
            presets: self.engine.presets.clone(),
            display_types: self.engine.display_types.clone(),
            next_id: self.engine.next_id,
        })
    }

    pub fn import_data(&mut self, data: ExportData) -> Result<(), SledPersistenceError> {
        for record in &data.widgets {
            self.persistence.store_widget(record)?;
        }
        
        for preset in &data.presets {
            self.persistence.store_preset(preset)?;
        }
        
        self.engine.records = data.widgets;
        self.engine.presets = data.presets;
        self.engine.display_types = data.display_types;
        self.engine.next_id = data.next_id;
        
        self.persistence.store_metadata("next_id", &self.engine.next_id.to_string())?;
        self.flush()?;
        
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub widgets: Vec<WidgetRecord>,
    pub presets: Vec<Preset>,
    pub display_types: HashMap<String, u64>,
    pub next_id: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;
    use crate::similarity_engine::Widget;

    #[test]
    fn test_persistence_basic_operations() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path = temp_dir.path().join("test_persistence_basic");
        
        // Ensure the directory exists
        fs::create_dir_all(&db_path)?;
        
        let mut system = PersistentWidgetSuggestionEngine::new(&db_path)?;
        
        let widget = Widget {
            label: Some("Test Volume".to_string()),
            minimum: Some(0.0),
            maximum: Some(127.0),
            current_value: Some(64.0),
            is_generated: Some(false),
            display_type: Some("slider".to_string()),
        };
        
        system.store_widget(widget)?;
        
        let stats = system.get_stats();
        assert_eq!(stats.get("total_widgets"), Some(&1));
        
        // Ensure changes are flushed to disk
        system.flush()?;
        
        // Explicitly drop the first system to release the database
        drop(system);
        
        let system2 = PersistentWidgetSuggestionEngine::new(&db_path)?;
        let stats2 = system2.get_stats();
        assert_eq!(stats2.get("total_widgets"), Some(&1));
        
        // Clean up by dropping the system and removing the temp directory
        drop(system2);
        fs::remove_dir_all(&db_path)?;
        
        Ok(())
    }

    #[test]
    fn test_export_import() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = tempdir()?;
        let db_path1 = temp_dir.path().join("test_export_1");
        let db_path2 = temp_dir.path().join("test_export_2");
        
        // Ensure directories exist
        fs::create_dir_all(&db_path1)?;
        fs::create_dir_all(&db_path2)?;
        
        let mut system1 = PersistentWidgetSuggestionEngine::new(&db_path1)?;
        
        let widget = Widget {
            label: Some("Master Volume".to_string()),
            minimum: Some(0.0),
            maximum: Some(100.0),
            current_value: Some(75.0),
            is_generated: Some(false),
            display_type: Some("knob".to_string()),
        };
        
        system1.store_widget(widget)?;
        system1.flush()?;
        
        let stats1 = system1.get_stats();
        let export_data = system1.export_data()?;

        // Drop system1 before creating system2
        drop(system1);

        let mut system2 = PersistentWidgetSuggestionEngine::new(&db_path2)?;
        system2.import_data(export_data)?;

        let stats2 = system2.get_stats();

        assert_eq!(stats1.get("total_widgets"), stats2.get("total_widgets"));
        
        // Clean up
        drop(system2);
        fs::remove_dir_all(&db_path1)?;
        fs::remove_dir_all(&db_path2)?;
        
        Ok(())
    }
}