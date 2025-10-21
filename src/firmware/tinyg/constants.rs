//! TinyG Protocol Constants and Capabilities
//!
//! This module defines all TinyG-specific constants, capabilities, status codes,
//! and protocol parameters according to the TinyG protocol specification.

// TinyG Version Info
/// Minimum supported TinyG version
pub const TINYG_MIN_VERSION: &str = "440.00";

/// TinyG standard version string pattern
pub const TINYG_VERSION_PREFIX: &str = "TinyG";

// TinyG Capabilities
/// Maximum number of axes supported by TinyG
pub const TINYG_MAX_AXES: u8 = 4;

/// Default buffer size for TinyG
pub const TINYG_DEFAULT_BUFFER_SIZE: usize = 64;

/// TinyG block size (maximum command length)
pub const TINYG_MAX_BLOCK_SIZE: usize = 256;

/// Default serial communication speed (baud rate)
pub const TINYG_DEFAULT_BAUD_RATE: u32 = 115200;

/// Alternative baud rates supported
pub const TINYG_ALT_BAUD_RATES: &[u32] = &[9600, 19200, 38400, 57600, 230400];

// TinyG Status States
/// Machine is IDLE
pub const STATUS_IDLE: &str = "Idle";

/// Machine is RUNNING
pub const STATUS_RUN: &str = "Run";

/// Machine is in HOLD (feed hold)
pub const STATUS_HOLD: &str = "Hold";

/// Machine is in JOG mode
pub const STATUS_JOG: &str = "Jog";

/// Machine has an ALARM
pub const STATUS_ALARM: &str = "Alarm";

/// Machine is in CHECK mode
pub const STATUS_CHECK: &str = "Check";

/// Machine door is open
pub const STATUS_DOOR: &str = "Door";

/// Machine is SLEEPING
pub const STATUS_SLEEP: &str = "Sleep";

/// Machine is SHUTTING DOWN
pub const STATUS_SHUTDOWN: &str = "Shutdown";

/// Machine is in PROBE mode
pub const STATUS_PROBE: &str = "Probe";

// TinyG JSON Response Codes
/// Success response
pub const RESPONSE_SUCCESS: u8 = 0;

/// Error response
pub const RESPONSE_ERROR: u8 = 1;

/// Report status response
pub const RESPONSE_STATUS: u8 = 2;

// TinyG Real-Time Commands
/// Query status (use JSON query)
pub const CMD_QUERY_STATUS: u8 = b'?';

/// Feed hold command
pub const CMD_FEED_HOLD: u8 = b'!';

/// Cycle start/resume command
pub const CMD_CYCLE_START: u8 = b'~';

/// Soft reset command (Ctrl+X = 0x18)
pub const CMD_SOFT_RESET: u8 = 0x18;

// TinyG JSON Settings
/// Motion report field
pub const SETTING_REPORT: &str = "sr";

/// Status report mask
pub const SETTING_STATUS_REPORT_MASK: &str = "srx";

/// Queue report enabled
pub const SETTING_QUEUE_REPORT: &str = "qr";

// TinyG Axes
/// X axis code
pub const AXIS_X: &str = "x";

/// Y axis code
pub const AXIS_Y: &str = "y";

/// Z axis code
pub const AXIS_Z: &str = "z";

/// A axis code
pub const AXIS_A: &str = "a";

// TinyG Status Report Fields
/// Position field in status report
pub const STATUS_POS: &str = "pos";

/// Machine position field in status report
pub const STATUS_MPOS: &str = "mpos";

/// Work position field in status report
pub const STATUS_WPOS: &str = "wpos";

/// Line number field in status report
pub const STATUS_LINE: &str = "line";

/// Feed rate field in status report
pub const STATUS_FEED: &str = "feed";

/// Speed field in status report
pub const STATUS_SPEED: &str = "speed";

/// State field in status report
pub const STATUS_STATE: &str = "stat";

/// Units field in status report (0 = mm, 1 = inches)
pub const STATUS_UNITS: &str = "unit";

/// Coordinate offset field in status report
pub const STATUS_OFFSET: &str = "offset";

// TinyG Coordinate Systems (G54-G59)
/// G54 - Primary coordinate system offset
pub const COORD_SYSTEM_G54: u8 = 1;
/// G55 - Secondary coordinate system offset
pub const COORD_SYSTEM_G55: u8 = 2;
/// G56 - Tertiary coordinate system offset
pub const COORD_SYSTEM_G56: u8 = 3;
/// G57 - Quaternary coordinate system offset
pub const COORD_SYSTEM_G57: u8 = 4;
/// G58 - Quinary coordinate system offset
pub const COORD_SYSTEM_G58: u8 = 5;
/// G59 - Senary coordinate system offset
pub const COORD_SYSTEM_G59: u8 = 6;

// TinyG Error Codes (non-exhaustive)
/// Generic error
pub const ERROR_GENERIC: u16 = 1;

/// Expected command letter
pub const ERROR_EXPECTED_COMMAND_LETTER: u16 = 2;

/// Bad number format
pub const ERROR_BAD_NUMBER_FORMAT: u16 = 3;

/// Input exceeds max length
pub const ERROR_INPUT_EXCEEDS_MAX_LENGTH: u16 = 4;

/// Input buffer overflow
pub const ERROR_INPUT_BUFFER_OVERFLOW: u16 = 5;

/// Invalid or missing parameter
pub const ERROR_INVALID_MISSING_PARAMETER: u16 = 6;

/// Invalid parameter value
pub const ERROR_INVALID_PARAMETER_VALUE: u16 = 7;

/// Device is busy
pub const ERROR_DEVICE_IS_BUSY: u16 = 8;

/// Command not accepted
pub const ERROR_COMMAND_NOT_ACCEPTED: u16 = 9;

// TinyG Alarm Codes (non-exhaustive)
/// Power up or reset
pub const ALARM_POWER_UP: u16 = 1;

/// Limit switch hit during motion
pub const ALARM_LIMIT_SWITCH_HIT: u16 = 2;

/// Probe fail
pub const ALARM_PROBE_FAIL: u16 = 3;

/// Soft limit exceeded
pub const ALARM_SOFT_LIMIT: u16 = 4;

/// System error
pub const ALARM_SYSTEM_ERROR: u16 = 5;

/// Serial RX buffer overrun
pub const ALARM_SERIAL_RX_OVERRUN: u16 = 6;

/// Hardlimit abort during homing
pub const ALARM_HARDLIMIT_DURING_HOMING: u16 = 7;

/// Step fail (motor stall/loss of position)
pub const ALARM_STEP_FAIL: u16 = 8;

/// SPI comm lost
pub const ALARM_SPI_COMM_LOST: u16 = 9;

/// Guard band limit exceeded
pub const ALARM_GUARD_BAND_LIMIT: u16 = 10;

/// Motor driver fault
pub const ALARM_MOTOR_DRIVER_FAULT: u16 = 11;

// TinyG Request/Response Indicators
/// Line number indicator for responses
pub const LINE_NUMBER_PREFIX: &str = "n";

/// NACK response indicator
pub const NACK_RESPONSE: &str = "nack";

/// OK response indicator
pub const OK_RESPONSE: &str = "ok";
