#!/bin/bash
APP_PATH="/opt/ClusterNoodle"
BIN_PATH="/opt/ClusterNoodle/ClusterNoodle"
BIN_SYMLINK_PATH="/usr/local/bin/ClusterNoodle"
CONFIG_PATH="$HOME/.config/ClusterNoodle"

sudo rm -rf "$APP_PATH"
sudo rm -rf "$CONFIG_PATH"
sudo rm -f "$BIN_SYMLINK_PATH"
