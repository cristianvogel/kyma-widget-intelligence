use crate::similarity_engine::{Preset, Suggestion, Widget, WidgetRecord, WidgetSuggestionEngine};
use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize}; // Keep temporarily for migration
use sled::{Db, Tree};
use std::collections::HashMap;

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

impl From<bincode::error::EncodeError> for SledPersistenceError {
    fn from(err: bincode::error::EncodeError) -> Self {
        SledPersistenceError::SerializationError(err.to_string())
    }
}

impl From<bincode::error::DecodeError> for SledPersistenceError {
    fn from(err: bincode::error::DecodeError) -> Self {
        SledPersistenceError::DeserializationError(err.to_string())
    }
}

impl std::fmt::Display for SledPersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SledPersistenceError::DatabaseError(e) => write!(f, "Database error: {e}"),
            SledPersistenceError::SerializationError(e) => write!(f, "Serialization error: {e}"),
            SledPersistenceError::DeserializationError(e) => {
                write!(f, "Deserialization error: {e}")
            }
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
        let widgets_tree = db.open_tree("widgets_v1")?; // New tree for bincode format
        let presets_tree = db.open_tree("presets_v1")?; // New tree for bincode format
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
        let value = bincode::encode_to_vec(record, bincode::config::standard())?;

        self.widgets_tree.insert(key, value)?;
        Ok(())
    }

    pub fn load_all_widgets(&self) -> Result<Vec<WidgetRecord>, SledPersistenceError> {
        let mut records = Vec::new();

        for result in self.widgets_tree.iter() {
            let (_key, value) = result?;
            match bincode::decode_from_slice(&value, bincode::config::standard()) {
                Ok((record, _)) => records.push(record),
                Err(e) => {
                    log::warn!("Failed to decode widget record with bincode: {e}");
                }
            }
        }

        Ok(records)
    }

    pub fn store_preset(&self, preset: &Preset) -> Result<(), SledPersistenceError> {
        let key = preset.name.as_bytes();
        let value = bincode::encode_to_vec(preset, bincode::config::standard())?;

        self.presets_tree.insert(key, value)?;
        Ok(())
    }

    pub fn load_all_presets(&self) -> Result<Vec<Preset>, SledPersistenceError> {
        let mut presets = Vec::new();

        for result in self.presets_tree.iter() {
            let (_key, value) = result?;
            match bincode::decode_from_slice(&value, bincode::config::standard()) {
                Ok((preset, _)) => presets.push(preset),
                Err(e) => {
                    log::warn!("Failed to decode preset with bincode: {e}");
                }
            }
        }

        Ok(presets)
    }

    pub fn store_metadata(&self, key: &str, value: &str) -> Result<(), SledPersistenceError> {
        self.metadata_tree
            .insert(key.as_bytes(), value.as_bytes())?;
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
        // Note: sled doesn't have a direct compact method, this clears the database
        // In a real implementation, you might want to implement a proper compaction
        log::warn!("Compact operation not implemented for sled database");
        Ok(())
    }

    pub fn size_on_disk(&self) -> Result<u64, SledPersistenceError> {
        Ok(self.db.size_on_disk()?)
    }
}

#[derive(Debug)]
pub struct MigrationStatus {
    pub legacy_widgets: usize,
    pub legacy_presets: usize,
    pub new_widgets: usize,
    pub new_presets: usize,
    pub migration_needed: bool,
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
                log::info!(
                    "Loaded {} widget records from database",
                    engine.records.len()
                );
            }
            Err(e) => {
                log::warn!("Failed to load widgets from database: {e}");
            }
        }

        match persistence.load_all_presets() {
            Ok(presets) => {
                engine.presets = presets;
                log::info!("Loaded {} presets from database", engine.presets.len());
            }
            Err(e) => {
                log::warn!("Failed to load presets from database: {e}");
            }
        }

        if let Some(next_id) = persistence.load_metadata("next_id").ok().flatten() {
            if let Ok(id) = next_id.parse::<u64>() {
                engine.next_id = id;
            }
        }

        Ok(Self {
            engine,
            persistence,
        })
    }

    pub fn store_widget(&mut self, widget: Widget) -> Result<(), SledPersistenceError> {
        let initial_count = self.engine.records.len();
        self.engine.store_widget(widget);

        if self.engine.records.len() > initial_count {
            if let Some(record) = self.engine.records.last() {
                self.persistence.store_widget(record)?;
                self.persistence
                    .store_metadata("next_id", &self.engine.next_id.to_string())?;
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

    pub fn get_suggestions(
        &self,
        partial_widget: &Widget,
        max_suggestions: usize,
    ) -> Vec<Suggestion> {
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

        self.persistence
            .store_metadata("next_id", &self.engine.next_id.to_string())?;
        self.flush()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Encode, Decode, Serialize, Deserialize)]
pub struct ExportData {
    pub widgets: Vec<WidgetRecord>,
    pub presets: Vec<Preset>,
    pub display_types: HashMap<String, u64>,
    pub next_id: u64,
}
