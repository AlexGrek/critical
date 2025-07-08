# Makefile for crit-cli cross-compilation
# Builds binaries for all supported platforms and architectures

# Configuration
BINARY_NAME := crit
SOURCE_DIR := backend
BUILD_DIR := build/crit-cli
DOCKER_IMAGE := ghcr.io/cross-rs/cross:x86_64-unknown-linux-gnu

# Version and metadata
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo "dev")
BUILD_DATE := $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
GIT_COMMIT := $(shell git rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Build flags
CARGO_BUILD_FLAGS := --release --bin crit-cli
LDFLAGS := -X main.version=$(VERSION) -X main.buildDate=$(BUILD_DATE) -X main.gitCommit=$(GIT_COMMIT)

# Target definitions
# Format: RUST_TARGET:ARCH:OS
TARGETS := \
	x86_64-unknown-linux-gnu:amd64:linux \
	i686-unknown-linux-gnu:386:linux \
	aarch64-unknown-linux-gnu:arm64:linux \
	armv7-unknown-linux-gnueabihf:arm:linux \
	x86_64-apple-darwin:amd64:darwin \
	aarch64-apple-darwin:arm64:darwin \
	x86_64-pc-windows-gnu:amd64:windows \
	i686-pc-windows-gnu:386:windows \
	aarch64-pc-windows-msvc:arm64:windows

# Colors for output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
NC := \033[0m

# Default target
.DEFAULT_GOAL := help

# Help target
.PHONY: help
help: ## Show this help message
	@echo "$(BLUE)crit-cli Build System$(NC)"
	@echo ""
	@echo "$(YELLOW)Available targets:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)Architecture/OS combinations:$(NC)"
	@echo "  Linux:   amd64, 386, arm64, arm"
	@echo "  macOS:   amd64, arm64"
	@echo "  Windows: amd64, 386, arm64"

# Check prerequisites
.PHONY: check-deps
check-deps: ## Check build dependencies
	@echo "$(BLUE)[INFO]$(NC) Checking build dependencies..."
	@command -v cargo >/dev/null 2>&1 || { echo "$(RED)[ERROR]$(NC) cargo not found. Please install Rust."; exit 1; }
	@command -v docker >/dev/null 2>&1 || { echo "$(RED)[ERROR]$(NC) docker not found. Please install Docker."; exit 1; }
	@command -v git >/dev/null 2>&1 || { echo "$(YELLOW)[WARN]$(NC) git not found. Version info will be limited."; }
	@echo "$(GREEN)[SUCCESS]$(NC) All dependencies found"

# Install cross-compilation tools
.PHONY: install-cross
install-cross: ## Install cross-compilation tools
	@echo "$(BLUE)[INFO]$(NC) Installing cross-compilation tools..."
	@cargo install cross --git https://github.com/cross-rs/cross 2>/dev/null || true
	@echo "$(GREEN)[SUCCESS]$(NC) Cross-compilation tools ready"

# Setup rust targets
.PHONY: setup-targets
setup-targets: ## Add required Rust targets
	@echo "$(BLUE)[INFO]$(NC) Adding Rust targets..."
	@for target_info in $(TARGETS); do \
		rust_target=$$(echo $$target_info | cut -d: -f1); \
		echo "$(BLUE)[INFO]$(NC) Adding target: $$rust_target"; \
		rustup target add $$rust_target 2>/dev/null || true; \
	done
	@echo "$(GREEN)[SUCCESS]$(NC) All targets added"

# Clean build directory
.PHONY: clean
clean: ## Clean build directory
	@echo "$(BLUE)[INFO]$(NC) Cleaning build directory..."
	@rm -rf $(BUILD_DIR)
	@cd $(SOURCE_DIR) && cargo clean
	@echo "$(GREEN)[SUCCESS]$(NC) Build directory cleaned"

# Create build directories
.PHONY: create-dirs
create-dirs:
	@for target_info in $(TARGETS); do \
		arch=$$(echo $$target_info | cut -d: -f2); \
		os=$$(echo $$target_info | cut -d: -f3); \
		mkdir -p $(BUILD_DIR)/$$arch/$$os; \
	done

# Build for specific target
define build-target
.PHONY: build-$(2)-$(3)
build-$(2)-$(3): ## Build for $(2)/$(3)
	@echo "$(BLUE)[INFO]$(NC) Building for $(2)/$(3) ($(1))..."
	@mkdir -p $(BUILD_DIR)/$(2)/$(3)
	@cd $(SOURCE_DIR) && \
		if [ "$(1)" = "x86_64-apple-darwin" ] || [ "$(1)" = "aarch64-apple-darwin" ]; then \
			if [ "$$(uname)" = "Darwin" ]; then \
				echo "$(YELLOW)[INFO]$(NC) Building natively on macOS..."; \
				cargo build $(CARGO_BUILD_FLAGS) --target $(1); \
			else \
				echo "$(YELLOW)[INFO]$(NC) Cross-compiling for macOS using cross..."; \
				cross build $(CARGO_BUILD_FLAGS) --target $(1); \
			fi; \
		elif [ "$(3)" = "windows" ]; then \
			echo "$(YELLOW)[INFO]$(NC) Cross-compiling for Windows..."; \
			cross build $(CARGO_BUILD_FLAGS) --target $(1); \
		else \
			echo "$(YELLOW)[INFO]$(NC) Cross-compiling for $(1)..."; \
			cross build $(CARGO_BUILD_FLAGS) --target $(1); \
		fi
	@cd $(SOURCE_DIR) && \
		if [ "$(3)" = "windows" ]; then \
			cp target/$(1)/release/crit-cli.exe ../$(BUILD_DIR)/$(2)/$(3)/$(BINARY_NAME).exe; \
		else \
			cp target/$(1)/release/crit-cli ../$(BUILD_DIR)/$(2)/$(3)/$(BINARY_NAME); \
		fi
	@echo "$(GREEN)[SUCCESS]$(NC) Built $(2)/$(3) -> $(BUILD_DIR)/$(2)/$(3)/$(BINARY_NAME)"
endef

# Generate build targets
$(foreach target_info,$(TARGETS),$(eval $(call build-target,$(word 1,$(subst :, ,$(target_info))),$(word 2,$(subst :, ,$(target_info))),$(word 3,$(subst :, ,$(target_info))))))

# Build all Linux targets
.PHONY: build-linux
build-linux: build-amd64-linux build-386-linux build-arm64-linux build-arm-linux ## Build all Linux targets

# Build all macOS targets
.PHONY: build-darwin
build-darwin: build-amd64-darwin build-arm64-darwin ## Build all macOS targets

# Build all Windows targets
.PHONY: build-windows
build-windows: build-amd64-windows build-386-windows build-arm64-windows ## Build all Windows targets

# Build all targets
.PHONY: build-all
build-all: check-deps install-cross setup-targets create-dirs build-linux build-darwin build-windows ## Build all targets
	@echo ""
	@echo "$(GREEN)[SUCCESS]$(NC) All targets built successfully!"
	@echo ""
	@echo "$(YELLOW)Build summary:$(NC)"
	@find $(BUILD_DIR) -type f -name "$(BINARY_NAME)*" | sort | while read file; do \
		size=$$(du -h "$$file" | cut -f1); \
		echo "  $$file ($$size)"; \
	done

# Build only Unix targets (Linux + macOS)
.PHONY: build-unix
build-unix: build-linux build-darwin ## Build Unix targets (Linux + macOS)

# Test build on current platform
.PHONY: test-build
test-build: ## Test build on current platform
	@echo "$(BLUE)[INFO]$(NC) Testing build on current platform..."
	@cd $(SOURCE_DIR) && cargo build $(CARGO_BUILD_FLAGS)
	@echo "$(GREEN)[SUCCESS]$(NC) Test build completed"

# Verify all binaries
.PHONY: verify
verify: ## Verify all built binaries
	@echo "$(BLUE)[INFO]$(NC) Verifying built binaries..."
	@for target_info in $(TARGETS); do \
		arch=$$(echo $$target_info | cut -d: -f2); \
		os=$$(echo $$target_info | cut -d: -f3); \
		if [ "$$os" = "windows" ]; then \
			binary_path="$(BUILD_DIR)/$$arch/$$os/$(BINARY_NAME).exe"; \
		else \
			binary_path="$(BUILD_DIR)/$$arch/$$os/$(BINARY_NAME)"; \
		fi; \
		if [ -f "$$binary_path" ]; then \
			size=$$(du -h "$$binary_path" | cut -f1); \
			echo "$(GREEN)[OK]$(NC) $$arch/$$os ($$size)"; \
		else \
			echo "$(RED)[MISSING]$(NC) $$arch/$$os"; \
		fi; \
	done

# Create release archive
.PHONY: archive
archive: ## Create release archives
	@echo "$(BLUE)[INFO]$(NC) Creating release archives..."
	@mkdir -p $(BUILD_DIR)/releases
	@for target_info in $(TARGETS); do \
		arch=$$(echo $$target_info | cut -d: -f2); \
		os=$$(echo $$target_info | cut -d: -f3); \
		if [ "$$os" = "windows" ]; then \
			binary_path="$(BUILD_DIR)/$$arch/$$os/$(BINARY_NAME).exe"; \
			archive_name="$(BINARY_NAME)-$(VERSION)-$$arch-$$os.zip"; \
			if [ -f "$$binary_path" ]; then \
				cd $(BUILD_DIR)/$$arch/$$os && zip -q "../../../releases/$$archive_name" "$(BINARY_NAME).exe"; \
			fi; \
		else \
			binary_path="$(BUILD_DIR)/$$arch/$$os/$(BINARY_NAME)"; \
			archive_name="$(BINARY_NAME)-$(VERSION)-$$arch-$$os.tar.gz"; \
			if [ -f "$$binary_path" ]; then \
				cd $(BUILD_DIR)/$$arch/$$os && tar -czf "../../../releases/$$archive_name" "$(BINARY_NAME)"; \
			fi; \
		fi; \
	done
	@echo "$(GREEN)[SUCCESS]$(NC) Archives created in $(BUILD_DIR)/releases/"

# Quick build for development (current platform only)
.PHONY: dev
dev: ## Quick development build
	@echo "$(BLUE)[INFO]$(NC) Building for development..."
	@cd $(SOURCE_DIR) && cargo build --bin crit-cli
	@echo "$(GREEN)[SUCCESS]$(NC) Development build completed"

# Release build (all platforms with archives)
.PHONY: release
release: clean build-all verify archive ## Full release build with archives
	@echo ""
	@echo "$(GREEN)[SUCCESS]$(NC) Release build completed!"
	@echo "$(YELLOW)Release info:$(NC)"
	@echo "  Version: $(VERSION)"
	@echo "  Build Date: $(BUILD_DATE)"
	@echo "  Git Commit: $(GIT_COMMIT)"
	@echo "  Binaries: $(BUILD_DIR)/"
	@echo "  Archives: $(BUILD_DIR)/releases/"

# Docker-based build (for CI/CD)
.PHONY: docker-build
docker-build: ## Build using Docker (for CI/CD)
	@echo "$(BLUE)[INFO]$(NC) Building in Docker container..."
	@docker run --rm \
		-v $(PWD):/workspace \
		-w /workspace \
		$(DOCKER_IMAGE) \
		make build-all
	@echo "$(GREEN)[SUCCESS]$(NC) Docker build completed"

# Show build info
.PHONY: info
info: ## Show build information
	@echo "$(BLUE)Build Information:$(NC)"
	@echo "  Version: $(VERSION)"
	@echo "  Build Date: $(BUILD_DATE)"
	@echo "  Git Commit: $(GIT_COMMIT)"
	@echo "  Source Directory: $(SOURCE_DIR)"
	@echo "  Build Directory: $(BUILD_DIR)"
	@echo "  Binary Name: $(BINARY_NAME)"
	@echo ""
	@echo "$(BLUE)Available Targets:$(NC)"
	@for target_info in $(TARGETS); do \
		rust_target=$$(echo $$target_info | cut -d: -f1); \
		arch=$$(echo $$target_info | cut -d: -f2); \
		os=$$(echo $$target_info | cut -d: -f3); \
		echo "  $$arch/$$os ($$rust_target)"; \
	done
