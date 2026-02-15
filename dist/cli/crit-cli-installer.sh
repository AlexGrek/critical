#!/bin/bash

# crit-cli installer script
# Usage: curl -fsSL https://critical.dcommunity.space/crit-cli/install.sh | bash
# or: wget -qO- https://critical.dcommunity.space/crit-cli/install.sh | bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="crit"
BASE_URL="https://critical.dcommunity.space/crit-cli"
INSTALL_DIR=""
TEMP_DIR=$(mktemp -d)
GITHUB_REPO="your-org/crit-cli" # Optional: for version checking

# Cleanup function
cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        log_warn "Running as root. Installing system-wide."
        return 0
    else
        return 1
    fi
}

# Detect operating system
detect_os() {
    local os=""

    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        os="linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        os="darwin"
    elif [[ "$OSTYPE" == "freebsd"* ]]; then
        os="freebsd"
    else
        log_error "Unsupported operating system: $OSTYPE"
        exit 1
    fi

    echo "$os"
}

# Detect architecture
detect_arch() {
    local arch=""
    local machine=$(uname -m)

    case "$machine" in
    x86_64 | amd64)
        arch="amd64"
        ;;
    i686 | i386)
        arch="386"
        ;;
    aarch64 | arm64)
        arch="arm64"
        ;;
    armv7l | armv6l)
        arch="arm"
        ;;
    *)
        log_error "Unsupported architecture: $machine"
        exit 1
        ;;
    esac

    echo "$arch"
}

# Get installation directory
get_install_dir() {
    local os="$1"
    local is_root="$2"

    if [[ "$is_root" == "true" ]]; then
        # System-wide installation
        case "$os" in
        linux)
            echo "/usr/local/bin"
            ;;
        darwin)
            echo "/usr/local/bin"
            ;;
        *)
            echo "/usr/local/bin"
            ;;
        esac
    else
        # User installation
        case "$os" in
        linux | darwin)
            # Create ~/.local/bin if it doesn't exist
            local user_bin="$HOME/.local/bin"
            mkdir -p "$user_bin"
            echo "$user_bin"
            ;;
        *)
            local user_bin="$HOME/.local/bin"
            mkdir -p "$user_bin"
            echo "$user_bin"
            ;;
        esac
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Download file
download_file() {
    local url="$1"
    local output="$2"

    log_info "Downloading from: $url"

    if command_exists curl; then
        curl -fsSL --connect-timeout 10 --max-time 300 "$url" -o "$output"
    elif command_exists wget; then
        wget -q --timeout=10 --tries=3 "$url" -O "$output"
    else
        log_error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Verify download
verify_download() {
    local file="$1"

    if [[ ! -f "$file" ]]; then
        log_error "Downloaded file not found: $file"
        exit 1
    fi

    if [[ ! -s "$file" ]]; then
        log_error "Downloaded file is empty: $file"
        exit 1
    fi

    # Check if file is a binary (not HTML error page)
    if file "$file" | grep -q "text\|HTML"; then
        log_error "Downloaded file appears to be text/HTML, not a binary"
        log_error "This might indicate a download error or wrong URL"
        exit 1
    fi
}

# Install binary
install_binary() {
    local source="$1"
    local target="$2"
    local is_root="$3"

    log_info "Installing $BINARY_NAME to $target"

    # Copy binary
    if [[ "$is_root" == "true" ]]; then
        cp "$source" "$target"
        chmod 755 "$target"
        chown root:root "$target" 2>/dev/null || true
    else
        cp "$source" "$target"
        chmod 755 "$target"
    fi

    # Verify installation
    if [[ -x "$target" ]]; then
        log_success "Binary installed successfully"
    else
        log_error "Failed to install binary"
        exit 1
    fi
}

# Add to PATH
add_to_path() {
    local install_dir="$1"
    local is_root="$2"

    # Check if already in PATH
    if echo "$PATH" | grep -q "$install_dir"; then
        log_info "Install directory already in PATH"
        return 0
    fi

    if [[ "$is_root" == "true" ]]; then
        # System-wide installation, usually already in PATH
        log_info "System-wide installation completed"
        return 0
    fi

    # User installation - add to shell profile
    local shell_profile=""
    local shell_name=$(basename "$SHELL")

    case "$shell_name" in
    bash)
        if [[ -f "$HOME/.bashrc" ]]; then
            shell_profile="$HOME/.bashrc"
        elif [[ -f "$HOME/.bash_profile" ]]; then
            shell_profile="$HOME/.bash_profile"
        fi
        ;;
    zsh)
        shell_profile="$HOME/.zshrc"
        ;;
    fish)
        # Fish shell has different syntax
        local fish_config="$HOME/.config/fish/config.fish"
        if [[ -f "$fish_config" ]]; then
            if ! grep -q "$install_dir" "$fish_config"; then
                echo "set -gx PATH $install_dir \$PATH" >>"$fish_config"
                log_success "Added to Fish shell PATH"
            fi
        fi
        return 0
        ;;
    *)
        shell_profile="$HOME/.profile"
        ;;
    esac

    if [[ -n "$shell_profile" ]]; then
        local path_export="export PATH=\"$install_dir:\$PATH\""

        if [[ -f "$shell_profile" ]] && grep -q "$install_dir" "$shell_profile"; then
            log_info "PATH already configured in $shell_profile"
        else
            echo "" >>"$shell_profile"
            echo "# Added by crit-cli installer" >>"$shell_profile"
            echo "$path_export" >>"$shell_profile"
            log_success "Added to PATH in $shell_profile"
            log_warn "Please restart your shell or run: source $shell_profile"
        fi
    fi
}

