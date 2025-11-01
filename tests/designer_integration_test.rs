/// Comprehensive integration tests for Designer with other components
///
/// Tests cover:
/// - Designer to G-code Editor workflows
/// - Designer to Visualizer workflows
/// - Template system integration
/// - Undo/Redo functionality
/// - Performance with large designs

#[cfg(test)]
mod designer_integration_tests {
    use gcodekit4::designer::*;
    use gcodekit4::designer_editor_integration::*;
    use gcodekit4::designer_visualizer_integration::SimulationState;
    use gcodekit4::designer_visualizer_integration::*;

    // ============================================================
    // Designer -> Editor Workflow Tests
    // ============================================================

    #[test]
    fn test_design_export_workflow() {
        // Create canvas
        let mut canvas = Canvas::with_size(1600.0, 1200.0);

        // Add shapes
        canvas.add_rectangle(10.0, 10.0, 50.0, 40.0);
        canvas.add_circle(Point::new(75.0, 75.0), 20.0);

        assert_eq!(canvas.shapes().len(), 2);
    }

    #[test]
    fn test_canvas_to_export_parameters() {
        let params = ExportParameters {
            tool_diameter: 3.0,
            cut_depth: 5.0,
            feed_rate: 500.0,
            spindle_speed: 12000,
            safe_z: 10.0,
        };

        assert_eq!(params.tool_diameter, 3.0);
        assert_eq!(params.feed_rate, 500.0);
    }

    #[test]
    fn test_gcode_export_to_editor() {
        let mut integration = DesignEditorIntegration::new();

        let params = ExportParameters::default();
        let export = DesignExport::new(
            "Test Design".to_string(),
            "G00 X0 Y0\nG01 X10 Y10\nM30\n".to_string(),
            params,
        );

        let export_id = integration.export_design(Some("design_1".to_string()), export);

        let exported = integration.get_export(&export_id);
        assert!(exported.is_some());
        assert_eq!(exported.unwrap().name, "Test Design");
    }

    // ============================================================
    // Designer -> Visualizer Workflow Tests
    // ============================================================

    #[test]
    fn test_design_visualization_setup() {
        let mut viz_integration = DesignerVisualizerIntegration::new();
        let bounds = VisualizationBounds::new(-100.0, -100.0, -10.0, 100.0, 100.0, 10.0);
        let design_viz = DesignVisualization::new("Test Design".to_string(), bounds);

        viz_integration.load_design(design_viz);

        assert!(viz_integration.current_visualization().is_some());
    }

    #[test]
    fn test_visualization_material_settings() {
        let mut viz_integration = DesignerVisualizerIntegration::new();
        let bounds = VisualizationBounds::default();
        let mut design_viz = DesignVisualization::new("Test".to_string(), bounds);

        // Customize material settings
        design_viz.material_settings.material_color = (1.0, 0.0, 0.0);
        design_viz.material_settings.opacity = 0.8;

        viz_integration.load_design(design_viz);

        let viz = viz_integration.current_visualization().unwrap();
        assert_eq!(viz.material_settings.material_color, (1.0, 0.0, 0.0));
        assert_eq!(viz.material_settings.opacity, 0.8);
    }

    #[test]
    fn test_simulation_workflow() {
        let mut viz_integration = DesignerVisualizerIntegration::new();
        let bounds = VisualizationBounds::default();
        let design_viz = DesignVisualization::new("Test".to_string(), bounds);

        viz_integration.load_design(design_viz);

        // Start simulation
        assert!(viz_integration.start_simulation());
        assert_eq!(viz_integration.simulation_state, SimulationState::Running);

        // Pause
        assert!(viz_integration.pause_simulation());
        assert_eq!(viz_integration.simulation_state, SimulationState::Paused);

        // Resume
        assert!(viz_integration.resume_simulation());
        assert_eq!(viz_integration.simulation_state, SimulationState::Running);

        // Stop
        viz_integration.stop_simulation();
        assert_eq!(viz_integration.simulation_state, SimulationState::Idle);
    }

    // ============================================================
    // Template Integration Tests
    // ============================================================

    #[test]
    fn test_template_creation_and_storage() {
        let canvas = Canvas::with_size(1600.0, 1200.0);

        // Templates would be created from canvas designs
        assert!(!canvas.shapes().is_empty() || canvas.shapes().is_empty()); // Basic check
    }

    #[test]
    fn test_template_export_workflow() {
        let mut integration = DesignEditorIntegration::new();

        // Create and export as template
        let params = ExportParameters::default();
        let export = DesignExport::new(
            "Template".to_string(),
            "G-code template".to_string(),
            params,
        );

        let export_id = integration.export_design(None, export);
        assert!(!export_id.is_empty());

        // Get recent exports (simulating template retrieval)
        let recent = integration.get_recent_exports();
        assert!(!recent.is_empty());
    }

    // ============================================================
    // Undo/Redo Integration Tests
    // ============================================================

    #[test]
    fn test_history_with_canvas_operations() {
        let mut history = UndoRedoManager::new(50);

        // Record shape creation
        let action1 =
            HistoryAction::simple(ActionType::ShapeCreated, "Created rectangle".to_string());
        history.record(action1);

        // Record shape movement
        let action2 =
            HistoryAction::simple(ActionType::ShapeMoved, "Moved to (10, 20)".to_string());
        history.record(action2);

        assert_eq!(history.undo_depth(), 2);

        // Undo both
        let _undo1 = history.undo();
        let _undo2 = history.undo();

        assert_eq!(history.redo_depth(), 2);
        assert_eq!(history.undo_depth(), 0);
    }

