//! Coordinate System Panel - Task 76
//!
//! Work coordinate system selection and offsets display

use std::collections::HashMap;

/// Coordinate system identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CoordinateSystemId {
    /// G54 - Work coordinate system 1
    G54,
    /// G55 - Work coordinate system 2
    G55,
    /// G56 - Work coordinate system 3
    G56,
    /// G57 - Work coordinate system 4
    G57,
    /// G58 - Work coordinate system 5
    G58,
    /// G59 - Work coordinate system 6
    G59,
}

impl CoordinateSystemId {
    /// Get all coordinate systems
    pub fn all() -> Vec<Self> {
        vec![
            Self::G54,
            Self::G55,
            Self::G56,
            Self::G57,
            Self::G58,
            Self::G59,
        ]
    }

    /// Get G-code command
    pub fn gcode(&self) -> &str {
        match self {
            Self::G54 => "G54",
            Self::G55 => "G55",
            Self::G56 => "G56",
            Self::G57 => "G57",
            Self::G58 => "G58",
            Self::G59 => "G59",
        }
    }

    /// Get system number (1-6)
    pub fn number(&self) -> u8 {
        match self {
            Self::G54 => 1,
            Self::G55 => 2,
            Self::G56 => 3,
            Self::G57 => 4,
            Self::G58 => 5,
            Self::G59 => 6,
        }
    }
}

impl std::fmt::Display for CoordinateSystemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.gcode())
    }
}

/// Coordinate offset
#[derive(Debug, Clone, Copy)]
pub struct CoordinateOffset {
    /// X offset
    pub x: f32,
    /// Y offset
    pub y: f32,
    /// Z offset
    pub z: f32,
}

impl CoordinateOffset {
    /// Create new coordinate offset
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Get offset as tuple
    pub fn as_tuple(&self) -> (f32, f32, f32) {
        (self.x, self.y, self.z)
    }

    /// Get formatted string
    pub fn formatted(&self, units: &str) -> String {
        format!("X:{:.2} Y:{:.2} Z:{:.2} {}", self.x, self.y, self.z, units)
    }

    /// Set offset value
    pub fn set(&mut self, axis: char, value: f32) {
        match axis {
            'X' => self.x = value,
            'Y' => self.y = value,
            'Z' => self.z = value,
            _ => {}
        }
    }

    /// Get offset value
    pub fn get(&self, axis: char) -> Option<f32> {
        match axis {
            'X' => Some(self.x),
            'Y' => Some(self.y),
            'Z' => Some(self.z),
            _ => None,
        }
    }
}

impl Default for CoordinateOffset {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// Work coordinate system definition
#[derive(Debug, Clone)]
pub struct WorkCoordinateSystem {
    /// System identifier
    pub id: CoordinateSystemId,
    /// Coordinate offsets
    pub offset: CoordinateOffset,
    /// Description
    pub description: String,
}

impl WorkCoordinateSystem {
    /// Create new WCS
    pub fn new(id: CoordinateSystemId) -> Self {
        Self {
            id,
            offset: CoordinateOffset::default(),
            description: format!("Work Coordinate System {}", id.number()),
        }
    }

    /// Set offset
    pub fn set_offset(&mut self, x: f32, y: f32, z: f32) {
        self.offset = CoordinateOffset::new(x, y, z);
    }

    /// Get current position with offset
    pub fn apply_offset(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        (x + self.offset.x, y + self.offset.y, z + self.offset.z)
    }

    /// Remove offset from position
    pub fn remove_offset(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        (x - self.offset.x, y - self.offset.y, z - self.offset.z)
    }
}

/// Zero operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZeroType {
    /// Zero X axis only
    X,
    /// Zero Y axis only
    Y,
    /// Zero Z axis only
    Z,
    /// Zero all axes
    All,
}

impl std::fmt::Display for ZeroType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X => write!(f, "Zero X"),
            Self::Y => write!(f, "Zero Y"),
            Self::Z => write!(f, "Zero Z"),
            Self::All => write!(f, "Zero All"),
        }
    }
}

/// Coordinate system panel
#[derive(Debug)]
pub struct CoordinateSystemPanel {
    /// All work coordinate systems
    pub systems: HashMap<CoordinateSystemId, WorkCoordinateSystem>,
    /// Active/current WCS
    pub active_system: CoordinateSystemId,
    /// Current machine position
    pub current_position: (f32, f32, f32),
    /// Unit system (mm or in)
    pub units: String,
}

impl CoordinateSystemPanel {
    /// Create new coordinate system panel
    pub fn new() -> Self {
        let mut systems = HashMap::new();

        for id in CoordinateSystemId::all() {
            systems.insert(id, WorkCoordinateSystem::new(id));
        }

        Self {
            systems,
            active_system: CoordinateSystemId::G54,
            current_position: (0.0, 0.0, 0.0),
            units: "mm".to_string(),
        }
    }