# Check if binary works
test_installation() {
    local install_path="$1"

    log_info "Testing installation..."

    if "$install_path" --version >/dev/null 2>&1 || "$install_path" --help >/dev/null 2>&1; then
        log_success "Installation test passed"
        return 0
    else
        log_warn "Installation test failed (binary might not support --version or --help)"
        log_info "Binary is installed at: $install_path"
        return 0
    fi
}

# Main installation function
main() {
    log_info "Starting crit-cli installation..."

    # Check prerequisites
    if ! command_exists uname; then
        log_error "uname command not found"
        exit 1
    fi

    if ! command_exists file; then
        log_error "file command not found. Please install it first."
        exit 1
    fi

    # Detect system
    local os=$(detect_os)
    local arch=$(detect_arch)
    local is_root="false"

    if check_root; then
        is_root="true"
    fi

    log_info "Detected OS: $os"
    log_info "Detected Architecture: $arch"
    log_info "Installation mode: $([ "$is_root" == "true" ] && echo "system-wide" || echo "user")"

    # Get installation directory
    INSTALL_DIR=$(get_install_dir "$os" "$is_root")
    log_info "Installation directory: $INSTALL_DIR"

    # Create installation directory if it doesn't exist
    if [[ "$is_root" == "true" ]]; then
        mkdir -p "$INSTALL_DIR"
    else
        mkdir -p "$INSTALL_DIR"
    fi

    # Construct download URL
    local download_url="$BASE_URL/$arch/$os/$BINARY_NAME"
    local temp_binary="$TEMP_DIR/$BINARY_NAME"
    local install_path="$INSTALL_DIR/$BINARY_NAME"

    # Check if already installed
    if [[ -f "$install_path" ]]; then
        log_warn "crit-cli is already installed at $install_path"
        read -p "Do you want to overwrite it? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Installation cancelled"
            exit 0
        fi
    fi

    # Download binary
    log_info "Downloading crit-cli..."
    if ! download_file "$download_url" "$temp_binary"; then
        log_error "Failed to download binary from $download_url"
        log_error "Please check if the URL is correct and the binary exists for your platform"
        exit 1
    fi

    # Verify download
    verify_download "$temp_binary"

    # Install binary
    install_binary "$temp_binary" "$install_path" "$is_root"

    # Add to PATH
    add_to_path "$INSTALL_DIR" "$is_root"

    # Test installation
    test_installation "$install_path"

    # Final message
    echo ""
    log_success "crit-cli has been successfully installed!"
    log_info "Installation location: $install_path"

    if [[ "$is_root" == "false" ]]; then
        log_info "You may need to restart your shell or run 'source ~/.bashrc' (or equivalent)"
    fi

    log_info "You can now use: $BINARY_NAME [options] ..."

    # Show version if possible
    if command_exists "$BINARY_NAME" || [[ "$PATH" == *"$INSTALL_DIR"* ]]; then
        echo ""
        log_info "Testing command:"
        "$install_path" --version 2>/dev/null || echo "Run '$BINARY_NAME --help' to see available options"
    fi
}

# Run main function
main "$@"
