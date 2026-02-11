#!/bin/bash

# setup_user.sh - Interactive script to create the initial admin user

set -e

# ANSI colors
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Mana Panel User Setup ===${NC}"

# Check if backend binary exists in target/release or debug
if [ -f "./target/release/mana-panel-backend" ]; then
    BACKEND="./target/release/mana-panel-backend"
elif [ -f "./target/debug/mana-panel-backend" ]; then
    BACKEND="./target/debug/mana-panel-backend"
elif [ -f "./backend/target/release/mana-panel-backend" ]; then
    BACKEND="./backend/target/release/mana-panel-backend"
elif [ -f "./backend/target/debug/mana-panel-backend" ]; then
    BACKEND="./backend/target/debug/mana-panel-backend"
# Handle case when installed in standard location (e.g., via script)
elif command -v mana-panel-backend &> /dev/null; then
    BACKEND="mana-panel-backend"
else
    echo -e "${RED}Error: mana-panel-backend binary not found.${NC}"
    echo "Please build the project first or run this script from the project root."
    exit 1
fi

echo -e "Using backend binary: $BACKEND"

# Prompt for username
read -p "Enter desired username: " USERNAME

if [ -z "$USERNAME" ]; then
    echo -e "${RED}Error: Username cannot be empty.${NC}"
    exit 1
fi

# Execute backend with create-user command
echo -e "\nCreating user '$USERNAME'..."
$BACKEND create-user --username "$USERNAME"

echo -e "\n${GREEN}Setup script completed.${NC}"
