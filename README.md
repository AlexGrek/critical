# Disclaimer

This text is AI-generated and not revewed by humans yet. **Do not trust it**

# crit-cli

A powerful command-line tool built with Rust, supporting multiple platforms and architectures.

## 🚀 Quick Start

### Installation

Install crit-cli with a single command:

```bash
curl -fsSL https://critical.dcommunity.space/install.sh | bash
```

Or using wget:

```bash
wget -qO- https://critical.dcommunity.space/install.sh | bash
```

### Usage

```bash
cr1t [command] [options]
```

## CLI (`cr1t`)

`cr1t` is a gitops-style CLI (similar to `kubectl`) that serves as a full alternative to the web frontend. It communicates with the Critical backend API over HTTP.

### Authentication

Login with username and password. The JWT is stored in `~/.cr1tical/context.yaml` and reused for subsequent commands.

```bash
# Interactive login (prompts for URL, username, password)
cr1t login

# Non-interactive
cr1t login --url https://critical.example.com --user alice
```

Registration is not supported from the CLI. Use the web frontend or API directly.

> **Note:** Additional login methods (API keys, SSO, etc.) are planned for future releases.

### Context Management

Contexts work like kubeconfigs — you can authenticate against multiple servers and switch between them.

```bash
cr1t context list           # Show all contexts
cr1t context use <name>     # Switch active context
```

Context file location: `~/.cr1tical/context.yaml`

### Building the CLI

```bash
cargo build --bin cr1t      # Development build
```

