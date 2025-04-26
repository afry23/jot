// $lib/utils/logger.ts
import { invoke } from "@tauri-apps/api/core";
import { LogLevel, DEFAULT_LOG_LEVEL } from "./logLevels";
import { writable, get } from "svelte/store";

// Store for log level configuration
export const logLevelStore = writable<LogLevel>(DEFAULT_LOG_LEVEL);

class Logger {
  private enabled: boolean = true;
  private buffer: string[] = [];
  private maxBufferSize: number = 100;

  constructor() {
    // Initialize buffer
    this.buffer = [];

    // Send buffered logs to backend on window close
    if (typeof window !== "undefined") {
      window.addEventListener("beforeunload", () => {
        this.flushLogs();
      });

      // Periodically flush logs (every 30 seconds)
      setInterval(() => this.flushLogs(), 30000);

      // Initialize log level from local storage if available
      this.initLogLevel();
    }
  }

  // Initialize log level from local storage
  private async initLogLevel() {
    try {
      const savedLevel = localStorage.getItem("jot-log-level");
      if (savedLevel !== null) {
        const level = parseInt(savedLevel, 10);
        if (
          !isNaN(level) &&
          level >= LogLevel.TRACE &&
          level <= LogLevel.NONE
        ) {
          logLevelStore.set(level);
        }
      }

      // Also try to get log level from backend
      const backendLevel = await invoke<number>("get_log_level");
      if (backendLevel >= LogLevel.TRACE && backendLevel <= LogLevel.NONE) {
        logLevelStore.set(backendLevel);
        localStorage.setItem("jot-log-level", backendLevel.toString());
      }
    } catch (error) {
      console.error("Failed to initialize log level:", error);
    }
  }

  // Enable or disable logging
  public setEnabled(enabled: boolean): void {
    this.enabled = enabled;
  }

  // Set the log level
  public async setLogLevel(level: LogLevel): Promise<void> {
    logLevelStore.set(level);
    localStorage.setItem("jot-log-level", level.toString());

    // Also update backend log level
    try {
      await invoke("set_log_level", { level });
    } catch (error) {
      console.error("Failed to set backend log level:", error);
    }
  }

  // Get current log level
  public getLogLevel(): LogLevel {
    return get(logLevelStore);
  }

  // Log a trace message (most verbose)
  public trace(message: string, context?: any): void {
    this.log(LogLevel.TRACE, message, context);
  }

  // Log a debug message
  public debug(message: string, context?: any): void {
    this.log(LogLevel.DEBUG, message, context);
  }

  // Log an info message
  public info(message: string, context?: any): void {
    this.log(LogLevel.INFO, message, context);
  }

  // Log a warning
  public warn(message: string, context?: any): void {
    this.log(LogLevel.WARN, message, context);
  }

  // Log an error
  public error(message: string, error?: any): void {
    this.log(LogLevel.ERROR, message, error);
  }

  // Internal log method
  private log(level: LogLevel, message: string, context?: any): void {
    if (!this.enabled) return;

    // Skip if level is below configured threshold
    const currentLogLevel = get(logLevelStore);
    if (level < currentLogLevel) return;

    const timestamp = new Date().toISOString();
    const levelStr = LogLevel[level]; // Convert enum to string
    const formattedContext = context ? ` | ${JSON.stringify(context)}` : "";
    const logMessage = `[${timestamp}] [${levelStr}] ${message}${formattedContext}`;

    // Add to internal buffer
    this.buffer.push(logMessage);

    // Keep buffer size in check
    if (this.buffer.length > this.maxBufferSize) {
      this.flushLogs();
    }

    // Also log to console for development
    switch (level) {
      case LogLevel.TRACE:
      case LogLevel.DEBUG:
        console.debug(message, context);
        break;
      case LogLevel.INFO:
        console.info(message, context);
        break;
      case LogLevel.WARN:
        console.warn(message, context);
        break;
      case LogLevel.ERROR:
        console.error(message, context);
        break;
    }
  }

  // Send logs to backend
  public async flushLogs(): Promise<void> {
    if (this.buffer.length === 0) return;

    try {
      // If we're in a Tauri context, send logs to backend
      if (typeof invoke === "function") {
        const logs = [...this.buffer];
        this.buffer = [];

        await invoke("log_from_frontend", {
          logs,
        });
      }
    } catch (error) {
      console.error("Failed to send logs to backend:", error);
      // Keep the logs in buffer to try again later
      // but avoid duplicates
      const uniqueLogs = [...new Set([...this.buffer])];
      this.buffer = uniqueLogs.slice(-this.maxBufferSize);
    }
  }

  // Get all logs from backend
  public async getLatestLogs(maxLines: number = 100): Promise<string> {
    try {
      return await invoke<string>("get_latest_logs", { maxLines });
    } catch (error) {
      console.error("Failed to get logs:", error);
      return "Failed to retrieve logs";
    }
  }

  // Get all log files
  public async getLogFiles(): Promise<string[]> {
    try {
      return await invoke<string[]>("list_log_files");
    } catch (error) {
      console.error("Failed to list log files:", error);
      return [];
    }
  }
}

// Create a singleton instance
export const logger = new Logger();
