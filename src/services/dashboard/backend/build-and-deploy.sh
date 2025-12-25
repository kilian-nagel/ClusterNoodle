docker_image_name=nagelkilian/clusternoodle-dashboard-backend:"$1"
npm run build
docker build -t "$docker_image_name" .
docker push "$docker_image_name"
echo "Backend Docker image built and pushed: $docker_image_name"
echo "Run it with this command : docker run -e AGENT_URL=http://localhost:8090 -e FRONTEND_URL=http://localhost:8080 -p 3001:3001 $docker_image_name"