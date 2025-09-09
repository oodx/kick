#!/bin/bash
set -e

# Kick API Client Deploy Script - Rust binary with lib-to-bin pattern
# Deploys kick binary to ~/.local/lib/odx/kick/ and creates bin symlink

# Configuration
LIB_DIR="$HOME/.local/lib/odx/kick"
BIN_DIR="$HOME/.local/bin/odx"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
BINARY_NAME="kick"
DEPLOYABLE="${BINARY_NAME}"

# Extract version from Cargo.toml at repo root
VERSION=$(grep '^version' "$ROOT_DIR/Cargo.toml" | head -1 | cut -d'"' -f2)

# Check boxy availability
has_boxy() {
    command -v boxy >/dev/null 2>&1
}

# Ceremonial messaging
ceremony_msg() {
    local msg="$1"
    local theme="${2:-info}"
    if has_boxy; then
        echo "$msg" | boxy --theme "$theme" --width max
    else
        echo "$msg"
    fi
}

step_msg() {
    local step="$1"
    local desc="$2"
    if has_boxy; then
        printf "%s\n%s\n" "$step" "• $desc" | boxy --style rounded --width max --title "📦 Deploy Step"
    else
        echo "$step: $desc"
    fi
}

# Welcome ceremony
ceremony_msg "🚀 KICK API CLIENT DEPLOYMENT CEREMONY v$VERSION" "success"
echo

step_msg "Step 1" "Building kick v$VERSION with plugin architecture..."
cd "$ROOT_DIR"
if ! cargo build --release; then
    ceremony_msg "❌ Build failed!" "error"
    exit 1
fi

# Clean up RSB's malformed XDG directory bug
if [ -d "${ROOT_DIR}/\${XDG_TMP:-" ]; then
    rm -rf "${ROOT_DIR}/\${XDG_TMP:-"
fi

# Check if binary was created
if [ ! -f "$ROOT_DIR/target/release/${DEPLOYABLE}" ]; then
    ceremony_msg "❌ Binary not found at target/release/${DEPLOYABLE}" "error"
    exit 1
fi

step_msg "Step 2" "Creating lib directory: $LIB_DIR"
mkdir -p "$LIB_DIR"

step_msg "Step 3" "Deploying binary to lib directory..."
if ! cp "$ROOT_DIR/target/release/${DEPLOYABLE}" "$LIB_DIR/$BINARY_NAME"; then
    ceremony_msg "❌ Failed to copy binary to $LIB_DIR" "error"
    exit 1
fi

if ! chmod +x "$LIB_DIR/$BINARY_NAME"; then
    ceremony_msg "❌ Failed to make binary executable" "error"
    exit 1
fi

step_msg "Step 4" "Creating bin directory: $BIN_DIR"
mkdir -p "$BIN_DIR"

step_msg "Step 5" "Creating bin symlink: $BIN_DIR/$BINARY_NAME → $LIB_DIR/$BINARY_NAME"
if [[ -L "$BIN_DIR/$BINARY_NAME" ]] || [[ -f "$BIN_DIR/$BINARY_NAME" ]]; then
    rm "$BIN_DIR/$BINARY_NAME"
fi

if ! ln -s "$LIB_DIR/$BINARY_NAME" "$BIN_DIR/$BINARY_NAME"; then
    ceremony_msg "❌ Failed to create symlink" "error"
    exit 1
fi

step_msg "Step 6" "Verifying deployment..."
if [[ ! -x "$LIB_DIR/$BINARY_NAME" ]]; then
    ceremony_msg "❌ Binary is not executable at $LIB_DIR/$BINARY_NAME" "error"
    exit 1
fi

if [[ ! -L "$BIN_DIR/$BINARY_NAME" ]]; then
    ceremony_msg "❌ Symlink not created at $BIN_DIR/$BINARY_NAME" "error"
    exit 1
fi

step_msg "Step 7" "Testing kick command..."
if ! "$BIN_DIR/$BINARY_NAME" --help >/dev/null 2>&1; then
    ceremony_msg "❌ Kick command test failed!" "error"
    exit 1
fi

# Success ceremony
ceremony_msg "✅ KICK API CLIENT v$VERSION DEPLOYED SUCCESSFULLY!" "success"
echo

if has_boxy; then
    {
        echo "🌐 Lightweight HTTP API client with plugin support"
        echo "📍 Library: $LIB_DIR/$BINARY_NAME"
        echo "📍 Binary: $BIN_DIR/$BINARY_NAME"
        echo
        echo "💡 Usage Examples:"
        echo "   kick get https://api.ipify.org/?format=json --pretty"
        echo "   kick post https://httpbin.org/post --data '{\"key\":\"value\"}'"
        echo "   kick download https://dog.ceo/api/breeds/image/random --output dog.json"
        echo "   kick get https://api.github.com/user -H \"Auth:Bearer TOKEN\" --verbose"
        echo
        echo "🎭 Features:"
        echo "   • HTTP GET, POST, and file download operations" 
        echo "   • Custom headers and user-agent configuration"
        echo "   • Plugin architecture with logging support"
        echo "   • Pretty JSON formatting and file saving"
        echo "   • Built on hyper 1.0 with async/await patterns"
        echo "   • XDG-compliant configuration and storage"
    } | boxy --theme success --header "🚀 Kick API Client v$VERSION Deployed" \
             --status "sr:$(date '+%H:%M:%S')" \
             --footer "✅ Ready for API testing" \
             --width max
