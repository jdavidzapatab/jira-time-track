# Stage 1: Build the frontend
FROM node:20-slim AS frontend-builder
WORKDIR /app/frontend
COPY frontend/package*.json ./
RUN npm install
COPY frontend/ ./
RUN npm run build

# Stage 2: Build the backend
FROM rust:1.85-slim AS backend-builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev libmariadb-dev git && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Create a dummy src/main.rs and src/lib.rs to build dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    touch src/lib.rs
RUN cargo build --release
# Remove the dummy source and copy the real ones
RUN rm -rf src/
COPY src/ ./src/
COPY build.rs ./
COPY migrations/ ./migrations/
# Ensure the dist folder exists (will be overwritten by frontend build)
RUN mkdir -p dist
# Capture git info if available, otherwise build.rs handles it
COPY .git ./.git
RUN cargo build --release

# Stage 3: Final runtime image
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 libmariadb3 ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=backend-builder /app/target/release/jira-time-track ./
COPY --from=frontend-builder /app/frontend/dist ./dist
COPY migrations ./migrations

EXPOSE 3000
CMD ["./jira-time-track"]
