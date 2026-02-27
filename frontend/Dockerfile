# Stage 1: Build
FROM node:22-alpine AS builder

WORKDIR /app

ARG VITE_API_BASE_URL=/api

COPY package*.json ./
RUN npm ci

COPY . .
RUN VITE_API_BASE_URL=${VITE_API_BASE_URL} npm run build

# Stage 2: Serve
FROM nginx:1.27-alpine AS runtime

COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]
