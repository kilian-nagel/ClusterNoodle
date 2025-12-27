#!bin/bash
set -euo pipefail
# Merges multiple local user exisiting .env with new .env file

if [ "$#" -lt 2 ]; then
    echo "Usage: $0 existing.env new.env"
    exit 1
fi

existing_env_file="$1"
new_env_file="$2"

# Make backup of existing .env file.
cp "$new_env_file" "$new_env_file.bak"

current_env_data=$(cat "$existing_env_file")
while IFS= read -r line; do
    echo "Text read from line: $line"
    env_variable_name=$(echo "$line" | grep -oE "^[^=]+")
    env_variable_value=$(echo "$line" | grep -oE "=.+" | cut -d= -f2)
    new_env_variable_line=$(grep -E "$env_variable_name" "$new_env_file" || true)
    new_env_variable_value=$(echo "$new_env_variable_line" | grep -oE "=.+" | cut -d= -f2)
    echo "Variable name : $env_variable_name"
    echo "Variable value : $env_variable_value"
    echo "New variable value: $new_env_variable_value"
    sed -i "s|$env_variable_name=.*|$env_variable_name=$new_env_variable_value|g" "$existing_env_file"
done < "$existing_env_file"