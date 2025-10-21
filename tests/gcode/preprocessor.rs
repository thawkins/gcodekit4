/// Tests for G-Code preprocessor framework (Task 13)
///
/// Tests the CommandProcessor trait, ProcessorPipeline, and ProcessorRegistry
use gcodekit4::gcode::{
    CommandProcessor, GcodeCommand, GcodeState, ProcessorConfig, ProcessorPipeline,
    ProcessorRegistry,
};
use std::sync::Arc;

/// Mock processor for testing
struct TestProcessor {
    name: String,
    enabled: bool,
    append_suffix: String,
}

impl TestProcessor {
    fn new(name: &str, append_suffix: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: true,
            append_suffix: append_suffix.to_string(),
        }
    }

    fn disabled(name: &str) -> Self {
        Self {
            name: name.to_string(),
            enabled: false,
            append_suffix: String::new(),
        }
    }
}

impl CommandProcessor for TestProcessor {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Test processor"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let mut processed = command.clone();
        processed.command = format!("{}{}", command.command, self.append_suffix);
        Ok(vec![processed])
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Mock processor that skips commands
struct SkipProcessor;

impl CommandProcessor for SkipProcessor {
    fn name(&self) -> &str {
        "skip_processor"
    }

    fn description(&self) -> &str {
        "Skips all commands"
    }

    fn process(
        &self,
        _command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        Ok(Vec::new())
    }
}

/// Mock processor that expands commands
struct ExpandProcessor;

impl CommandProcessor for ExpandProcessor {
    fn name(&self) -> &str {
        "expand_processor"
    }

    fn description(&self) -> &str {
        "Expands commands into multiple"
    }

    fn process(
        &self,
        command: &GcodeCommand,
        _state: &GcodeState,
    ) -> Result<Vec<GcodeCommand>, String> {
        let mut cmd1 = command.clone();
        cmd1.command = format!("{}_part1", command.command);

        let mut cmd2 = command.clone();
        cmd2.command = format!("{}_part2", command.command);

        Ok(vec![cmd1, cmd2])
    }
}

#[test]
fn test_processor_config_creation() {
    let config = ProcessorConfig::new();
    assert!(config.enabled);
    assert!(config.options.is_empty());
}

#[test]
fn test_processor_config_disabled() {
    let config = ProcessorConfig::disabled();
    assert!(!config.enabled);
}

#[test]
fn test_processor_config_options() {
    let config = ProcessorConfig::new()
        .with_option("key1", "value1")
        .with_option("key2", "value2");

    assert_eq!(config.get_option("key1"), Some("value1"));
    assert_eq!(config.get_option("key2"), Some("value2"));
    assert_eq!(config.get_option("key3"), None);
}

#[test]
fn test_basic_processor_pipeline() {
    let mut pipeline = ProcessorPipeline::new();
    assert_eq!(pipeline.processor_count(), 0);

    let processor = Arc::new(TestProcessor::new("test1", "_processed"));
    pipeline.register(processor);
    assert_eq!(pipeline.processor_count(), 1);
}

#[test]
fn test_processor_pipeline_list() {
    let mut pipeline = ProcessorPipeline::new();

    let proc1 = Arc::new(TestProcessor::new("proc1", ""));
    let proc2 = Arc::new(TestProcessor::new("proc2", ""));
    let proc3 = Arc::new(TestProcessor::disabled("proc3"));

    pipeline.register(proc1);
    pipeline.register(proc2);
    pipeline.register(proc3);

    let list = pipeline.list_processors();
    assert_eq!(list.len(), 3);
    assert_eq!(list[0].0, "proc1");
    assert_eq!(list[1].0, "proc2");
    assert_eq!(list[2].0, "proc3");
    assert!(list[0].2); // proc1 enabled
    assert!(list[1].2); // proc2 enabled
    assert!(!list[2].2); // proc3 disabled
}

#[test]
fn test_processor_pipeline_get_by_name() {
    let mut pipeline = ProcessorPipeline::new();

    let proc1 = Arc::new(TestProcessor::new("proc1", ""));
    let proc2 = Arc::new(TestProcessor::new("proc2", ""));

    pipeline.register(proc1);
    pipeline.register(proc2);

    assert!(pipeline.get_processor_by_name("proc1").is_some());
    assert!(pipeline.get_processor_by_name("proc2").is_some());
    assert!(pipeline.get_processor_by_name("proc3").is_none());
}

#[test]
fn test_process_single_command() {
    let pipeline = ProcessorPipeline::new();
    let state = GcodeState::new();
    let command = GcodeCommand::new("G00 X10 Y20");

    let result = pipeline.process_command(&command, &state);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.len(), 1);
    assert_eq!(processed[0].command, command.command);
}

