//! Demo of the custom text editor with undo/redo functionality
//!
//! Run with: cargo run --example custom_editor_demo

use gcodekit4_ui::{EditorBridge, SlintTextLine};
use std::rc::Rc;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    // Create the editor bridge
    let bridge = Rc::new(EditorBridge::new(600.0, 20.0));
    
    // Load some initial text
    bridge.load_text("G00 X0 Y0 Z0\nG01 X10 Y10 F500\nG01 Z-1\nG01 X20 Y20\nM5\n");
    
    // Create the UI
    let ui = AppWindow::new()?;
    
    // Set initial state
    ui.set_can_undo(bridge.can_undo());
    ui.set_can_redo(bridge.can_redo());
    ui.set_total_lines(bridge.line_count() as i32);
    ui.set_cursor_line(0);
    ui.set_cursor_column(0);
    
    // Update visible lines
    let visible_lines_model = bridge.get_visible_lines_model();
    ui.set_visible_lines(visible_lines_model);
    
    // Handle undo
    {
        let bridge = bridge.clone();
        let ui_weak = ui.as_weak();
        ui.on_undo_requested(move || {
            if bridge.undo() {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_can_undo(bridge.can_undo());
                    ui.set_can_redo(bridge.can_redo());
                    ui.set_visible_lines(bridge.get_visible_lines_model());
                }
            }
        });
    }
    
    // Handle redo
    {
        let bridge = bridge.clone();
        let ui_weak = ui.as_weak();
        ui.on_redo_requested(move || {
            if bridge.redo() {
                if let Some(ui) = ui_weak.upgrade() {
                    ui.set_can_undo(bridge.can_undo());
                    ui.set_can_redo(bridge.can_redo());
                    ui.set_visible_lines(bridge.get_visible_lines_model());
                }
            }
        });
    }
    
    // Handle scroll
    {
        let bridge = bridge.clone();
        let ui_weak = ui.as_weak();
        ui.on_scroll_changed(move |line| {
            bridge.scroll_to_line(line as usize);
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_visible_lines(bridge.get_visible_lines_model());
            }
        });
    }
    
    // Handle text changes (simplified - in real app would track actual edits)
    {
        let bridge = bridge.clone();
        let ui_weak = ui.as_weak();
        ui.on_text_changed(move |_text| {
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_can_undo(bridge.can_undo());
                ui.set_can_redo(bridge.can_redo());
            }
        });
    }
    
    // Handle save
    {
        let bridge = bridge.clone();
        ui.on_save_requested(move || {
            let text = bridge.get_text();
            println!("Saving text ({} bytes):\n{}", text.len(), text);
            bridge.mark_unmodified();
        });
    }
    
    // Handle open
    {
        let bridge = bridge.clone();
        let ui_weak = ui.as_weak();
        ui.on_open_requested(move || {
            // In real app, would open file dialog
            let sample_text = "G00 X0 Y0 Z5\nG01 Z-2 F200\nG01 X50 Y50\nG00 Z5\nM5\n";
            bridge.load_text(sample_text);
            
            if let Some(ui) = ui_weak.upgrade() {
                ui.set_can_undo(bridge.can_undo());
                ui.set_can_redo(bridge.can_redo());
                ui.set_total_lines(bridge.line_count() as i32);
                ui.set_visible_lines(bridge.get_visible_lines_model());
            }
        });
    }
    
    ui.run()
}
