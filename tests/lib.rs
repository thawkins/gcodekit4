//! Integration test modules for gcodekit4

pub mod common;
mod firmware_connection_watch;
mod firmware_file_service;
mod firmware_fluidnc_capabilities;
mod firmware_fluidnc_command_creator;
mod firmware_fluidnc_response_parser;
mod firmware_g2core_capabilities;
mod firmware_grbl_command_creator;
mod firmware_grbl_override_manager;
mod firmware_grbl_utils;
mod firmware_override_manager;
mod firmware_settings;
mod firmware_smoothieware_capabilities;
mod firmware_smoothieware_command_creator;
mod firmware_smoothieware_response_parser;
mod phase7_integration;
mod processing_advanced_features;
mod processing_stats;
mod test_console_listener;
mod test_console_output_debug;
mod test_materials_database;
mod test_svg_import_integration;
mod test_tools_palette;

