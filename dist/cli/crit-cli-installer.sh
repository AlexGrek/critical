#!/bin/bash

# cr1t CLI installer
# Usage: curl -fsSL <raw-url>/crit-cli-installer.sh | bash

set -euo pipefail

BINARY_NAME="cr1t"
GITHUB_REPO="AlexGrek/critical"
INSTALL_DIR=""
TEMP_DIR=$(mktemp -d)

trap 'rm -rf "$TEMP_DIR"' EXIT

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; }

detect_os() {
    case "$OSTYPE" in
        linux-gnu*)  echo "linux" ;;
        darwin*)     echo "darwin" ;;
        *)           error "Unsupported OS: $OSTYPE"; exit 1 ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)       echo "amd64" ;;
        aarch64|arm64)      echo "arm64" ;;
        *)                  error "Unsupported architecture: $(uname -m)"; exit 1 ;;
    esac
}

get_install_dir() {
    if [[ $EUID -eq 0 ]]; then
        echo "/usr/local/bin"
    else
        local dir="$HOME/.local/bin"
        mkdir -p "$dir"
        echo "$dir"
    fi
}

ensure_in_path() {
    local dir="$1"

    echo "$PATH" | grep -q "$dir" && return 0
    [[ $EUID -eq 0 ]] && return 0

    local profile=""
    case "$(basename "$SHELL")" in
        bash) profile="${HOME}/.bashrc" ;;
        zsh)  profile="${HOME}/.zshrc" ;;
        fish)
            local fc="$HOME/.config/fish/config.fish"
            [[ -f "$fc" ]] && ! grep -q "$dir" "$fc" && echo "set -gx PATH $dir \$PATH" >> "$fc"
            return 0 ;;
        *)    profile="${HOME}/.profile" ;;
    esac

    if [[ -n "$profile" ]] && ! grep -q "$dir" "$profile" 2>/dev/null; then
        echo "" >> "$profile"
        echo "# Added by cr1t installer" >> "$profile"
        echo "export PATH=\"$dir:\$PATH\"" >> "$profile"
        warn "Added $dir to PATH in $profile — restart your shell or: source $profile"
    fi
}

download() {
    local url="$1" output="$2"
    info "Downloading from: $url"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --connect-timeout 10 --max-time 300 "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q --timeout=10 --tries=3 "$url" -O "$output"
    else
        error "Neither curl nor wget found."; exit 1
    fi
}

main() {
    info "Installing cr1t CLI..."

    local os=$(detect_os)
    local arch=$(detect_arch)
    local artifact_name="cr1t-${os}-${arch}"

    info "Platform: ${os}/${arch}"

    INSTALL_DIR=$(get_install_dir)
    info "Install directory: $INSTALL_DIR"

    local install_path="$INSTALL_DIR/$BINARY_NAME"

    if [[ -f "$install_path" ]]; then
        warn "cr1t already installed at $install_path"
        read -p "Overwrite? (y/N): " -n 1 -r; echo
        [[ ! $REPLY =~ ^[Yy]$ ]] && { info "Cancelled."; exit 0; }
    fi

    # Try GitHub releases first
    local release_url="https://github.com/${GITHUB_REPO}/releases/latest/download/${artifact_name}.tar.gz"
    local temp_archive="$TEMP_DIR/${artifact_name}.tar.gz"
    local temp_binary="$TEMP_DIR/$BINARY_NAME"

    if download "$release_url" "$temp_archive" 2>/dev/null; then
        info "Downloaded from GitHub release"
        tar -xzf "$temp_archive" -C "$TEMP_DIR"
    else
        error "No release found at: $release_url"
        error ""
        error "To install from a CI artifact instead:"
        error "  1. Go to https://github.com/${GITHUB_REPO}/actions/workflows/cli.yml"
        error "  2. Download the '${artifact_name}' artifact from the latest run"
        error "  3. Unzip and copy 'cr1t' to a directory in your PATH"
        exit 1
    fi

    # Verify it's a binary
    if ! file "$temp_binary" | grep -q "executable\|ELF\|Mach-O"; then
        error "Downloaded file is not a valid binary"
        exit 1
    fi

    cp "$temp_binary" "$install_path"
    chmod 755 "$install_path"
    success "Installed to $install_path"

    ensure_in_path "$INSTALL_DIR"

    # Test
    if "$install_path" --version >/dev/null 2>&1; then
        echo ""
        success "$("$install_path" --version 2>&1)"
    fi

    echo ""
    success "cr1t is ready! Run: cr1t --help"
}

main "$@"
