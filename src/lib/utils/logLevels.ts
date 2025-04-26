// $lib/utils/logLevels.ts
export enum LogLevel {
  TRACE = 0,
  DEBUG = 1,
  INFO = 2,
  WARN = 3,
  ERROR = 4,
  NONE = 5,
}

// Map log level enum to string representations
export const logLevelNames: Record<LogLevel, string> = {
  [LogLevel.TRACE]: "TRACE",
  [LogLevel.DEBUG]: "DEBUG",
  [LogLevel.INFO]: "INFO",
  [LogLevel.WARN]: "WARN",
  [LogLevel.ERROR]: "ERROR",
  [LogLevel.NONE]: "NONE",
};

// User-friendly names for the UI
export const logLevelLabels: Record<LogLevel, string> = {
  [LogLevel.TRACE]: "Trace (Most Verbose)",
  [LogLevel.DEBUG]: "Debug",
  [LogLevel.INFO]: "Info (Normal)",
  [LogLevel.WARN]: "Warning",
  [LogLevel.ERROR]: "Error Only",
  [LogLevel.NONE]: "Disabled",
};

// Default log level
export const DEFAULT_LOG_LEVEL = LogLevel.INFO;
