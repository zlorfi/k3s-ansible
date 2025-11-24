#!/usr/bin/env bash
# multi-ssh.sh — Connect to multiple Raspberry Pi nodes in tmux

# ---- CONFIG ----
HOSTS=(
  192.168.30.101
  192.168.30.102
  192.168.30.103
  192.168.30.104
)

USER_NAME="${USER_NAME:-pi}"   # default to 'pi' user for Raspberry Pi
SESSION="multi-ssh"

# ---- CHECKS ----
if ! command -v tmux >/dev/null 2>&1; then
  echo "tmux not found. Install with: brew install tmux" >&2
  exit 1
fi
if [ ${#HOSTS[@]} -eq 0 ]; then
  echo "No hosts configured." >&2
  exit 1
fi

# Kill existing session if it exists
tmux kill-session -t "$SESSION" 2>/dev/null

# ---- CREATE SESSION WITH FIRST HOST ----
echo "Creating tmux session and connecting to hosts..."
tmux new-session -d -s "$SESSION" "ssh ${USER_NAME}@${HOSTS[0]}"

# ---- ADD PANES FOR REMAINING HOSTS ----
for host in "${HOSTS[@]:1}"; do
  tmux split-window -t "$SESSION" -d "ssh ${USER_NAME}@${host}"
done

# Tile evenly
tmux select-layout -t "$SESSION" tiled

# Optional visuals
tmux set-option -t "$SESSION" pane-border-status top
tmux set-option -t "$SESSION" pane-border-format "#{pane_index} #{pane_current_command}"

# Enable synchronized input (type once → all panes)
tmux set-window-option -t "$SESSION" synchronize-panes on

# Display message
tmux display-message -t "$SESSION" "Connected to ${#HOSTS[@]} hosts. Synchronized input enabled. Press Ctrl-b + Shift-E to toggle sync."

# ---- ATTACH OR SWITCH ----
if [ -n "$TMUX" ]; then
  tmux switch-client -t "$SESSION"
else
  tmux attach-session -t "$SESSION"
fi