#[test]
fn test_process_command_with_processor() {
    let mut pipeline = ProcessorPipeline::new();
    let processor = Arc::new(TestProcessor::new("test", "_modified"));
    pipeline.register(processor);

    let state = GcodeState::new();
    let command = GcodeCommand::new("G00 X10");

    let result = pipeline.process_command(&command, &state);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.len(), 1);
    assert_eq!(processed[0].command, "G00 X10_modified");
}

#[test]
fn test_process_command_chain() {
    let mut pipeline = ProcessorPipeline::new();

    let proc1 = Arc::new(TestProcessor::new("proc1", "_p1"));
    let proc2 = Arc::new(TestProcessor::new("proc2", "_p2"));

    pipeline.register(proc1);
    pipeline.register(proc2);

    let state = GcodeState::new();
    let command = GcodeCommand::new("G00");

    let result = pipeline.process_command(&command, &state);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.len(), 1);
    assert_eq!(processed[0].command, "G00_p1_p2");
}

#[test]
fn test_skip_command() {
    let mut pipeline = ProcessorPipeline::new();
    let processor = Arc::new(SkipProcessor);
    pipeline.register(processor);

    let state = GcodeState::new();
    let command = GcodeCommand::new("G00 X10");

    let result = pipeline.process_command(&command, &state);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.len(), 0);
}

#[test]
fn test_expand_command() {
    let mut pipeline = ProcessorPipeline::new();
    let processor = Arc::new(ExpandProcessor);
    pipeline.register(processor);

    let state = GcodeState::new();
    let command = GcodeCommand::new("G00");

    let result = pipeline.process_command(&command, &state);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.len(), 2);
    assert_eq!(processed[0].command, "G00_part1");
    assert_eq!(processed[1].command, "G00_part2");
}

#[test]
fn test_disabled_processor_skipped() {
    let mut pipeline = ProcessorPipeline::new();

    let proc1 = Arc::new(TestProcessor::new("proc1", "_p1"));
    let proc2 = Arc::new(TestProcessor::disabled("proc2"));
    let proc3 = Arc::new(TestProcessor::new("proc3", "_p3"));

    pipeline.register(proc1);
    pipeline.register(proc2);
    pipeline.register(proc3);

    let state = GcodeState::new();
    let command = GcodeCommand::new("G00");

    let result = pipeline.process_command(&command, &state);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.len(), 1);
    assert_eq!(processed[0].command, "G00_p1_p3");
}

#[test]
fn test_process_multiple_commands() {
    let mut pipeline = ProcessorPipeline::new();
    let processor = Arc::new(TestProcessor::new("proc", "_done"));
    pipeline.register(processor);

    let mut state = GcodeState::new();
    let commands = vec![
        GcodeCommand::new("G00 X10"),
        GcodeCommand::new("G01 Y20"),
        GcodeCommand::new("G02 Z30"),
    ];

    let result = pipeline.process_commands(&commands, &mut state);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.len(), 3);
    assert_eq!(processed[0].command, "G00 X10_done");
    assert_eq!(processed[1].command, "G01 Y20_done");
    assert_eq!(processed[2].command, "G02 Z30_done");
}

#[test]
fn test_processor_registry_creation() {
    let registry = ProcessorRegistry::new();
    assert_eq!(registry.list_registered().len(), 0);
}

#[test]
fn test_processor_registry_register() {
    let mut registry = ProcessorRegistry::new();

    registry.register("test_proc", || {
        Arc::new(TestProcessor::new("test_proc", ""))
    });

    assert_eq!(registry.list_registered().len(), 1);
    assert!(registry.list_registered().contains(&"test_proc"));
}

