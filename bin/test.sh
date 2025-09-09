#!/bin/bash
# Test runner entry point for Kick API Client

set -e

echo "🚀 Kick API Client Test Runner"
echo "================================"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test functions
run_unit_tests() {
    echo -e "${BLUE}📋 Running unit tests...${NC}"
    cargo test
}

run_network_tests() {
    echo -e "${BLUE}🌐 Running network tests...${NC}"
    cargo test -- --ignored
}

run_endpoint_tests() {
    echo -e "${BLUE}📡 Running endpoint validation tests...${NC}"
    cargo run --bin test_all_apis
}

run_builder_tests() {
    echo -e "${BLUE}🏗️ Running builder pattern tests...${NC}"
    cargo run --bin test_endpoints
}

# Parse command line arguments
case "${1:-all}" in
    "unit")
        run_unit_tests
        ;;
    "network")
        run_network_tests
        ;;
    "endpoints")
        run_endpoint_tests
        ;;
    "builder")
        run_builder_tests
        ;;
    "quick")
        echo -e "${YELLOW}⚡ Quick test mode${NC}"
        run_unit_tests
        run_endpoint_tests
        ;;
    "all")
        echo -e "${YELLOW}🎯 Full test suite${NC}"
        run_unit_tests
        run_network_tests
        run_endpoint_tests
        run_builder_tests
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [test-type]"
        echo ""
        echo "Test types:"
        echo "  unit      - Run unit tests only"
        echo "  network   - Run network integration tests"
        echo "  endpoints - Test against real API endpoints"
        echo "  builder   - Test builder pattern functionality"
        echo "  quick     - Run unit tests + endpoint validation"
        echo "  all       - Run complete test suite (default)"
        echo "  help      - Show this help message"
        echo ""
        echo "Examples:"
        echo "  $0              # Run all tests"
        echo "  $0 quick        # Quick validation"
        echo "  $0 endpoints    # Just test real APIs"
        ;;
    *)
        echo -e "${RED}❌ Unknown test type: $1${NC}"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac

echo -e "${GREEN}✅ Test run completed!${NC}"