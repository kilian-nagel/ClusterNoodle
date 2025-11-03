#!/bin/bash
APP_PATH="/opt/ClusterNoodle"
BIN_PATH="/opt/ClusterNoodle/ClusterNoodle"
BIN_SYMLINK_PATH="/usr/local/bin/ClusterNoodle"
CONFIG_PATH="$HOME/.config/ClusterNoodle"
CONFIG_FILE_PATH="$CONFIG_PATH/conf.cluster_noodle"
TAR_PATH="/tmp/ClusterNoodle.tar.gz"

echo "Checking if docker is installed..."
if dpkg -l docker >/dev/null ; then
    echo "docker is installed."
else
    echo "Error : docker is not installed!"
    return 1;
fi

echo "Downloading last ClusterNoodle release..."

curl -s https://api.github.com/repos/kilian-nagel/ClusterNoodle/releases/latest \
| grep "browser_download_url" \
| cut -d '"' -f 4 \
| xargs -n 1 curl -L -o "$TAR_PATH"

echo "Extracting ClusterNoodle archive in $APP_PATH"
sudo mkdir -p $APP_PATH
sudo tar -xf "$TAR_PATH" -C "$APP_PATH"

echo "Setting up config folder in $CONFIG_PATH"
sudo mkdir -p $CONFIG_PATH

if ! [ -z "$CONFIG_FILE_PATH" ] && ! [ -e "$CONFIG_FILE_PATH" ]; then
    sudo touch "$CONFIG_FILE_PATH"
    sudo touch "$CONFIG_PATH/app.env"
fi
sudo chown -R "$(whoami)" $CONFIG_PATH

echo "Setting up executable..."
sudo chmod +x "$BIN_PATH"

if ! [ -z "$BIN_SYMLINK_PATH" ] && ! [ -e "$BIN_SYMLINK_PATH" ]; then
    sudo ln -s /opt/ClusterNoodle/ClusterNoodle "$BIN_SYMLINK_PATH"
fi

rm -f "$TAR_PATH"