#[test]
fn test_processor_registry_create() {
    let mut registry = ProcessorRegistry::new();

    registry.register("test_proc", || {
        Arc::new(TestProcessor::new("test_proc", "_reg"))
    });

    let processor = registry.create("test_proc");
    assert!(processor.is_some());

    let processor = processor.unwrap();
    assert_eq!(processor.name(), "test_proc");
}

#[test]
fn test_processor_registry_create_nonexistent() {
    let registry = ProcessorRegistry::new();
    let processor = registry.create("nonexistent");
    assert!(processor.is_none());
}

#[test]
fn test_processor_registry_create_pipeline() {
    let mut registry = ProcessorRegistry::new();

    registry.register("proc1", || Arc::new(TestProcessor::new("proc1", "_p1")));
    registry.register("proc2", || Arc::new(TestProcessor::new("proc2", "_p2")));

    let pipeline = registry.create_pipeline(&["proc1", "proc2"]);
    assert!(pipeline.is_ok());

    let pipeline = pipeline.unwrap();
    assert_eq!(pipeline.processor_count(), 2);
}

#[test]
fn test_processor_registry_pipeline_with_invalid() {
    let mut registry = ProcessorRegistry::new();

    registry.register("proc1", || Arc::new(TestProcessor::new("proc1", "_p1")));

    let pipeline = registry.create_pipeline(&["proc1", "nonexistent", "proc1"]);
    assert!(pipeline.is_err());
}

#[test]
fn test_pipeline_clear() {
    let mut pipeline = ProcessorPipeline::new();

    let proc1 = Arc::new(TestProcessor::new("proc1", ""));
    let proc2 = Arc::new(TestProcessor::new("proc2", ""));

    pipeline.register(proc1);
    pipeline.register(proc2);
    assert_eq!(pipeline.processor_count(), 2);

    pipeline.clear();
    assert_eq!(pipeline.processor_count(), 0);
}

#[test]
fn test_pipeline_config() {
    let mut pipeline = ProcessorPipeline::new();

    let config = pipeline.config_mut();
    config.enabled = false;

    assert!(!pipeline.config().enabled);
}

#[test]
fn test_processor_registry_list_registered() {
    let mut registry = ProcessorRegistry::new();

    registry.register("proc1", || Arc::new(TestProcessor::new("proc1", "")));
    registry.register("proc2", || Arc::new(TestProcessor::new("proc2", "")));
    registry.register("proc3", || Arc::new(TestProcessor::new("proc3", "")));

    let list = registry.list_registered();
    assert_eq!(list.len(), 3);
    assert!(list.contains(&"proc1"));
    assert!(list.contains(&"proc2"));
    assert!(list.contains(&"proc3"));
}

#[test]
fn test_processor_state_update() {
    let mut pipeline = ProcessorPipeline::new();
    let processor = Arc::new(TestProcessor::new("proc", ""));
    pipeline.register(processor);

    let mut state = GcodeState::new();
    let commands = vec![
        GcodeCommand::new("G01"),
        GcodeCommand::new("G21"),
        GcodeCommand::new("F100"),
    ];

    let _result = pipeline.process_commands(&commands, &mut state);

    assert_eq!(state.motion_mode, 1); // G01
    assert_eq!(state.units_mode, 21); // G21
}

#[test]
fn test_processor_chain_with_expansion() {
    let mut pipeline = ProcessorPipeline::new();

    let proc1 = Arc::new(TestProcessor::new("proc1", "_p1"));
    let expand = Arc::new(ExpandProcessor);
    let proc3 = Arc::new(TestProcessor::new("proc3", "_p3"));

    pipeline.register(proc1);
    pipeline.register(expand);
    pipeline.register(proc3);

    let state = GcodeState::new();
    let command = GcodeCommand::new("G00");

    let result = pipeline.process_command(&command, &state);
    assert!(result.is_ok());

    let processed = result.unwrap();
    assert_eq!(processed.len(), 2);
    assert_eq!(processed[0].command, "G00_p1_part1_p3");
    assert_eq!(processed[1].command, "G00_p1_part2_p3");
}