    /// Select coordinate system
    pub fn select_system(&mut self, id: CoordinateSystemId) -> Option<String> {
        if self.systems.contains_key(&id) {
            self.active_system = id;
            Some(id.gcode().to_string())
        } else {
            None
        }
    }

    /// Get active system
    pub fn get_active_system(&self) -> Option<&WorkCoordinateSystem> {
        self.systems.get(&self.active_system)
    }

    /// Get mutable active system
    pub fn get_active_system_mut(&mut self) -> Option<&mut WorkCoordinateSystem> {
        self.systems.get_mut(&self.active_system)
    }

    /// Set offset in active system
    pub fn set_active_offset(&mut self, x: f32, y: f32, z: f32) {
        if let Some(system) = self.get_active_system_mut() {
            system.set_offset(x, y, z);
        }
    }

    /// Zero axis in active system
    pub fn zero_axis(&mut self, axis: char) -> bool {
        if let Some(system) = self.get_active_system_mut() {
            system.offset.set(axis, 0.0);
            true
        } else {
            false
        }
    }

    /// Zero all axes in active system
    pub fn zero_all_axes(&mut self) -> bool {
        if let Some(system) = self.get_active_system_mut() {
            system.set_offset(0.0, 0.0, 0.0);
            true
        } else {
            false
        }
    }

    /// Set work position (sets offset based on current position)
    pub fn set_work_position(&mut self, x: f32, y: f32, z: f32) -> bool {
        let (cx, cy, cz) = self.current_position;
        if let Some(system) = self.get_active_system_mut() {
            system.set_offset(x - cx, y - cy, z - cz);
            true
        } else {
            false
        }
    }

    /// Go to zero (return to work coordinate zero)
    pub fn go_to_zero(&self) -> (f32, f32, f32) {
        if let Some(system) = self.get_active_system() {
            let (x, y, z) = system.offset.as_tuple();
            (-x, -y, -z)
        } else {
            (0.0, 0.0, 0.0)
        }
    }

    /// Update current machine position
    pub fn update_position(&mut self, x: f32, y: f32, z: f32) {
        self.current_position = (x, y, z);
    }

    /// Get work position from machine position
    pub fn get_work_position(&self) -> (f32, f32, f32) {
        if let Some(system) = self.get_active_system() {
            system.remove_offset(
                self.current_position.0,
                self.current_position.1,
                self.current_position.2,
            )
        } else {
            self.current_position
        }
    }

    /// Get all offsets
    pub fn get_all_offsets(&self) -> HashMap<CoordinateSystemId, (f32, f32, f32)> {
        self.systems
            .iter()
            .map(|(id, sys)| (*id, sys.offset.as_tuple()))
            .collect()
    }

    /// Get offset for specific system
    pub fn get_offset(&self, id: CoordinateSystemId) -> Option<(f32, f32, f32)> {
        self.systems.get(&id).map(|s| s.offset.as_tuple())
    }

    /// Get system description
    pub fn get_description(&self, id: CoordinateSystemId) -> Option<String> {
        self.systems.get(&id).map(|s| s.description.clone())
    }

    /// Set system description
    pub fn set_description(
        &mut self,
        id: CoordinateSystemId,
        description: impl Into<String>,
    ) -> bool {
        if let Some(system) = self.systems.get_mut(&id) {
            system.description = description.into();
            true
        } else {
            false
        }
    }

    /// Get offset summary for active system
    pub fn active_offset_summary(&self) -> String {
        if let Some(system) = self.get_active_system() {
            format!("{}: {}", system.id, system.offset.formatted(&self.units))
        } else {
            "No active system".to_string()
        }
    }

    /// Get all systems list
    pub fn get_systems_list(&self) -> Vec<(CoordinateSystemId, String)> {
        CoordinateSystemId::all()
            .iter()
            .map(|id| {
                (
                    *id,
                    self.systems
                        .get(id)
                        .map(|s| s.description.clone())
                        .unwrap_or_else(|| format!("WCS {}", id.number())),
                )
            })
            .collect()
    }
}

