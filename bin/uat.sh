#!/bin/bash
# UAT (User Acceptance Testing) CLI for Kick API Client
# Simple wrapper around the kick release binary

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get script directory and find kick binary
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KICK_BIN="$SCRIPT_DIR/../target/release/kick"

# Check if binary exists, build if needed
if [ ! -f "$KICK_BIN" ]; then
    echo -e "${YELLOW}📦 Building kick binary...${NC}"
    cd "$SCRIPT_DIR/.."
    cargo build --release --bin kick
    cd - > /dev/null
fi

show_help() {
    echo -e "${BLUE}🎯 Kick API Client - UAT Wrapper${NC}"
    echo "================================="
    echo ""
    echo "This is a simple wrapper around the kick binary for easy testing."
    echo ""
    echo -e "${YELLOW}QUICK EXAMPLES:${NC}"
    echo "  $0 https://api.ipify.org/?format=json"
    echo "  $0 https://dog.ceo/api/breeds/image/random --pretty --save dog.json"
    echo "  $0 https://httpbin.org/headers -H \"X-Test:value\" --verbose"
    echo ""
    echo -e "${YELLOW}ADVANCED USAGE:${NC}"
    echo "  # POST request"
    echo "  $0 post https://httpbin.org/post --data '{\"key\":\"value\"}' --pretty"
    echo ""
    echo "  # Download file"  
    echo "  $0 download https://api.ipify.org/?format=json --output myip.json"
    echo ""
    echo -e "${CYAN}For full options: $KICK_BIN --help${NC}"
    echo ""
}

# Handle help
if [[ $# -eq 0 || "$1" == "--help" || "$1" == "-h" ]]; then
    show_help
    exit 0
fi

# Check if first arg looks like a subcommand
if [[ "$1" =~ ^(get|post|download)$ ]]; then
    # Direct pass-through to kick binary
    echo -e "${BLUE}🚀 Running: kick $*${NC}"
    exec "$KICK_BIN" "$@"
elif [[ "$1" =~ ^https?:// ]]; then
    # URL provided - assume GET request
    echo -e "${BLUE}🚀 Running: kick get $*${NC}"
    exec "$KICK_BIN" get "$@"  
else
    echo -e "${RED}❌ Invalid arguments. Expected URL or subcommand.${NC}"
    echo ""
    show_help
    exit 1
fi