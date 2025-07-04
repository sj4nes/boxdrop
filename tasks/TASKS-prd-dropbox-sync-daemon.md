# Task List: Dropbox Sync Daemon Implementation

## Tasks

- [ ] 1.0 Core Dropbox API Integration and File Operations
  - [ ] 1.1 Implement Dropbox API v2 client with authentication and token management
  - [ ] 1.2 Create file metadata retrieval functions (timestamps, sizes, paths)
  - [ ] 1.3 Implement file download functionality with progress tracking
  - [ ] 1.4 Implement file upload functionality with conflict detection
  - [ ] 1.5 Create batch operations for efficient file transfers
  - [ ] 1.6 Implement timestamp preservation using utime system calls
  - [ ] 1.7 Add API rate limiting and error handling with retry logic
- [ ] 2.0 File System Monitoring and Synchronization Engine
  - [ ] 2.1 Implement inotify-based file system monitoring for real-time changes
  - [ ] 2.2 Create polling fallback mechanism for folders with >20,000 files
  - [ ] 2.3 Build bidirectional sync logic with change detection
  - [ ] 2.4 Implement initial sync behavior with timestamp preservation
  - [ ] 2.5 Create file change queue and processing system
  - [ ] 2.6 Add sync state management and persistence
  - [ ] 2.7 Implement large file handling (>100MB) with progress reporting
- [ ] 3.0 User Interface and Configuration Management
  - [ ] 3.1 Design and implement TUI for initial configuration setup
  - [ ] 3.2 Create progress display with bars, file counts, and time estimates
  - [ ] 3.3 Implement desktop notifications for sync status and errors
  - [ ] 3.4 Build configuration file management (JSON/YAML) in ~/.config/dropbox-sync-daemon/
  - [ ] 3.5 Create CLI commands for daemon management (start/stop/pause/status)
  - [ ] 3.6 Implement logging system with debug logs in ~/.local/share/dropbox-sync-daemon/logs/
  - [ ] 3.7 Add error message handling with clear, non-technical descriptions
- [ ] 4.0 Daemon Service and System Integration
  - [ ] 4.1 Create systemd user service configuration for automatic startup
  - [ ] 4.2 Implement daemon process management and signal handling
  - [ ] 4.3 Build service installation and uninstallation scripts
  - [ ] 4.4 Create portable executable with static linking for cross-distribution compatibility
  - [ ] 4.5 Implement graceful shutdown and restart procedures
  - [ ] 4.6 Add offline mode handling and reconnection logic
  - [ ] 4.7 Create system tray menu or CLI-based user interaction system
- [ ] 5.0 Conflict Resolution and Backup System
  - [ ] 5.1 Implement conflict detection using file sizes, timestamps, and MD5 hashes
  - [ ] 5.2 Create TUI interface for conflict resolution with file diff display
  - [ ] 5.3 Build automatic backup system with timestamped files in .conflicts directory
  - [ ] 5.4 Implement user choice handling (local vs remote version selection)
  - [ ] 5.5 Add backup retention and cleanup policies
  - [ ] 5.6 Create conflict resolution state persistence
  - [ ] 5.7 Implement batch conflict resolution for multiple file conflicts

### Relevant Files

- `src/main.rs` - Main application entry point and daemon process management
- `src/dropbox/client.rs` - Dropbox API v2 client implementation with authentication
- `src/dropbox/operations.rs` - File operations (upload, download, metadata retrieval)
- `src/sync/engine.rs` - Core synchronization engine with bidirectional sync logic
- `src/sync/monitor.rs` - File system monitoring with inotify and polling fallback
- `src/sync/initial_sync.rs` - Initial sync implementation with timestamp preservation
- `src/ui/tui.rs` - Text User Interface for configuration and conflict resolution
- `src/ui/progress.rs` - Progress display and reporting components
- `src/ui/notifications.rs` - Desktop notification system integration
- `src/config/manager.rs` - Configuration file management and validation
- `src/config/schema.rs` - Configuration schema definitions (JSON/YAML)
- `src/service/daemon.rs` - Daemon process management and systemd integration
- `src/service/installer.rs` - Service installation and system integration scripts
- `src/conflict/resolver.rs` - Conflict detection and resolution logic
- `src/conflict/backup.rs` - Backup system for conflict preservation
- `src/utils/timestamps.rs` - Timestamp manipulation utilities using utime
- `src/utils/logging.rs` - Logging system with file rotation and debug levels
- `src/utils/cli.rs` - Command-line interface for daemon management
- `systemd/dropbox-sync-daemon.service` - Systemd user service configuration
- `build.rs` - Build script for static linking and portable executable creation
- `Cargo.toml` - Rust project dependencies and build configuration 