else
    echo "📍 Library location: $LIB_DIR/$BINARY_NAME"
    echo "📍 Binary symlink: $BIN_DIR/$BINARY_NAME"
    echo
    echo "💡 Usage Examples:"
    echo "   kick get https://api.ipify.org/?format=json --pretty"
    echo "   kick post https://httpbin.org/post --data '{\"key\":\"value\"}'"
    echo "   kick download https://dog.ceo/api/breeds/image/random --output dog.json"
    echo "   kick get https://api.github.com/user -H \"Auth:Bearer TOKEN\" --verbose"
fi

echo
step_msg "🧪 Quick Test" "Running comprehensive functionality test"

echo "Testing GET request..."
if RESULT=$("$BIN_DIR/$BINARY_NAME" get "https://api.ipify.org/?format=json") && [[ "$RESULT" =~ "ip" ]]; then
    echo "✅ GET request successful: $(echo "$RESULT" | head -c 50)..."
else
    ceremony_msg "❌ GET request failed!" "error"
    exit 1
fi

echo "Testing pretty JSON formatting..."
if "$BIN_DIR/$BINARY_NAME" get "https://api.ipify.org/?format=json" --pretty | grep -q "ip"; then
    echo "✅ Pretty JSON formatting functional"
else
    ceremony_msg "❌ Pretty JSON formatting failed!" "error"
    exit 1
fi

echo "Testing custom headers..."
if RESULT=$("$BIN_DIR/$BINARY_NAME" get "https://httpbin.org/headers" -H "X-Deploy-Test:kick-v$VERSION") && 
   [[ "$RESULT" =~ "X-Deploy-Test" ]]; then
    echo "✅ Custom headers functional"
else
    ceremony_msg "❌ Custom headers failed!" "error"
    exit 1
fi

echo "Testing user agent configuration..."
if RESULT=$("$BIN_DIR/$BINARY_NAME" get "https://httpbin.org/user-agent" -A "KickDeployTest/1.0") && 
   [[ "$RESULT" =~ "KickDeployTest" ]]; then
    echo "✅ User agent configuration functional"
else
    ceremony_msg "❌ User agent configuration failed!" "error"
    exit 1
fi

echo "Testing POST request..."
if RESULT=$("$BIN_DIR/$BINARY_NAME" post "https://httpbin.org/post" --data '{"deploy":"test","version":"'$VERSION'"}') && 
   [[ "$RESULT" =~ "deploy" ]]; then
    echo "✅ POST request functional"
else
    ceremony_msg "❌ POST request failed!" "error"
    exit 1
fi

echo "Testing file download..."
TEMP_FILE="/tmp/kick_deploy_test.json"
if "$BIN_DIR/$BINARY_NAME" download "https://api.ipify.org/?format=json" --output "$TEMP_FILE" &&
   [[ -f "$TEMP_FILE" ]] && grep -q "ip" "$TEMP_FILE"; then
    echo "✅ File download functional"
    rm -f "$TEMP_FILE" 2>/dev/null
else
    ceremony_msg "❌ File download failed!" "error"
    exit 1
fi

echo "Testing verbose mode with plugin logging..."
if "$BIN_DIR/$BINARY_NAME" get "https://api.ipify.org/?format=json" --verbose 2>&1 | grep -q "PLUGIN-LOG"; then
    echo "✅ Verbose mode and plugin logging functional"
else
    ceremony_msg "❌ Verbose mode/plugin logging failed!" "error"
    exit 1
fi

echo "Testing help system..."
if "$BIN_DIR/$BINARY_NAME" get --help | grep -q "Make a GET request" &&
   "$BIN_DIR/$BINARY_NAME" post --help | grep -q "Make a POST request" &&
   "$BIN_DIR/$BINARY_NAME" download --help | grep -q "Download file"; then
    echo "✅ Help system functional"
else
    ceremony_msg "❌ Help system failed!" "error"
    exit 1
fi

# Final ceremony
ceremony_msg "🎉 KICK API CLIENT v$VERSION READY FOR USE!" "success"

if has_boxy; then
    {
        echo "Run the comprehensive test suite:"
        echo "   cd $ROOT_DIR && ./bin/test.sh"
        echo
        echo "Test immediately with UAT wrapper:"
        echo "   cd $ROOT_DIR && ./bin/uat.sh https://api.ipify.org/?format=json --pretty"
        echo
        echo "Or use the binary directly:"
        echo "   $BIN_DIR/$BINARY_NAME get https://dog.ceo/api/breeds/image/random --verbose"
        echo "   $BIN_DIR/$BINARY_NAME post https://httpbin.org/post --data '{\"test\":\"deployed\"}'"
    } | boxy --theme info --header "🚀 Next Steps"
fi
