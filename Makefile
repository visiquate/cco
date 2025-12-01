.PHONY: build release test check fmt clippy install clean help dev debug sbom audit

# Default target
.DEFAULT_GOAL := help

# Build variables
CARGO_FLAGS ?=
RELEASE_FLAGS ?= --release
INSTALL_DIR ?= /usr/local/bin

# Detect OS for platform-specific commands
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    STRIP_CMD := strip
else
    STRIP_CMD := strip
endif

help:
	@echo "Claude Orchestra Build System (CCO)"
	@echo ""
	@echo "Available targets:"
	@echo "  build       - Build debug binary"
	@echo "  dev         - Build debug with verbose output"
	@echo "  release     - Build optimized release binary"
	@echo "  test        - Run all tests"
	@echo "  check       - Check code without building"
	@echo "  clippy      - Run Clippy linter"
	@echo "  fmt         - Format code with rustfmt"
	@echo "  fmt-check   - Check formatting without changes"
	@echo "  install     - Build and install to /usr/local/bin"
	@echo "  uninstall   - Remove installed binary"
	@echo "  clean       - Clean build artifacts"
	@echo "  doc         - Generate documentation"
	@echo "  sbom        - Generate Software Bill of Materials"
	@echo "  audit       - Run security audit on dependencies"
	@echo ""
	@echo "Variables:"
	@echo "  INSTALL_DIR - Installation directory (default: /usr/local/bin)"
	@echo "  CARGO_FLAGS - Additional Cargo flags"

# Build debug binary
build:
	@echo "Building CCO (debug)..."
	@cargo build $(CARGO_FLAGS)
	@echo "Build complete: target/debug/cco"

# Build with verbose output
dev:
	@echo "Building CCO (debug, verbose)..."
	@cargo build --verbose $(CARGO_FLAGS)
	@echo "Build complete: target/debug/cco"

# Build debug with additional info
debug: dev

# Build release binary with optimizations
release:
	@echo "Building CCO (release)..."
	@cargo build $(RELEASE_FLAGS) $(CARGO_FLAGS)
	@echo "Stripping symbols..."
	@$(STRIP_CMD) target/release/cco || true
	@echo "Release build complete: target/release/cco"
	@ls -lh target/release/cco

# Run tests
test:
	@echo "Running tests..."
	@cargo test --lib $(CARGO_FLAGS)
	@cargo test --test '*' $(CARGO_FLAGS)
	@echo "All tests passed!"

# Run tests with output
test-verbose:
	@echo "Running tests (verbose)..."
	@cargo test --lib -- --nocapture $(CARGO_FLAGS)
	@cargo test --test '*' -- --nocapture $(CARGO_FLAGS)

# Run integration tests only
test-integration:
	@echo "Running integration tests..."
	@cargo test --test '*' $(CARGO_FLAGS)

# Check code without building
check:
	@echo "Checking code..."
	@cargo check $(CARGO_FLAGS)
	@echo "Check complete!"

# Run Clippy linter
clippy:
	@echo "Running Clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings
	@echo "Clippy check complete!"

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt --all
	@echo "Code formatted!"

# Check formatting
fmt-check:
	@echo "Checking code formatting..."
	@cargo fmt --all -- --check
	@echo "Format check complete!"

# Install binary to system
install: release
	@echo "Installing CCO to $(INSTALL_DIR)..."
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/cco $(INSTALL_DIR)/
	@chmod +x $(INSTALL_DIR)/cco
	@echo "Installation complete!"
	@echo "CCO installed at: $(INSTALL_DIR)/cco"
	@$(INSTALL_DIR)/cco --version 2>/dev/null || echo "Verify with: cco --version"

# Uninstall binary from system
uninstall:
	@echo "Uninstalling CCO from $(INSTALL_DIR)..."
	@rm -f $(INSTALL_DIR)/cco
	@echo "Uninstall complete!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -f target/release/cco
	@echo "Clean complete!"

# Generate documentation
doc:
	@echo "Generating documentation..."
	@cargo doc --no-deps --open
	@echo "Documentation generated!"

# Run all checks (format, clippy, tests)
check-all: fmt-check clippy test
	@echo "All checks passed!"

# Development workflow: format, lint, test, build
dev-check: fmt clippy test build
	@echo "Development workflow complete!"

# Continuous integration target
ci: check fmt-check clippy test
	@echo "CI checks passed!"

# Generate Software Bill of Materials (SBOM)
sbom:
	@echo "Generating SBOM..."
	@mkdir -p artifacts
	@cargo sbom --output-format cyclone_dx_json_1_5 > artifacts/sbom-cyclonedx.json
	@cargo sbom --output-format spdx_json_2_3 > artifacts/sbom-spdx.json
	@echo "SBOM files generated:"
	@echo "  - artifacts/sbom-cyclonedx.json (CycloneDX 1.5)"
	@echo "  - artifacts/sbom-spdx.json (SPDX 2.3)"
	@echo "Components: $$(jq '.components | length' artifacts/sbom-cyclonedx.json)"

# Run security audit on dependencies
audit:
	@echo "Running security audit..."
	@cargo audit || echo "Audit completed with warnings"
	@echo "Audit complete!"
