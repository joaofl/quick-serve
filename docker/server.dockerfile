# Use debian:bullseye-slim as the base image
FROM ubuntu:latest

# Copy the quick-serve binary from the relative path to the container
COPY ./target/debug/quick-serve /usr/local/bin/quick-serve
ENTRYPOINT ["/bin/sh", "-c", "timeout 10 quick-serve --dhcp=6767 -v --bind-ip=172.12.1.4"]
