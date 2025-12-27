# =========================================
# Stage 1: Build the Vue.js Application
# =========================================
ARG NODE_VERSION=23.11.0-alpine
ARG NGINX_VERSION=1.29

# Use a lightweight Node.js image for building (customizable via ARG)
FROM node:${NODE_VERSION} AS builder

# Set the working directory inside the container
WORKDIR /app

# Copy package-related files first to leverage Docker's caching mechanism
COPY package.json package-lock.json ./

# Install project dependencies using npm ci (ensures a clean, reproducible install)
RUN --mount=type=cache,target=/root/.npm npm ci

# Copy the rest of the application source code into the container
COPY . .

# Build the Vue.js application
RUN npm run build

# =========================================
# Stage 2: Prepare Nginx to Serve Static Files
# =========================================

FROM nginx:${NGINX_VERSION} AS runner

# Copy custom Nginx config
COPY nginx.conf /etc/nginx/nginx.conf


# Copy the static build output from the build stage to Nginx's default HTML serving directory
COPY --chown=nginx:nginx --from=builder /app/dist /usr/share/nginx/html

# Copy entrypoint to inject runtime env into a JS file
COPY docker-entrypoint.sh /docker-entrypoint.sh

# Ensure the entrypoint script is executable
USER root
RUN chmod +x /docker-entrypoint.sh \
	&& chown -R nginx:nginx /usr/share/nginx/html

# Prepare Nginx runtime directories with proper permissions for non-root
RUN mkdir -p /var/cache/nginx /var/run /var/log/nginx \
	&& chown -R nginx:nginx /var/cache/nginx /var/run /var/log/nginx

# Drop privileges back to the built-in non-root user
USER nginx

# Expose port 8080 to allow HTTP traffic
# Note: The default NGINX container now listens on port 8080 instead of 80 
EXPOSE 8080

# Start via entrypoint to generate env-config and run Nginx
ENTRYPOINT ["/docker-entrypoint.sh"]
CMD []