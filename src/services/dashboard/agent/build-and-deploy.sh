docker_image_name=nagelkilian/clusternoodle-dashboard-agent:"$1"
npm run build
docker build -t "$docker_image_name" .
docker push "$docker_image_name"
echo "Backend Docker image built and pushed: $docker_image_name"
echo "Run it with this command : docker run -p 8090:8090 $docker_image_name"