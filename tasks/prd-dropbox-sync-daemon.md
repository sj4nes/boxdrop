# Product Requirements Document: Dropbox Sync Daemon

## Introduction/Overview

This feature creates a fool-proof daemon that provides an alternative Dropbox synchronization solution. The primary problem it solves is the user's frustration with Dropbox's behavior of marking all files with the same timestamp (when they were first synced to a new computer) instead of preserving their original last-modified timestamps from Dropbox.

The daemon will provide bidirectional sync between local files and Dropbox, with special handling for the initial sync to preserve original file timestamps. It targets home users who are not CLI-savvy or experienced Linux users, requiring minimal technical knowledge to operate.

## Goals

1. **Preserve Original Timestamps**: On first sync, download files and update their last-modified timestamps to match Dropbox's recorded timestamps
2. **Bidirectional Sync**: Provide two-way synchronization between local files and Dropbox after initial sync
3. **Fool-Proof Operation**: Require minimal user intervention and technical knowledge
4. **Automatic Operation**: Run as a background daemon that starts automatically
5. **Conflict Resolution**: Provide user-friendly conflict resolution with backup preservation
6. **Cross-Distribution Compatibility**: Work as a portable executable across all Linux distributions

## User Stories

1. **As a home user**, I want the first sync of my files from Dropbox to have the same last-modified timestamp as they are recorded by Dropbox, not when they are synced onto my computer the first time, so that I can see when files were actually created or modified.

2. **As a home user**, I want the sync daemon to run automatically in the background without requiring technical knowledge, so that my files stay synchronized without manual intervention.

3. **As a home user**, I want to be notified when there are conflicts between my local files and Dropbox files, so that I can decide which version to keep without losing any data.

4. **As a home user**, I want to see progress during large sync operations, so that I know the system is working and can estimate completion time.

## Functional Requirements

1. **Initial Sync Behavior**: The system must download all files from Dropbox in batches and update each file's last-modified timestamp to match Dropbox's metadata using the `touch` command.

2. **Bidirectional Synchronization**: After initial sync, the system must monitor both local file changes and Dropbox changes, syncing in both directions.

3. **File System Monitoring**: The system must use file system events for real-time sync when the folder contains fewer than 20,000 files, and fall back to 5-minute polling for larger folders.

4. **Conflict Resolution**: When conflicts are detected, the system must:
   - Show a diff of file lengths and timestamps
   - Allow user to choose which version to keep
   - Automatically create a backup of the non-selected version
   - Present choices through a TUI interface

5. **User Interface**: The system must provide a TUI (Text User Interface) for initial configuration and conflict resolution.

6. **Automatic Startup**: The system must start automatically as a user service on boot.

7. **Progress Reporting**: During large operations, the system must show progress and estimated completion time.

8. **Desktop Notifications**: The system must provide desktop notifications for:
   - Sync completion status
   - API token expiration warnings
   - Critical errors

9. **Portable Distribution**: The system must be distributed as a single x86 executable that works across all Linux distributions.

10. **Configuration Management**: The system must store configuration in `~/.config/dropbox-sync-daemon/` and require:
    - Dropbox API token
    - Sync folder path (default: `~/Dropbox`)

11. **Logging**: The system must provide detailed debugging logs stored in `~/.local/share/dropbox-sync-daemon/logs/`.

12. **Service Management**: The system must provide CLI commands or tray menu for:
    - Starting/stopping the daemon
    - Pausing/resuming sync
    - Viewing sync status

## Non-Goals (Out of Scope)

1. **File Type Filtering**: No specific file types or folders will be excluded from sync
2. **Bandwidth Management**: No artificial limits on file sizes or bandwidth usage
3. **Advanced Conflict Resolution**: No automatic conflict resolution based on file content analysis
4. **Multi-Account Support**: Support for only one Dropbox account per installation
5. **GUI Configuration**: No graphical user interface for configuration (TUI only)
6. **Real-time Collaboration**: No special handling for collaborative editing scenarios

## Design Considerations

1. **TUI Design**: Use a simple, intuitive text-based interface with clear navigation and help text
2. **Progress Display**: Show progress bars, file counts, and time estimates during operations
3. **Error Messages**: Provide clear, non-technical error messages with suggested actions
4. **Notification Integration**: Use standard desktop notification systems (freedesktop.org)
5. **Service Integration**: Follow systemd user service conventions for automatic startup

## Technical Considerations

1. **Dropbox API Integration**: Must use Dropbox API v2 for file operations and metadata retrieval
2. **File System Events**: Use inotify for Linux file system monitoring with fallback to polling
3. **Timestamp Preservation**: Use `utime` system calls to modify file timestamps
4. **Conflict Detection**: Compare file sizes, modification times, and optionally MD5 hashes
5. **Backup Strategy**: Create timestamped backups in a `.conflicts` subdirectory
6. **Portable Binary**: Use static linking or include all dependencies in the executable
7. **Configuration Storage**: Use JSON or YAML format for configuration files

## Success Metrics

1. **Timestamp Accuracy**: 100% of files should have correct timestamps after initial sync
2. **Sync Reliability**: 99.9% uptime for the daemon service
3. **User Satisfaction**: Users should not need to manually intervene in sync operations more than once per month
4. **Performance**: Initial sync should complete within 2 hours for 10GB of data on standard home internet
5. **Error Rate**: Less than 0.1% of files should encounter sync errors

## Open Questions

1. **Tray Menu vs CLI**: Should the daemon provide a system tray menu or rely on CLI commands for user interaction?
2. **Backup Retention**: How long should conflict backups be retained before automatic cleanup?
3. **API Rate Limiting**: What strategy should be used when approaching Dropbox API rate limits?
4. **Large File Handling**: Should large files (>100MB) be handled differently during sync?
5. **Offline Mode**: How should the daemon behave when offline for extended periods? 