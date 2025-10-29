#!/bin/bash
# Build release binaries for multiple platforms
# This script helps create pre-built binaries for distribution

set -e

echo "🔨 Building AgentSync Release Binaries"
echo "======================================"
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Version from Cargo.toml
VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${BLUE}Version: ${VERSION}${NC}"
echo ""

# Create releases directory
RELEASE_DIR="releases/v${VERSION}"
mkdir -p "$RELEASE_DIR"
echo -e "${GREEN}✓ Created release directory: ${RELEASE_DIR}${NC}"
echo ""

# Detect current platform
CURRENT_OS=$(uname -s)
CURRENT_ARCH=$(uname -m)

echo -e "${BLUE}Current platform: ${CURRENT_OS} ${CURRENT_ARCH}${NC}"
echo ""

# Build for current platform
echo -e "${BLUE}Building for current platform...${NC}"
cargo build --release

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✓ Build successful${NC}"

    # Determine target based on current platform
    if [ "$CURRENT_OS" = "Darwin" ]; then
        if [ "$CURRENT_ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        else
            TARGET="x86_64-apple-darwin"
        fi
    elif [ "$CURRENT_OS" = "Linux" ]; then
        if [ "$CURRENT_ARCH" = "aarch64" ]; then
            TARGET="aarch64-unknown-linux-gnu"
        else
            TARGET="x86_64-unknown-linux-gnu"
        fi
    else
        echo -e "${RED}✗ Unsupported platform${NC}"
        exit 1
    fi

    # Package binary
    TARBALL="agentsync-v${VERSION}-${TARGET}.tar.gz"
    echo -e "${BLUE}Creating tarball: ${TARBALL}${NC}"

    tar -czf "${RELEASE_DIR}/${TARBALL}" \
        -C target/release agentsync \
        -C ../../ README.md LICENSE

    echo -e "${GREEN}✓ Created ${TARBALL}${NC}"
    echo ""
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi

# Generate checksum
echo -e "${BLUE}Generating checksums...${NC}"
cd "$RELEASE_DIR"
shasum -a 256 *.tar.gz > SHA256SUMS
cd ../..
echo -e "${GREEN}✓ Generated SHA256SUMS${NC}"
echo ""

# Display results
echo "======================================"
echo -e "${GREEN}✓ Release build complete!${NC}"
echo ""
echo "Files created in ${RELEASE_DIR}:"
ls -lh "$RELEASE_DIR"
echo ""
echo "Checksums:"
cat "${RELEASE_DIR}/SHA256SUMS"
echo ""

# Instructions
echo "======================================"
echo "Next steps:"
echo ""
echo "1. Test the binary:"
echo "   tar -xzf ${RELEASE_DIR}/${TARBALL}"
echo "   ./agentsync --version"
echo ""
echo "2. Create GitHub Release:"
echo "   - Go to: https://github.com/yourusername/agentsync/releases/new"
echo "   - Tag: v${VERSION}"
echo "   - Upload files from: ${RELEASE_DIR}/"
echo ""
echo "3. Update Homebrew formula with new SHA256"
echo ""
echo "For cross-platform builds, use GitHub Actions or:"
echo "   rustup target add <target>"
echo "   cargo build --release --target <target>"
echo ""
echo "Available targets:"
echo "   - aarch64-apple-darwin (macOS Apple Silicon)"
echo "   - x86_64-apple-darwin (macOS Intel)"
echo "   - x86_64-unknown-linux-gnu (Linux x86_64)"
echo "   - aarch64-unknown-linux-gnu (Linux ARM64)"
echo ""

