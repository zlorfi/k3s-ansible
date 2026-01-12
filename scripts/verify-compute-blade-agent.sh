#!/bin/bash

# Compute Blade Agent Verification Script
# This script checks the compute-blade-agent installation on worker nodes

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INVENTORY="${SCRIPT_DIR}/inventory/hosts.ini"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Compute Blade Agent Verification Script                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}\n"

# Parse worker nodes from inventory
echo -e "${YELLOW}Parsing worker nodes from inventory...${NC}"
WORKERS=$(grep -E "^\[worker\]" -A 100 "$INVENTORY" | grep -E "^cm4-|^pi-worker|^cb-0" | grep -v "^\[" | awk '{print $1}')

if [ -z "$WORKERS" ]; then
    echo -e "${RED}No worker nodes found in inventory${NC}"
    exit 1
fi

echo -e "${GREEN}Found worker nodes: ${WORKERS//$'\n'/, }${NC}\n"

# Check each worker
for worker in $WORKERS; do
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}Checking: ${worker}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    # Get host IP from inventory
    HOST_IP=$(grep -A 0 "^${worker}" "$INVENTORY" | grep ansible_host | awk '{print $NF}' | cut -d= -f2)

    if [ -z "$HOST_IP" ]; then
        echo -e "${RED}Could not find IP for ${worker}${NC}\n"
        continue
    fi

    echo -e "Host: ${worker} (${HOST_IP})"

    # Check connectivity
    echo -n "Network: "
    if ping -c 1 -W 2 "$HOST_IP" &> /dev/null; then
        echo -e "${GREEN}✓ Reachable${NC}"
    else
        echo -e "${RED}✗ Unreachable${NC}\n"
        continue
    fi

    # Check compute-blade-agent service
    echo -n "Service Status: "
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "pi@${HOST_IP}" \
        "systemctl is-active compute-blade-agent" &> /dev/null; then
        echo -e "${GREEN}✓ Running${NC}"
    else
        echo -e "${RED}✗ Not running${NC}"
    fi

    # Check binary
    echo -n "Binary: "
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "pi@${HOST_IP}" \
        "[ -f /usr/bin/compute-blade-agent ]" &> /dev/null; then
        echo -e "${GREEN}✓ Installed${NC}"

        # Try to get version
        VERSION=$(ssh -o ConnectTimeout=5 -o BatchMode=yes "pi@${HOST_IP}" \
            "/usr/bin/compute-blade-agent --version 2>/dev/null" || echo "unknown")
        echo "        Version: $VERSION"
    else
        echo -e "${RED}✗ Not found${NC}"
    fi

    # Check config
    echo -n "Config: "
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "pi@${HOST_IP}" \
        "[ -f /etc/compute-blade-agent/config.yaml ]" &> /dev/null; then
        echo -e "${GREEN}✓ Found${NC}"
    else
        echo -e "${YELLOW}⚠ Not found${NC}"
    fi

    # Check systemd service file
    echo -n "Systemd Unit: "
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "pi@${HOST_IP}" \
        "[ -f /etc/systemd/system/compute-blade-agent.service ]" &> /dev/null; then
        echo -e "${GREEN}✓ Installed${NC}"
    else
        echo -e "${RED}✗ Not found${NC}"
    fi

    echo ""
done

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Verification complete!${NC}\n"

echo -e "${YELLOW}To view logs on a specific node, run:${NC}"
echo -e "  ssh pi@<host-ip>"
echo -e "  sudo journalctl -u compute-blade-agent -f\n"

echo -e "${YELLOW}To restart the service:${NC}"
echo -e "  ssh pi@<host-ip>"
echo -e "  sudo systemctl restart compute-blade-agent\n"
