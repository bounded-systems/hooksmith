#!/bin/bash
# Xtask wrapper script for Hooksmith
# This script makes it easier to run xtask commands from the project root

set -e

# Change to the xtask directory and run the command
cd "$(dirname "$0")/xtask"
cargo run -- "$@" 
