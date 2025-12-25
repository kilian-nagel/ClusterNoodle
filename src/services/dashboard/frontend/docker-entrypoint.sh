#!/bin/sh
set -eu

# Defaults
: "${BACKEND_URL:=}"

# Generate runtime env config for the frontend
# File will be served statically by Nginx
cat > /usr/share/nginx/html/env-config.js <<EOF
window.__ENV = {
  // Primary runtime value (prefer runtime over build-time)
  BACKEND_URL: "${BACKEND_URL}"
};
EOF

# Start Nginx with the provided config
exec nginx -c /etc/nginx/nginx.conf -g 'daemon off;'