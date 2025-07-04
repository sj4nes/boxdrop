# BoxDrop Sync Daemon

A fool-proof Dropbox synchronization daemon that preserves original file timestamps during initial sync.

## Features

- **Timestamp Preservation**: Maintains original file timestamps from Dropbox during initial sync
- **Bidirectional Sync**: Two-way synchronization between local files and Dropbox
- **Cross-Platform**: Portable Linux binary that works across all distributions
- **Automatic Operation**: Runs as a background daemon with minimal user intervention
- **Conflict Resolution**: User-friendly conflict resolution with backup preservation

## Building

### Local Development (macOS)

```bash
# Install dependencies
cargo build

# Run tests
cargo test

# Build release version
cargo build --release
```

### Cross-Compilation for Linux

The project uses GitHub Actions to build portable Linux binaries. The workflow:

1. Builds on Ubuntu with `cross` for `x86_64-unknown-linux-musl`
2. Creates a statically linked binary that works across Linux distributions
3. Uploads artifacts for each push/PR
4. Creates release assets when a release is published

### Manual Cross-Build (Linux only)

If you're on Linux and want to build manually:

```bash
# Install cross
cargo install cross

# Build for Linux
cross build --target x86_64-unknown-linux-musl --release
```

## Project Structure

```
src/
├── dropbox/          # Dropbox API client and operations
├── sync/            # Synchronization engine
├── ui/              # Text User Interface components
├── config/          # Configuration management
├── service/         # Daemon service and system integration
├── conflict/        # Conflict resolution and backup
└── utils/           # Utility functions (timestamps, logging, CLI)
```

## Configuration

The daemon stores configuration in `~/.config/dropbox-sync-daemon/config.json`:

```json
{
  "dropbox_token": "your_dropbox_api_token",
  "sync_folder": "/home/user/Dropbox",
  "polling_interval": 300,
  "max_files_for_inotify": 20000,
  "large_file_threshold": 104857600,
  "log_level": "info"
}
```

## Development Status

- [x] Project structure and cross-compilation setup
- [x] Dropbox API v2 client with authentication
- [x] Configuration management
- [ ] File system monitoring and sync engine
- [ ] User interface and progress display
- [ ] Daemon service and system integration
- [ ] Conflict resolution and backup system

## License

MIT License 