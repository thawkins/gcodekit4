//! # Design Template Management Module
//!
//! Provides a comprehensive design template system for saving, organizing, and quickly
//! creating designs from templates.
//!
//! Features:
//! - Save current design as template with metadata
//! - Template browser with search and filtering
//! - Template categories and organization
//! - Favorite templates for quick access
//! - Community template sharing format
//! - Template versioning and persistence

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

/// Template categories for organizing designs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Mechanical parts and components
    Mechanical,
    /// Decorative or artistic designs
    Decorative,
    /// Signage and text-based designs
    Signage,
    /// PCB and electronics-related designs
    Electronics,
    /// Everyday household items
    Household,
    /// Educational and example designs
    Educational,
    /// User-defined custom category
    Custom,
}

impl TemplateCategory {
    /// Get category as string
    pub fn as_str(&self) -> &'static str {
        match self {
            TemplateCategory::Mechanical => "mechanical",
            TemplateCategory::Decorative => "decorative",
            TemplateCategory::Signage => "signage",
            TemplateCategory::Electronics => "electronics",
            TemplateCategory::Household => "household",
            TemplateCategory::Educational => "educational",
            TemplateCategory::Custom => "custom",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mechanical" => Some(TemplateCategory::Mechanical),
            "decorative" => Some(TemplateCategory::Decorative),
            "signage" => Some(TemplateCategory::Signage),
            "electronics" => Some(TemplateCategory::Electronics),
            "household" => Some(TemplateCategory::Household),
            "educational" => Some(TemplateCategory::Educational),
            "custom" => Some(TemplateCategory::Custom),
            _ => None,
        }
    }
}

/// Design template with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignTemplate {
    /// Unique template identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Category for organization
    pub category: TemplateCategory,
    /// Template version (semantic versioning)
    pub version: String,
    /// Author name
    pub author: String,
    /// Creation timestamp
    pub created_at: String,
    /// Last modified timestamp
    pub modified_at: String,
    /// Base64-encoded thumbnail image
    pub thumbnail: Option<String>,
    /// Template tags for search
    pub tags: Vec<String>,
    /// Is this template marked as favorite
    pub is_favorite: bool,
    /// Design data (JSON string format)
    pub design_data: String,
    /// License information
    pub license: String,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl DesignTemplate {
    /// Create new design template
    pub fn new(
        id: String,
        name: String,
        description: String,
        category: TemplateCategory,
        author: String,
        design_data: String,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id,
            name,
            description,
            category,
            version: "1.0.0".to_string(),
            author,
            created_at: now.clone(),
            modified_at: now,
            thumbnail: None,
            tags: Vec::new(),
            is_favorite: false,
            design_data,
            license: "CC0".to_string(),
            metadata: HashMap::new(),
        }
    }

    /// Add tag to template
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Remove tag from template
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        if let Some(pos) = self.tags.iter().position(|t| t == tag) {
            self.tags.remove(pos);
            true
        } else {
            false
        }
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
    }

    /// Set favorite status
    pub fn set_favorite(&mut self, favorite: bool) {
        self.is_favorite = favorite;
    }

    /// Update design data and modification time
    pub fn update_design(&mut self, design_data: String) {
        self.design_data = design_data;
        self.modified_at = Utc::now().to_rfc3339();
    }

    /// Set version
    pub fn set_version(&mut self, version: String) {
        self.version = version;
    }

    /// Set thumbnail
    pub fn set_thumbnail(&mut self, thumbnail: String) {
        self.thumbnail = Some(thumbnail);
    }

    /// Matches search query
    pub fn matches_search(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.name.to_lowercase().contains(&q)
            || self.description.to_lowercase().contains(&q)
            || self.tags.iter().any(|t| t.to_lowercase().contains(&q))
            || self.author.to_lowercase().contains(&q)
    }
}

/// Template library for managing designs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignTemplateLibrary {
    /// Map of template ID to template
    templates: HashMap<String, DesignTemplate>,
    /// Last sync time
    pub last_sync: Option<String>,
}