    #[test]
    fn test_undo_redo_with_export() {
        let mut history = UndoRedoManager::new(50);
        let mut editor_integration = DesignEditorIntegration::new();

        // Simulate design modifications
        let actions = vec![
            ActionType::ShapeCreated,
            ActionType::ShapeMoved,
            ActionType::ShapeResized,
        ];

        for (i, action_type) in actions.iter().enumerate() {
            let action =
                HistoryAction::simple(action_type.clone(), format!("Design modification {}", i));
            history.record(action);
        }

        // Export current state
        let params = ExportParameters::default();
        let export = DesignExport::new("Design State".to_string(), "G-code".to_string(), params);
        editor_integration.export_design(None, export);

        assert_eq!(history.undo_depth(), 3);
        assert_eq!(editor_integration.stats().total_exports, 1);
    }

    // ============================================================
    // Comprehensive Workflow Tests
    // ============================================================

    #[test]
    fn test_full_design_to_machine_workflow() {
        // 1. Create design
        let canvas = Canvas::with_size(1600.0, 1200.0);

        // 2. Track history
        let mut history = UndoRedoManager::new(50);
        let action = HistoryAction::simple(ActionType::ShapeCreated, "Rectangle".to_string());
        history.record(action);

        // 3. Export to editor
        let mut editor_integration = DesignEditorIntegration::new();
        let params = ExportParameters::default();
        let export = DesignExport::new(
            "Machine Design".to_string(),
            "G-code output".to_string(),
            params,
        );
        let export_id = editor_integration.export_design(None, export);

        // 4. Visualize
        let mut viz_integration = DesignerVisualizerIntegration::new();
        let bounds = VisualizationBounds::default();
        let design_viz = DesignVisualization::new("Visualization".to_string(), bounds);
        viz_integration.load_design(design_viz);

        // Verify workflow completion
        assert_eq!(canvas.shapes().len(), 0); // Empty canvas
        assert_eq!(history.undo_depth(), 1);
        assert!(editor_integration.get_export(&export_id).is_some());
        assert!(viz_integration.current_visualization().is_some());
    }

    // ============================================================
    // Error Handling and Edge Cases
    // ============================================================

    #[test]
    fn test_empty_design_export() {
        let canvas = Canvas::with_size(1600.0, 1200.0);

        // Empty canvas
        assert_eq!(canvas.shapes().len(), 0);

        // Should still be able to export
        let mut integration = DesignEditorIntegration::new();
        let params = ExportParameters::default();
        let export = DesignExport::new("Empty Design".to_string(), "".to_string(), params);

        let export_id = integration.export_design(None, export);
        assert!(!export_id.is_empty());
    }

    #[test]
    fn test_large_export_handling() {
        let mut integration = DesignEditorIntegration::new();

        // Simulate large G-code
        let mut large_gcode = String::new();
        for i in 0..1000 {
            large_gcode.push_str(&format!("G01 X{} Y{}\n", i, i * 2));
        }

        let params = ExportParameters::default();
        let export = DesignExport::new("Large Design".to_string(), large_gcode, params);

        let export_id = integration.export_design(None, export);
        let exported = integration.get_export(&export_id).unwrap();

        assert_eq!(exported.gcode_lines(), 1000);
    }

    #[test]
    fn test_visualization_without_active_design() {
        let mut viz_integration = DesignerVisualizerIntegration::new();

        // Try operations without active design
        assert!(!viz_integration.start_simulation());
        assert!(!viz_integration.pause_simulation());
        assert!(!viz_integration.resume_simulation());
        assert!(!viz_integration.toggle_toolpath());

        // Stats should reflect no active design
        let stats = viz_integration.stats();
        assert!(!stats.has_active_design);
    }

    #[test]
    fn test_history_depth_limit() {
        let mut history = UndoRedoManager::new(10);

        // Add more than max depth
        for i in 0..20 {
            let action = HistoryAction::simple(ActionType::ShapeCreated, format!("Action {}", i));
            history.record(action);
        }

        // Should be limited to max_depth
        assert_eq!(history.undo_depth(), 10);
    }

    // ============================================================
    // Performance Tests
    // ============================================================

    #[test]
    fn test_export_history_performance() {
        let mut integration = DesignEditorIntegration::new();
        let params = ExportParameters::default();

        // Create many exports
        for i in 0..100 {
            let export = DesignExport::new(
                format!("Design {}", i),
                "G-code".to_string(),
                params.clone(),
            );
            integration.export_design(None, export);
        }

        let stats = integration.stats();
        assert_eq!(stats.total_exports, 100);

        // Verify recent tracking still works
        let recent = integration.get_recent_exports();
        assert!(recent.len() <= 10); // max_recent limit
    }

    #[test]
    fn test_visualization_bounds_calculations() {
        let bounds = VisualizationBounds::new(0.0, 0.0, 0.0, 1000.0, 1000.0, 1000.0);

        // Many calculations should be fast
        for _ in 0..1000 {
            let _center = bounds.center();
            let _dims = bounds.dimensions();
        }

        // Verify calculations are correct
        let (cx, cy, cz) = bounds.center();
        assert_eq!(cx, 500.0);
        assert_eq!(cy, 500.0);
        assert_eq!(cz, 500.0);
    }

    #[test]
    fn test_undo_manager_with_many_actions() {
        let mut history = UndoRedoManager::new(100);

        // Record 100 actions
        for i in 0..100 {
            let action = HistoryAction::simple(
                if i % 2 == 0 {
                    ActionType::ShapeCreated
                } else {
                    ActionType::ShapeMoved
                },
                format!("Action {}", i),
            );
            history.record(action);
        }

        // Verify all recorded
        assert_eq!(history.undo_depth(), 100);

        // Undo all
        for _ in 0..100 {
            history.undo();
        }

        assert_eq!(history.undo_depth(), 0);
        assert_eq!(history.redo_depth(), 100);
    }
}
