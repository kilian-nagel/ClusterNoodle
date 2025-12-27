#!/bin/bash
set -e

docker_image_name=nagelkilian/clusternoodle-dashboard-frontend:"$1"
npm run build
docker build -t "$docker_image_name" .
docker push "$docker_image_name"
echo "Frontend Docker image built and pushed: $docker_image_name"
echo "Run it with this command : docker run -e BACKEND_URL=http://localhost:3001 -p 8080:8080 $docker_image_name"