## 🛠️ Development

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable version)
- [Docker](https://www.docker.com/) (for cross-compilation)
- [Git](https://git-scm.com/) (for version information)

### Project Structure

```
├── backend/           # Rust source code (Cargo workspace)
├── build/             # Build output directory
│   └── crit-cli/      # Cross-compiled binaries
│       ├── amd64/
│       ├── arm64/
│       ├── 386/
│       └── arm/
├── Makefile           # Build system
└── README.md          # This file
```

### Development Build

For quick development builds on your current platform:

```bash
make dev
```

This runs `cargo build --bin crit-cli` in the `backend/` directory.

## 🏗️ Build System

The project uses a comprehensive Makefile-based build system that supports cross-compilation for multiple platforms and architectures.

### Supported Platforms

| Platform | Architectures | Status |
|----------|---------------|---------|
| Linux    | amd64, 386, arm64, arm | ✅ |
| macOS    | amd64, arm64 | ✅ |
| Windows  | amd64, 386, arm64 | ✅ |

### Build Targets

#### Setup and Dependencies

```bash
make help           # Show all available targets
make check-deps     # Check build dependencies
make install-cross  # Install cross-compilation tools
make setup-targets  # Add required Rust targets
make info          # Show build configuration
```

#### Platform-Specific Builds

```bash
# Build all platforms
make build-all

# Build by platform
make build-linux    # All Linux targets
make build-darwin   # All macOS targets  
make build-windows  # All Windows targets
make build-unix     # Linux + macOS only

# Build specific targets
make build-amd64-linux
make build-arm64-linux
make build-386-linux
make build-arm-linux
make build-amd64-darwin
make build-arm64-darwin
make build-amd64-windows
make build-386-windows
make build-arm64-windows
```

#### Development Targets

```bash
make dev           # Quick development build (current platform)
make test-build    # Test build on current platform
make clean         # Clean build directory
```

#### Release Targets

```bash
make release       # Full release build with archives
make verify        # Verify all built binaries
make archive       # Create release archives
```

#### CI/CD Targets

```bash
make docker-build  # Build using Docker (for CI/CD)
```

### Build Output

All binaries are placed in the `build/crit-cli/` directory structure:

```
build/crit-cli/
├── amd64/
│   ├── linux/crit
│   ├── darwin/crit
│   └── windows/crit.exe
├── arm64/
│   ├── linux/crit
│   ├── darwin/crit
│   └── windows/crit.exe
├── 386/
│   ├── linux/crit
│   └── windows/crit.exe
└── arm/
    └── linux/crit
```

### Release Archives

The `make archive` target creates compressed archives in `build/crit-cli/releases/`:

- **Linux/macOS:** `crit-{version}-{arch}-{os}.tar.gz`
- **Windows:** `crit-{version}-{arch}-{os}.zip`

## 🔧 Build Process Details

### Cross-Compilation

The build system uses [cross](https://github.com/cross-rs/cross) for cross-compilation, which provides Docker-based toolchains for each target platform.

### Rust Targets

The following Rust targets are supported:

| Target | Architecture | OS |
|--------|-------------|-----|
| `x86_64-unknown-linux-gnu` | amd64 | linux |
| `i686-unknown-linux-gnu` | 386 | linux |
| `aarch64-unknown-linux-gnu` | arm64 | linux |
| `armv7-unknown-linux-gnueabihf` | arm | linux |
| `x86_64-apple-darwin` | amd64 | darwin |
| `aarch64-apple-darwin` | arm64 | darwin |
| `x86_64-pc-windows-gnu` | amd64 | windows |
| `i686-pc-windows-gnu` | 386 | windows |
| `aarch64-pc-windows-msvc` | arm64 | windows |

### Build Flags

The build system uses the following Cargo flags:

- `--release` - Optimized release build
- `--bin crit-cli` - Build the crit-cli binary specifically

### Version Information

Version information is automatically extracted from Git:

- **Version:** Git tags or "dev" if no tags
- **Build Date:** UTC timestamp
- **Git Commit:** Short commit hash

## 🚀 Usage Examples

### Basic Build Workflow

```bash
# Check dependencies
make check-deps

# Build for all platforms
make build-all

# Verify builds
make verify

# Create release archives
make archive
```

### Development Workflow

```bash
# Quick development build
make dev

# Test the build
./backend/target/debug/crit-cli --help

# Clean and rebuild
make clean
make dev
```

### CI/CD Integration

```bash
# Full release build (suitable for CI/CD)
make release

# Docker-based build (for consistent environments)
make docker-build
```

### Platform-Specific Development

```bash
# Build only for Linux
make build-linux

# Build for a specific target
make build-amd64-linux

# Verify specific builds
ls -la build/crit-cli/amd64/linux/
```

## 📦 Distribution

The installer script automatically detects the user's platform and downloads the appropriate binary from:

```
https://critical.dcommunity.space/crit-cli/{ARCH}/{OS}/crit
```

Where:
- `{ARCH}` is one of: `amd64`, `arm64`, `386`, `arm`
- `{OS}` is one of: `linux`, `darwin`, `windows`

## 🔍 Troubleshooting

### Common Issues

#### Docker Permission Errors

```bash
# Add user to docker group (Linux)
sudo usermod -aG docker $USER
# Log out and back in

# Or use sudo for Docker commands
sudo make docker-build
```

#### Missing Rust Targets

```bash
# Install missing targets
make setup-targets

# Or manually install a specific target
rustup target add x86_64-unknown-linux-gnu
```

#### Cross-Compilation Failures

```bash
# Update cross tool
cargo install cross --git https://github.com/cross-rs/cross

# Clean and retry
make clean
make build-all
```

### Debug Information

```bash
# Show build configuration
make info

# Check dependencies
make check-deps

# Verify builds
make verify
```

### Build Verification

After building, verify your binaries:

```bash
# Check all built binaries
make verify

# Test a specific binary
./build/crit-cli/amd64/linux/crit --version

# Check file sizes
find build/crit-cli -name "crit*" -exec ls -lh {} \;
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test with `make dev`
5. Build all targets with `make build-all`
6. Submit a pull request

## 📄 License

[Add your license information here]

## 🆘 Support

For issues and questions:

- GitHub Issues: [Your repository URL]
- Documentation: [Your docs URL]
- Community: [Your community URL]

---

**Note:** This build system is designed for cross-platform compatibility and uses Docker-based toolchains to ensure consistent builds across different environments.