#!/bin/bash

# Drovity installer script
# Detects OS and architecture, downloads appropriate binary

set -e

REPO="MixasV/drovity"
INSTALL_DIR="/usr/local/bin"
BINARY_NAME="drovity"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "========================================"
echo "      Drovity Installer v1.0"
echo "  Gemini API Proxy for Factory Droid"
echo "========================================"
echo ""

# Detect OS
OS="$(uname -s)"
case "$OS" in
    Linux*)     OS_TYPE=linux;;
    Darwin*)    OS_TYPE=macos;;
    *)          
        echo -e "${RED}[ERROR] Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

# Detect architecture
ARCH="$(uname -m)"
case "$ARCH" in
    x86_64)     ARCH_TYPE=x64;;
    arm64|aarch64) ARCH_TYPE=arm64;;
    *)          
        echo -e "${RED}[ERROR] Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

echo -e "${GREEN}Detected:${NC} $OS_TYPE-$ARCH_TYPE"

# Get latest release
echo "Fetching latest release..."
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    echo -e "${RED}[ERROR] Failed to fetch latest release${NC}"
    exit 1
fi

echo -e "${GREEN}Latest version:${NC} $LATEST_RELEASE"

# Construct download URL
if [ "$OS_TYPE" = "linux" ]; then
    # Use static musl build (works on all distros)
    BINARY_FILE="drovity-linux-$ARCH_TYPE"
    echo -e "${GREEN}Using static musl build (works on all Linux distributions)${NC}"
elif [ "$OS_TYPE" = "macos" ]; then
    BINARY_FILE="drovity-macos-$ARCH_TYPE"
fi

DOWNLOAD_URL="https://github.com/$REPO/releases/download/$LATEST_RELEASE/$BINARY_FILE"

echo "Downloading $BINARY_FILE..."
TMP_FILE="/tmp/$BINARY_FILE"

if ! curl -L -o "$TMP_FILE" "$DOWNLOAD_URL"; then
    echo -e "${RED}[ERROR] Download failed${NC}"
    exit 1
fi

# Make executable
chmod +x "$TMP_FILE"

# Install
echo "Installing to $INSTALL_DIR/$BINARY_NAME..."

if [ -w "$INSTALL_DIR" ]; then
    mv "$TMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
else
    echo -e "${YELLOW}Root permissions required for installation${NC}"
    sudo mv "$TMP_FILE" "$INSTALL_DIR/$BINARY_NAME"
fi

# Verify
if command -v $BINARY_NAME &> /dev/null; then
    echo ""
    echo -e "${GREEN}[SUCCESS] Drovity installed successfully!${NC}"
    echo ""
    echo "Get started:"
    echo "  $ drovity"
    echo ""
    echo "Documentation:"
    echo "  https://github.com/$REPO"
    echo ""
    echo "Contact: @onexv on Telegram (https://t.me/onexv)"
else
    echo -e "${RED}[ERROR] Installation failed${NC}"
    exit 1
fi
