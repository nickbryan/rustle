#!/bin/bash
# This script downloads the necessary JS/CSS dependencies for the project.
# It places them in the 'assets/' directory, ready for Trunk to bundle.
# Exit immediately if a command exits with a non-zero status.
set -e

XTERM_VERSION="5.5.0"
FIT_ADDON_VERSION="0.10.0"
DEST_DIR="assets"

XTERM_CSS_URL="https://cdn.jsdelivr.net/npm/@xterm/xterm@${XTERM_VERSION}/css/xterm.min.css"
XTERM_JS_URL="https://cdn.jsdelivr.net/npm/@xterm/xterm@${XTERM_VERSION}/lib/xterm.min.js"
FIT_ADDON_JS_URL="https://cdn.jsdelivr.net/npm/@xterm/addon-fit@${FIT_ADDON_VERSION}/lib/addon-fit.min.js"

echo "Setting up dependencies in the './${DEST_DIR}/' directory..."

mkdir -p "$DEST_DIR"
echo "Directory '${DEST_DIR}' is ready."

echo "Downloading xterm.min.css (v${XTERM_VERSION})..."
curl -L --fail -o "${DEST_DIR}/xterm.css" "$XTERM_CSS_URL"

echo "Downloading xterm.min.js (v${XTERM_VERSION})..."
curl -L --fail -o "${DEST_DIR}/xterm.js" "$XTERM_JS_URL"

echo "Downloading xterm-addon-fit.min.js (v${FIT_ADDON_VERSION})..."
curl -L --fail -o "${DEST_DIR}/xterm-addon-fit.js" "$FIT_ADDON_JS_URL"

echo ""
echo "✅ All dependencies downloaded successfully!"