impl Default for CoordinateSystemPanel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_system_id_all() {
        let all = CoordinateSystemId::all();
        assert_eq!(all.len(), 6);
    }

    #[test]
    fn test_coordinate_system_id_gcode() {
        assert_eq!(CoordinateSystemId::G54.gcode(), "G54");
        assert_eq!(CoordinateSystemId::G59.gcode(), "G59");
    }

    #[test]
    fn test_coordinate_system_id_number() {
        assert_eq!(CoordinateSystemId::G54.number(), 1);
        assert_eq!(CoordinateSystemId::G59.number(), 6);
    }

    #[test]
    fn test_coordinate_offset() {
        let offset = CoordinateOffset::new(10.5, 20.0, -5.5);
        assert_eq!(offset.x, 10.5);
        let (x, y, z) = offset.as_tuple();
        assert_eq!(x, 10.5);
    }

    #[test]
    fn test_coordinate_offset_set() {
        let mut offset = CoordinateOffset::default();
        offset.set('X', 5.0);
        assert_eq!(offset.x, 5.0);
    }

    #[test]
    fn test_coordinate_offset_get() {
        let offset = CoordinateOffset::new(5.0, 10.0, 15.0);
        assert_eq!(offset.get('X'), Some(5.0));
        assert_eq!(offset.get('Y'), Some(10.0));
    }

    #[test]
    fn test_wcs_creation() {
        let wcs = WorkCoordinateSystem::new(CoordinateSystemId::G54);
        assert_eq!(wcs.id, CoordinateSystemId::G54);
        assert_eq!(wcs.offset.x, 0.0);
    }

    #[test]
    fn test_wcs_apply_offset() {
        let mut wcs = WorkCoordinateSystem::new(CoordinateSystemId::G54);
        wcs.set_offset(10.0, 20.0, 5.0);
        let (x, y, z) = wcs.apply_offset(0.0, 0.0, 0.0);
        assert_eq!(x, 10.0);
        assert_eq!(y, 20.0);
        assert_eq!(z, 5.0);
    }

    #[test]
    fn test_wcs_remove_offset() {
        let mut wcs = WorkCoordinateSystem::new(CoordinateSystemId::G54);
        wcs.set_offset(10.0, 20.0, 5.0);
        let (x, y, z) = wcs.remove_offset(15.0, 25.0, 10.0);
        assert_eq!(x, 5.0);
        assert_eq!(y, 5.0);
        assert_eq!(z, 5.0);
    }

    #[test]
    fn test_panel_creation() {
        let panel = CoordinateSystemPanel::new();
        assert_eq!(panel.systems.len(), 6);
        assert_eq!(panel.active_system, CoordinateSystemId::G54);
    }

    #[test]
    fn test_panel_select_system() {
        let mut panel = CoordinateSystemPanel::new();
        let result = panel.select_system(CoordinateSystemId::G55);
        assert!(result.is_some());
        assert_eq!(panel.active_system, CoordinateSystemId::G55);
    }

    #[test]
    fn test_panel_zero_axis() {
        let mut panel = CoordinateSystemPanel::new();
        panel.set_active_offset(10.0, 20.0, 5.0);
        panel.zero_axis('X');
        let offset = panel.get_active_system().unwrap().offset;
        assert_eq!(offset.x, 0.0);
    }

    #[test]
    fn test_panel_zero_all() {
        let mut panel = CoordinateSystemPanel::new();
        panel.set_active_offset(10.0, 20.0, 5.0);
        panel.zero_all_axes();
        let offset = panel.get_active_system().unwrap().offset;
        assert_eq!(offset.x, 0.0);
        assert_eq!(offset.y, 0.0);
        assert_eq!(offset.z, 0.0);
    }

    #[test]
    fn test_panel_set_work_position() {
        let mut panel = CoordinateSystemPanel::new();
        panel.update_position(100.0, 200.0, 50.0);
        panel.set_work_position(10.0, 20.0, 5.0);
        let offset = panel.get_active_system().unwrap().offset;
        assert_eq!(offset.x, -90.0);
        assert_eq!(offset.y, -180.0);
        assert_eq!(offset.z, -45.0);
    }

    #[test]
    fn test_panel_go_to_zero() {
        let mut panel = CoordinateSystemPanel::new();
        panel.set_active_offset(10.0, 20.0, 5.0);
        let (x, y, z) = panel.go_to_zero();
        assert_eq!(x, -10.0);
        assert_eq!(y, -20.0);
        assert_eq!(z, -5.0);
    }

    #[test]
    fn test_panel_work_position() {
        let mut panel = CoordinateSystemPanel::new();
        panel.set_active_offset(10.0, 20.0, 5.0);
        panel.update_position(50.0, 60.0, 30.0);
        let (x, y, z) = panel.get_work_position();
        assert_eq!(x, 40.0);
        assert_eq!(y, 40.0);
        assert_eq!(z, 25.0);
    }

    #[test]
    fn test_panel_get_all_offsets() {
        let panel = CoordinateSystemPanel::new();
        let offsets = panel.get_all_offsets();
        assert_eq!(offsets.len(), 6);
    }

    #[test]
    fn test_panel_offset_summary() {
        let mut panel = CoordinateSystemPanel::new();
        panel.set_active_offset(5.0, 10.0, 15.0);
        let summary = panel.active_offset_summary();
        assert!(summary.contains("G54"));
    }

    #[test]
    fn test_panel_systems_list() {
        let panel = CoordinateSystemPanel::new();
        let list = panel.get_systems_list();
        assert_eq!(list.len(), 6);
    }

    #[test]
    fn test_panel_description() {
        let mut panel = CoordinateSystemPanel::new();
        panel.set_description(CoordinateSystemId::G54, "Main Setup");
        let desc = panel.get_description(CoordinateSystemId::G54);
        assert_eq!(desc, Some("Main Setup".to_string()));
    }
}
