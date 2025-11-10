//! Macros Panel - Task 77
//!
//! G-Code macro button grid, editor, execution, variable substitution,
//! and macro import/export functionality

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Macro variable for substitution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroVariable {
    /// Variable name
    pub name: String,
    /// Variable value
    pub value: String,
    /// Variable description
    pub description: Option<String>,
}

impl MacroVariable {
    /// Create new macro variable
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            description: None,
        }
    }

    /// Set variable description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// G-Code macro definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcodeMacro {
    /// Macro ID/name
    pub id: String,
    /// Macro display name
    pub name: String,
    /// G-Code content with optional variables
    pub gcode: String,
    /// Variables used in this macro
    pub variables: Vec<MacroVariable>,
    /// Macro description
    pub description: Option<String>,
    /// Button color/icon hint
    pub button_color: Option<String>,
    /// Whether macro has been modified
    pub modified: bool,
}

impl GcodeMacro {
    /// Create new macro
    pub fn new(id: impl Into<String>, name: impl Into<String>, gcode: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            gcode: gcode.into(),
            variables: Vec::new(),
            description: None,
            button_color: None,
            modified: false,
        }
    }

    /// Add variable to macro
    pub fn add_variable(mut self, var: MacroVariable) -> Self {
        self.variables.push(var);
        self
    }

    /// Set macro description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set button color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.button_color = Some(color.into());
        self
    }

    /// Substitute variables in macro G-code
    pub fn substitute_variables(&self, var_values: &HashMap<String, String>) -> String {
        let mut result = self.gcode.clone();
        for var in &self.variables {
            let search = format!("${{{}}}", var.name);
            if let Some(value) = var_values.get(&var.name) {
                result = result.replace(&search, value);
            } else {
                result = result.replace(&search, &var.value);
            }
        }
        result
    }
}

/// Macros panel manager
#[derive(Debug, Clone)]
pub struct MacrosPanel {
    /// All defined macros
    pub macros: HashMap<String, GcodeMacro>,
    /// Current macro grid layout (columns)
    pub grid_columns: usize,
    /// Selected macro ID
    pub selected_macro: Option<String>,
    /// Macro editor content (when editing)
    pub editor_content: Option<String>,
}

impl MacrosPanel {
    /// Create new macros panel
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            grid_columns: 4,
            selected_macro: None,
            editor_content: None,
        }
    }

    /// Add macro to panel
    pub fn add_macro(&mut self, macro_def: GcodeMacro) {
        self.macros.insert(macro_def.id.clone(), macro_def);
    }

    /// Get macro by ID
    pub fn get_macro(&self, id: &str) -> Option<&GcodeMacro> {
        self.macros.get(id)
    }

    /// Get mutable macro by ID
    pub fn get_macro_mut(&mut self, id: &str) -> Option<&mut GcodeMacro> {
        self.macros.get_mut(id)
    }

    /// Remove macro by ID
    pub fn remove_macro(&mut self, id: &str) -> Option<GcodeMacro> {
        self.macros.remove(id)
    }

    /// List all macro IDs
    pub fn list_macros(&self) -> Vec<&str> {
        self.macros.keys().map(|k| k.as_str()).collect()
    }

    /// Start editing macro
    pub fn edit_macro(&mut self, id: &str) {
        if let Some(macro_def) = self.macros.get(id) {
            self.selected_macro = Some(id.to_string());
            self.editor_content = Some(macro_def.gcode.clone());
        }
    }

    /// Save macro edit
    pub fn save_macro_edit(&mut self, content: String) -> bool {
        if let Some(id) = &self.selected_macro.clone() {
            if let Some(macro_def) = self.macros.get_mut(id) {
                macro_def.gcode = content;
                macro_def.modified = true;
                return true;
            }
        }
        false
    }

    /// Cancel macro edit
    pub fn cancel_edit(&mut self) {
        self.editor_content = None;
    }

    /// Set grid layout columns
    pub fn set_grid_columns(&mut self, columns: usize) {
        self.grid_columns = columns.max(1);
    }

    /// Serialize macros to JSON string
    pub fn export_macros(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.macros)
    }

    /// Load macros from JSON string
    pub fn import_macros(&mut self, json: &str) -> Result<(), serde_json::Error> {
        let loaded: HashMap<String, GcodeMacro> = serde_json::from_str(json)?;
        self.macros.extend(loaded);
        Ok(())
    }

    /// Clear all macros
    pub fn clear_all(&mut self) {
        self.macros.clear();
        self.selected_macro = None;
        self.editor_content = None;
    }
}

impl Default for MacrosPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macro_creation() {
        let macro_def = GcodeMacro::new("home", "Home All", "G28");
        assert_eq!(macro_def.id, "home");
        assert_eq!(macro_def.name, "Home All");
        assert_eq!(macro_def.gcode, "G28");
    }

    #[test]
    fn test_macro_with_variables() {
        let var = MacroVariable::new("speed", "100");
        let macro_def = GcodeMacro::new("move", "Move", "G0 X${x} Y${y} F${speed}")
            .add_variable(MacroVariable::new("x", "10"))
            .add_variable(MacroVariable::new("y", "20"))
            .add_variable(var);

        let mut var_values = HashMap::new();
        var_values.insert("x".to_string(), "50".to_string());
        var_values.insert("y".to_string(), "60".to_string());
        var_values.insert("speed".to_string(), "200".to_string());

        let result = macro_def.substitute_variables(&var_values);
        assert_eq!(result, "G0 X50 Y60 F200");
    }

    #[test]
    fn test_macro_default_variables() {
        let var = MacroVariable::new("speed", "100");
        let macro_def = GcodeMacro::new("move", "Move", "G0 X${x} Y${y} F${speed}")
            .add_variable(MacroVariable::new("x", "10"))
            .add_variable(MacroVariable::new("y", "20"))
            .add_variable(var);

        let var_values = HashMap::new();
        let result = macro_def.substitute_variables(&var_values);
        assert_eq!(result, "G0 X10 Y20 F100");
    }

    #[test]
    fn test_macros_panel() {
        let mut panel = MacrosPanel::new();
        let macro_def = GcodeMacro::new("home", "Home All", "G28");
        panel.add_macro(macro_def);

        assert_eq!(panel.list_macros().len(), 1);
        assert!(panel.get_macro("home").is_some());
    }

    #[test]
    fn test_macros_export_import() {
        let mut panel = MacrosPanel::new();
        let macro_def = GcodeMacro::new("home", "Home All", "G28");
        panel.add_macro(macro_def);

        let json = panel.export_macros().unwrap();
        let mut panel2 = MacrosPanel::new();
        panel2.import_macros(&json).unwrap();

        assert_eq!(panel2.list_macros().len(), 1);
        assert_eq!(panel2.get_macro("home").unwrap().name, "Home All");
    }
}
