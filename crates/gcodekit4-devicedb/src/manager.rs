use crate::model::DeviceProfile;
use crate::traits::DeviceProfileProvider;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, RwLock, Mutex};

#[derive(Debug, Clone)]
pub struct DeviceManager {
    profiles: Arc<RwLock<HashMap<String, DeviceProfile>>>,
    active_profile_id: Arc<RwLock<Option<String>>>,
    config_path: PathBuf,
    file_lock: Arc<Mutex<()>>,
}

impl DeviceManager {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            profiles: Arc::new(RwLock::new(HashMap::new())),
            active_profile_id: Arc::new(RwLock::new(None)),
            config_path,
            file_lock: Arc::new(Mutex::new(())),
        }
    }

    pub fn load(&self) -> Result<()> {
        if !self.config_path.exists() {
            // Create default profile if file doesn't exist
            let default_profile = DeviceProfile::default();
            self.save_profile(default_profile.clone())?;
            self.set_active_profile(&default_profile.id)?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.config_path)
            .context("Failed to read device profiles file")?;
        
        let data: serde_json::Value = serde_json::from_str(&content)
            .context("Failed to parse device profiles JSON")?;

        let mut profiles_map = HashMap::new();
        
        if let Some(profiles_array) = data.get("profiles").and_then(|v| v.as_array()) {
            for p in profiles_array {
                let profile: DeviceProfile = serde_json::from_value(p.clone())?;
                profiles_map.insert(profile.id.clone(), profile);
            }
        }

        let active_id = data.get("active_id").and_then(|v| v.as_str()).map(String::from);

        {
            let mut profiles_lock = self.profiles.write().unwrap();
            *profiles_lock = profiles_map;
            
            let mut active_lock = self.active_profile_id.write().unwrap();
            *active_lock = active_id;
        }

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        // Acquire file lock to prevent concurrent writes
        let _file_guard = self.file_lock.lock().unwrap();

        let profiles_lock = self.profiles.read().unwrap();
        let active_lock = self.active_profile_id.read().unwrap();

        let profiles_vec: Vec<&DeviceProfile> = profiles_lock.values().collect();
        
        let data = serde_json::json!({
            "profiles": profiles_vec,
            "active_id": *active_lock
        });

        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&data)?;
        fs::write(&self.config_path, content)
            .context("Failed to write device profiles file")?;

        Ok(())
    }

    pub fn get_profile(&self, id: &str) -> Option<DeviceProfile> {
        self.profiles.read().unwrap().get(id).cloned()
    }

    pub fn get_all_profiles(&self) -> Vec<DeviceProfile> {
        self.profiles.read().unwrap().values().cloned().collect()
    }

    pub fn save_profile(&self, profile: DeviceProfile) -> Result<()> {
        {
            let mut lock = self.profiles.write().unwrap();
            lock.insert(profile.id.clone(), profile);
        }
        self.save()
    }

    pub fn delete_profile(&self, id: &str) -> Result<()> {
        {
            let mut lock = self.profiles.write().unwrap();
            lock.remove(id);
        }
        
        // If active profile was deleted, clear active selection
        {
            let mut active_lock = self.active_profile_id.write().unwrap();
            if let Some(active) = &*active_lock {
                if active == id {
                    *active_lock = None;
                }
            }
        }
        
        self.save()
    }

    pub fn set_active_profile(&self, id: &str) -> Result<()> {
        let exists = self.profiles.read().unwrap().contains_key(id);
        if exists {
            {
                let mut lock = self.active_profile_id.write().unwrap();
                *lock = Some(id.to_string());
            }
            self.save()?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Profile not found"))
        }
    }

    pub fn get_active_profile(&self) -> Option<DeviceProfile> {
        let active_id = self.active_profile_id.read().unwrap().clone()?;
        self.get_profile(&active_id)
    }
}

impl DeviceProfileProvider for DeviceManager {
    fn get_active_profile(&self) -> Option<DeviceProfile> {
        self.get_active_profile()
    }

    fn get_profile(&self, id: &str) -> Option<DeviceProfile> {
        self.get_profile(id)
    }
}
