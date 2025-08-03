#!/bin/bash

# Comprehensive status script for the generated files registry system
# This provides an overview of the current state and available tools

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                GENERATED FILES REGISTRY STATUS               ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"

echo -e "\n${CYAN}📊 Current Registry Status:${NC}"
./scripts/compare-generated-files.sh

echo -e "\n${PURPLE}🔧 Available Management Scripts:${NC}"
echo -e "${YELLOW}  • scripts/compare-generated-files.sh${NC}     - Compare git files with registry"
echo -e "${YELLOW}  • scripts/validate-registry.sh${NC}           - Comprehensive validation"
echo -e "${YELLOW}  • scripts/update-all-checksums.sh${NC}        - Update all checksums"
echo -e "${YELLOW}  • scripts/cleanup-registry.sh${NC}            - Remove ignored files"
echo -e "${YELLOW}  • scripts/fix-generated-files-registry.sh${NC} - Add missing files & fix checksums"

echo -e "\n${PURPLE}📁 Registry File:${NC}"
echo -e "${YELLOW}  • config/generated-files.jsonc${NC}"

echo -e "\n${PURPLE}💾 Backup Files:${NC}"
BACKUP_COUNT=$(ls config/generated-files.jsonc.backup.* 2>/dev/null | wc -l)
if [ "$BACKUP_COUNT" -gt 0 ]; then
    echo -e "${GREEN}  • $BACKUP_COUNT backup files available${NC}"
    ls -la config/generated-files.jsonc.backup.* | tail -3 | while read line; do
        echo -e "${YELLOW}    $line${NC}"
    done
else
    echo -e "${YELLOW}  • No backup files found${NC}"
fi

echo -e "\n${PURPLE}🎯 Quick Actions:${NC}"
echo -e "${CYAN}  • Validate:${NC}     ./scripts/validate-registry.sh"
echo -e "${CYAN}  • Compare:${NC}      ./scripts/compare-generated-files.sh"
echo -e "${CYAN}  • Update:${NC}       ./scripts/update-all-checksums.sh"
echo -e "${CYAN}  • Cleanup:${NC}      ./scripts/cleanup-registry.sh"

echo -e "\n${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║                    SYSTEM READY! 🎉                          ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}" 
