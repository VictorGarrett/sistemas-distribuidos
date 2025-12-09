# Use the official Golang image as the base
FROM golang:1.24-alpine AS builder

# Set environment variables
ENV CGO_ENABLED=0 \
    GOOS=linux \
    GOARCH=amd64




COPY proto-definitions /proto-definitions
COPY proto-go /proto-go

# Set working directory inside the container
WORKDIR /build

# Copy go.mod and go.sum files for dependency installation
COPY ./gateway/go.mod ./gateway/go.sum ./

# Download dependencies
RUN go mod download

# Copy the entire application source
COPY ./gateway .

# Build the Go binary
RUN go build -o /app ./cmd/main.go

# Final lightweight stage
FROM alpine:3.21 AS final

# Copy the compiled binary from the builder stage
COPY --from=builder /app /bin/app

# Expose the application's port
EXPOSE 8000

# Run the application
CMD ["bin/app", "-docker"]