impl DesignTemplateLibrary {
    /// Create new template library
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            last_sync: None,
        }
    }

    /// Add template to library
    pub fn add_template(&mut self, template: DesignTemplate) -> Result<()> {
        if self.templates.contains_key(&template.id) {
            return Err(anyhow!("Template with ID '{}' already exists", template.id));
        }
        self.templates.insert(template.id.clone(), template);
        Ok(())
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&DesignTemplate> {
        self.templates.get(id)
    }

    /// Get mutable template by ID
    pub fn get_template_mut(&mut self, id: &str) -> Option<&mut DesignTemplate> {
        self.templates.get_mut(id)
    }

    /// Remove template by ID
    pub fn remove_template(&mut self, id: &str) -> Option<DesignTemplate> {
        self.templates.remove(id)
    }

    /// Update existing template
    pub fn update_template(&mut self, id: &str, template: DesignTemplate) -> Result<()> {
        if !self.templates.contains_key(id) {
            return Err(anyhow!("Template with ID '{}' not found", id));
        }
        self.templates.insert(id.to_string(), template);
        Ok(())
    }

    /// Get all templates
    pub fn list_all(&self) -> Vec<&DesignTemplate> {
        self.templates.values().collect()
    }

    /// Get templates by category
    pub fn list_by_category(&self, category: TemplateCategory) -> Vec<&DesignTemplate> {
        self.templates
            .values()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Get favorite templates
    pub fn list_favorites(&self) -> Vec<&DesignTemplate> {
        self.templates
            .values()
            .filter(|t| t.is_favorite)
            .collect()
    }

    /// Search templates by query
    pub fn search(&self, query: &str) -> Vec<&DesignTemplate> {
        self.templates
            .values()
            .filter(|t| t.matches_search(query))
            .collect()
    }

    /// Search by multiple criteria
    pub fn search_advanced(
        &self,
        query: Option<&str>,
        category: Option<TemplateCategory>,
        tags: Option<&[String]>,
        favorites_only: bool,
    ) -> Vec<&DesignTemplate> {
        self.templates
            .values()
            .filter(|t| {
                if let Some(q) = query {
                    if !t.matches_search(q) {
                        return false;
                    }
                }

                if let Some(cat) = category {
                    if t.category != cat {
                        return false;
                    }
                }

                if let Some(tag_list) = tags {
                    if !tag_list.iter().any(|tag| t.tags.contains(tag)) {
                        return false;
                    }
                }

                if favorites_only && !t.is_favorite {
                    return false;
                }

                true
            })
            .collect()
    }

    /// Get template count
    pub fn count(&self) -> usize {
        self.templates.len()
    }

    /// Clear all templates
    pub fn clear(&mut self) {
        self.templates.clear();
    }

    /// Get all template IDs
    pub fn get_ids(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Check if template exists
    pub fn exists(&self, id: &str) -> bool {
        self.templates.contains_key(id)
    }

    /// Get categories in use
    pub fn get_categories(&self) -> Vec<TemplateCategory> {
        let mut cats: Vec<TemplateCategory> = self.templates
            .values()
            .map(|t| t.category)
            .collect();
        cats.sort_by_key(|c| c.as_str());
        cats.dedup();
        cats
    }

    /// Get all tags in use
    pub fn get_all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = self.templates
            .values()
            .flat_map(|t| t.tags.clone())
            .collect();
        tags.sort();
        tags.dedup();
        tags
    }

    /// Update last sync time
    pub fn set_sync_time(&mut self) {
        self.last_sync = Some(Utc::now().to_rfc3339());
    }
}

impl Default for DesignTemplateLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Template persistence manager
pub struct TemplatePersistence;

impl TemplatePersistence {
    /// Save template library to JSON file
    pub fn save(library: &DesignTemplateLibrary, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(library)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load template library from JSON file
    pub fn load(path: &Path) -> Result<DesignTemplateLibrary> {
        if !path.exists() {
            return Ok(DesignTemplateLibrary::new());
        }
        let content = std::fs::read_to_string(path)?;
        let library = serde_json::from_str(&content)?;
        Ok(library)
    }

    /// Save single template
    pub fn save_template(template: &DesignTemplate, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(template)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load single template
    pub fn load_template(path: &Path) -> Result<DesignTemplate> {
        let content = std::fs::read_to_string(path)?;
        let template = serde_json::from_str(&content)?;
        Ok(template)
    }

    /// Export template for sharing
    pub fn export_shareable(template: &DesignTemplate, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(template)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Import shared template
    pub fn import_shareable(path: &Path) -> Result<DesignTemplate> {
        let content = std::fs::read_to_string(path)?;
        let template = serde_json::from_str(&content)?;
        Ok(template)
    }
}

/// Template manager with persistence
pub struct TemplateManager {
    library: DesignTemplateLibrary,
    storage_path: PathBuf,
}

impl TemplateManager {
    /// Create new template manager
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        let library = TemplatePersistence::load(&storage_path)?;
        Ok(Self {
            library,
            storage_path,
        })
    }

    /// Add template and save
    pub fn add_template(&mut self, template: DesignTemplate) -> Result<()> {
        self.library.add_template(template)?;
        self.save()?;
        Ok(())
    }

    /// Remove template and save
    pub fn remove_template(&mut self, id: &str) -> Result<()> {
        self.library.remove_template(id)
            .ok_or_else(|| anyhow!("Template not found"))?;
        self.save()?;
        Ok(())
    }

    /// Update template and save
    pub fn update_template(&mut self, id: &str, template: DesignTemplate) -> Result<()> {
        self.library.update_template(id, template)?;
        self.save()?;
        Ok(())
    }

    /// Get template by ID
    pub fn get_template(&self, id: &str) -> Option<&DesignTemplate> {
        self.library.get_template(id)
    }

    /// List all templates
    pub fn list_all(&self) -> Vec<&DesignTemplate> {
        self.library.list_all()
    }

    /// List by category
    pub fn list_by_category(&self, category: TemplateCategory) -> Vec<&DesignTemplate> {
        self.library.list_by_category(category)
    }

    /// List favorites
    pub fn list_favorites(&self) -> Vec<&DesignTemplate> {
        self.library.list_favorites()
    }

    /// Search templates
    pub fn search(&self, query: &str) -> Vec<&DesignTemplate> {
        self.library.search(query)
    }

    /// Advanced search
    pub fn search_advanced(
        &self,
        query: Option<&str>,
        category: Option<TemplateCategory>,
        tags: Option<&[String]>,
        favorites_only: bool,
    ) -> Vec<&DesignTemplate> {
        self.library.search_advanced(query, category, tags, favorites_only)
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self, id: &str) -> Result<()> {
        if let Some(template) = self.library.get_template_mut(id) {
            template.toggle_favorite();
            self.save()?;
            Ok(())
        } else {
            Err(anyhow!("Template not found"))
        }
    }

    /// Get template count
    pub fn count(&self) -> usize {
        self.library.count()
    }

    /// Get categories
    pub fn get_categories(&self) -> Vec<TemplateCategory> {
        self.library.get_categories()
    }

    /// Get all tags
    pub fn get_all_tags(&self) -> Vec<String> {
        self.library.get_all_tags()
    }

    /// Save library to disk
    pub fn save(&mut self) -> Result<()> {
        self.library.set_sync_time();
        TemplatePersistence::save(&self.library, &self.storage_path)?;
        Ok(())
    }

    /// Reload library from disk
    pub fn reload(&mut self) -> Result<()> {
        self.library = TemplatePersistence::load(&self.storage_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_category_conversion() {
        assert_eq!(TemplateCategory::Mechanical.as_str(), "mechanical");
        assert_eq!(TemplateCategory::from_str("mechanical"), Some(TemplateCategory::Mechanical));
        assert_eq!(TemplateCategory::from_str("invalid"), None);
    }

    #[test]
    fn test_create_design_template() {
        let template = DesignTemplate::new(
            "test-1".to_string(),
            "Test Template".to_string(),
            "A test template".to_string(),
            TemplateCategory::Mechanical,
            "Test Author".to_string(),
            "{}".to_string(),
        );

        assert_eq!(template.id, "test-1");
        assert_eq!(template.name, "Test Template");
        assert_eq!(template.version, "1.0.0");
        assert!(!template.is_favorite);
    }

    #[test]
    fn test_template_tags() {
        let mut template = DesignTemplate::new(
            "test-1".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            TemplateCategory::Mechanical,
            "Author".to_string(),
            "{}".to_string(),
        );

        template.add_tag("gear".to_string());
        template.add_tag("metal".to_string());
        assert_eq!(template.tags.len(), 2);

        let removed = template.remove_tag("gear");
        assert!(removed);
        assert_eq!(template.tags.len(), 1);
    }

    #[test]
    fn test_template_favorite() {
        let mut template = DesignTemplate::new(
            "test-1".to_string(),
            "Test".to_string(),
            "Test".to_string(),
            TemplateCategory::Mechanical,
            "Author".to_string(),
            "{}".to_string(),
        );

        assert!(!template.is_favorite);
        template.toggle_favorite();
        assert!(template.is_favorite);
        template.set_favorite(false);
        assert!(!template.is_favorite);
    }

    #[test]
    fn test_template_search() {
        let template = DesignTemplate::new(
            "test-1".to_string(),
            "Gear Box".to_string(),
            "A precision gear assembly".to_string(),
            TemplateCategory::Mechanical,
            "John Doe".to_string(),
            "{}".to_string(),
        );

        assert!(template.matches_search("gear"));
        assert!(template.matches_search("assembly"));
        assert!(template.matches_search("john"));
        assert!(!template.matches_search("invalid"));
    }

    #[test]
    fn test_template_library_add_remove() {
        let mut library = DesignTemplateLibrary::new();

        let template1 = DesignTemplate::new(
            "tmpl-1".to_string(),
            "Template 1".to_string(),
            "First template".to_string(),
            TemplateCategory::Mechanical,
            "Author".to_string(),
            "{}".to_string(),
        );

        assert!(library.add_template(template1).is_ok());
        assert_eq!(library.count(), 1);

        let template2 = DesignTemplate::new(
            "tmpl-2".to_string(),
            "Template 2".to_string(),
            "Second template".to_string(),
            TemplateCategory::Decorative,
            "Author".to_string(),
            "{}".to_string(),
        );

        assert!(library.add_template(template2).is_ok());
        assert_eq!(library.count(), 2);

        let removed = library.remove_template("tmpl-1");
        assert!(removed.is_some());
        assert_eq!(library.count(), 1);
    }

    #[test]
    fn test_template_library_duplicate_id() {
        let mut library = DesignTemplateLibrary::new();

        let template1 = DesignTemplate::new(
            "tmpl-1".to_string(),
            "Template 1".to_string(),
            "First".to_string(),
            TemplateCategory::Mechanical,
            "Author".to_string(),
            "{}".to_string(),
        );

        let template2 = DesignTemplate::new(
            "tmpl-1".to_string(),
            "Template 2".to_string(),
            "Second".to_string(),
            TemplateCategory::Decorative,
            "Author".to_string(),
            "{}".to_string(),
        );

        assert!(library.add_template(template1).is_ok());
        assert!(library.add_template(template2).is_err());
    }

    #[test]
    fn test_template_library_search() {
        let mut library = DesignTemplateLibrary::new();

        let mut t1 = DesignTemplate::new(
            "tmpl-1".to_string(),
            "Gear Box".to_string(),
            "Mechanical".to_string(),
            TemplateCategory::Mechanical,
            "Author".to_string(),
            "{}".to_string(),
        );
        t1.add_tag("metal".to_string());

        let t2 = DesignTemplate::new(
            "tmpl-2".to_string(),
            "Decoration".to_string(),
            "Artistic".to_string(),
            TemplateCategory::Decorative,
            "Author".to_string(),
            "{}".to_string(),
        );

        library.add_template(t1).ok();
        library.add_template(t2).ok();

        let results = library.search("gear");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "tmpl-1");
    }

    #[test]
    fn test_template_library_category_filter() {
        let mut library = DesignTemplateLibrary::new();

        let t1 = DesignTemplate::new(
            "tmpl-1".to_string(),
            "T1".to_string(),
            "D".to_string(),
            TemplateCategory::Mechanical,
            "Author".to_string(),
            "{}".to_string(),
        );

        let t2 = DesignTemplate::new(
            "tmpl-2".to_string(),
            "T2".to_string(),
            "D".to_string(),
            TemplateCategory::Decorative,
            "Author".to_string(),
            "{}".to_string(),
        );

        library.add_template(t1).ok();
        library.add_template(t2).ok();

        let mechanical = library.list_by_category(TemplateCategory::Mechanical);
        assert_eq!(mechanical.len(), 1);
        assert_eq!(mechanical[0].id, "tmpl-1");

        let decorative = library.list_by_category(TemplateCategory::Decorative);
        assert_eq!(decorative.len(), 1);
        assert_eq!(decorative[0].id, "tmpl-2");
    }

    #[test]
    fn test_template_library_favorites() {
        let mut library = DesignTemplateLibrary::new();

        let mut t1 = DesignTemplate::new(
            "tmpl-1".to_string(),
            "T1".to_string(),
            "D".to_string(),
            TemplateCategory::Mechanical,
            "Author".to_string(),
            "{}".to_string(),
        );

        let t2 = DesignTemplate::new(
            "tmpl-2".to_string(),
            "T2".to_string(),
            "D".to_string(),
            TemplateCategory::Decorative,
            "Author".to_string(),
            "{}".to_string(),
        );

        t1.set_favorite(true);

        library.add_template(t1).ok();
        library.add_template(t2).ok();

        let favorites = library.list_favorites();
        assert_eq!(favorites.len(), 1);
        assert_eq!(favorites[0].id, "tmpl-1");
    }

    #[test]
    fn test_template_library_advanced_search() {
        let mut library = DesignTemplateLibrary::new();

        let mut t1 = DesignTemplate::new(
            "tmpl-1".to_string(),
            "Gear".to_string(),
            "Metal part".to_string(),
            TemplateCategory::Mechanical,
            "Author".to_string(),
            "{}".to_string(),
        );
        t1.add_tag("metal".to_string());

        let mut t2 = DesignTemplate::new(
            "tmpl-2".to_string(),
            "Flower".to_string(),
            "Decorative".to_string(),
            TemplateCategory::Decorative,
            "Author".to_string(),
            "{}".to_string(),
        );
        t2.set_favorite(true);

        library.add_template(t1).ok();
        library.add_template(t2).ok();

        let results = library.search_advanced(
            None,
            Some(TemplateCategory::Mechanical),
            None,
            false,
        );
        assert_eq!(results.len(), 1);

        let results = library.search_advanced(
            None,
            None,
            None,
            true,
        );
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "tmpl-2");
    }
}
