#!/bin/bash

# Git Worktree Wrapper Script
# This script provides guidance for worktree management

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to show worktree guidance
show_worktree_guidance() {
    echo -e "${PURPLE}🌳 Worktree Management${NC}"
    echo -e "${PURPLE}==================${NC}"
    echo ""
    echo -e "${RED}❌ Please use worktree commands instead of git worktree:${NC}"
    echo ""
    echo -e "${CYAN}  📋 List worktrees:     ${GREEN}cargo xtask worktree list${NC}"
    echo -e "${CYAN}  ➕ Create worktree:    ${GREEN}cargo xtask worktree create --branch <branch>${NC}"
    echo -e "${CYAN}  🔄 Switch worktree:    ${GREEN}cargo xtask worktree switch --worktree <name>${NC}"
    echo -e "${CYAN}  🗑️  Remove worktree:    ${GREEN}cargo xtask worktree remove --worktree <name>${NC}"
    echo -e "${CYAN}  🛠️  Setup tools:        ${GREEN}cargo xtask worktree setup${NC}"
    echo ""
    echo -e "${YELLOW}  📁 Or use ${GREEN}git xworktree${YELLOW} for direct git worktree access${NC}"
    echo ""
    echo -e "${GREEN}💡 Tip: Worktrees are now created in .wt/ directory by default${NC}"
    echo ""
    echo -e "${BLUE}🔧 Available aliases:${NC}"
    echo -e "${CYAN}  git wtl${NC}  - List worktrees"
    echo -e "${CYAN}  git wtc${NC}  - Create worktree"
    echo -e "${CYAN}  git wts${NC}  - Switch worktree"
    echo -e "${CYAN}  git wtr${NC}  - Remove worktree"
    echo ""
}

# Function to show current worktree status
show_worktree_status() {
    echo -e "${BLUE}📊 Current Worktree Status:${NC}"
    echo -e "${BLUE}========================${NC}"
    
    if command -v cargo >/dev/null 2>&1; then
        echo -e "${CYAN}Running: cargo xtask worktree list${NC}"
        cargo xtask worktree list 2>/dev/null || echo -e "${YELLOW}⚠️  Cargo xtask not available${NC}"
    else
        echo -e "${YELLOW}⚠️  Cargo not available${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}📁 .wt Directory Contents:${NC}"
    if [ -d ".wt" ]; then
        ls -la .wt/ 2>/dev/null || echo -e "${YELLOW}⚠️  Cannot list .wt directory${NC}"
    else
        echo -e "${YELLOW}⚠️  .wt directory not found${NC}"
    fi
}

# Main execution
main() {
    # Check if any arguments were passed
    if [ $# -eq 0 ]; then
        show_worktree_guidance
        echo ""
        show_worktree_status
        exit 0
    fi
    
    # If arguments were passed, show guidance but also suggest the direct command
    show_worktree_guidance
    echo ""
    echo -e "${YELLOW}🔧 To execute the original git worktree command, use:${NC}"
    echo -e "${GREEN}git xworktree $*${NC}"
    echo ""
    echo -e "${YELLOW}Or use the worktree aliases:${NC}"
    echo -e "${GREEN}git wtl${NC}  - List worktrees"
    echo -e "${GREEN}git wtc${NC}  - Create worktree"
    echo -e "${GREEN}git wts${NC}  - Switch worktree"
    echo -e "${GREEN}git wtr${NC}  - Remove worktree"
    echo ""
}

# Run main function
main "$@" 
