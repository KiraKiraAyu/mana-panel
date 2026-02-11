#!/bin/bash

# Mana Panel Backend Cleanup Script
# This script helps terminate all mana-panel-backend processes

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=====================================${NC}"
echo -e "${BLUE}  Mana Panel Backend Cleanup Script ${NC}"
echo -e "${BLUE}=====================================${NC}\n"

# Check if running on Windows (Git Bash/MSYS)
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
    echo -e "${YELLOW}Detected Windows environment${NC}"
    echo "Searching for mana-panel-back processes..."

    # Try to find and kill processes on Windows
    tasklist | grep -i "mana-panel" || {
        echo -e "${GREEN}No mana-panel processes found!${NC}"
        exit 0
    }

    echo -e "${YELLOW}Attempting to kill processes...${NC}"
    taskkill //F //IM mana-panel-back.exe 2>/dev/null || \
    taskkill //F //IM mana-panel-backend.exe 2>/dev/null || \
    echo -e "${RED}Failed to kill processes. They may not exist or you need admin rights.${NC}"

    exit 0
fi

# For Linux/macOS
echo "Searching for mana-panel backend processes..."

# Find all processes
PIDS=$(pgrep -f "mana-panel-back" || true)

if [ -z "$PIDS" ]; then
    echo -e "${GREEN}✓ No mana-panel-back processes found!${NC}"
    echo "System is clean."
    exit 0
fi

# Count processes
COUNT=$(echo "$PIDS" | wc -l | xargs)
echo -e "${YELLOW}Found ${COUNT} mana-panel-back process(es):${NC}\n"

# Display process details
echo -e "${BLUE}PID\tCPU%\tMEM%\tCOMMAND${NC}"
echo "----------------------------------------"
ps -p $PIDS -o pid=,pcpu=,pmem=,comm= 2>/dev/null || {
    # Fallback if ps command fails
    echo "$PIDS"
}
echo ""

# Ask for confirmation
read -p "Do you want to terminate these processes? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Operation cancelled.${NC}"
    exit 0
fi

echo -e "${YELLOW}Terminating processes...${NC}"

# Try graceful shutdown first (SIGTERM)
for PID in $PIDS; do
    if kill -15 "$PID" 2>/dev/null; then
        echo -e "${GREEN}✓ Sent SIGTERM to PID $PID${NC}"
    else
        echo -e "${RED}✗ Failed to send SIGTERM to PID $PID (may require sudo)${NC}"
    fi
done

# Wait a bit for graceful shutdown
sleep 2

# Check if any processes are still running
REMAINING=$(pgrep -f "mana-panel-back" || true)

if [ -z "$REMAINING" ]; then
    echo -e "\n${GREEN}✓ All processes terminated successfully!${NC}"
    exit 0
fi

# Force kill remaining processes (SIGKILL)
echo -e "${YELLOW}Some processes are still running. Forcing termination...${NC}"

for PID in $REMAINING; do
    if kill -9 "$PID" 2>/dev/null; then
        echo -e "${GREEN}✓ Sent SIGKILL to PID $PID${NC}"
    elif sudo kill -9 "$PID" 2>/dev/null; then
        echo -e "${GREEN}✓ Sent SIGKILL to PID $PID (with sudo)${NC}"
    else
        echo -e "${RED}✗ Failed to kill PID $PID${NC}"
    fi
done

# Final check
sleep 1
FINAL_CHECK=$(pgrep -f "mana-panel-back" || true)

if [ -z "$FINAL_CHECK" ]; then
    echo -e "\n${GREEN}✓ All processes have been terminated!${NC}"
    echo -e "${BLUE}You can now safely restart with: pnpm run dev${NC}"
else
    echo -e "\n${RED}⚠ Warning: Some processes could not be terminated:${NC}"
    ps -p $FINAL_CHECK -o pid=,comm= 2>/dev/null || echo "$FINAL_CHECK"
    echo -e "${YELLOW}Try running this script with sudo or reboot your system.${NC}"
    exit 1
fi
