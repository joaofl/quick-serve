# FROM debian:bookworm-slim
FROM ubuntu:latest

RUN apt-get update && \
    apt-get install -y isc-dhcp-client && \
    rm -rf /var/lib/apt/lists/*

# For some weird reason, I pass the port 6768 but it opens on 6767
# so the client also use 6767
ENTRYPOINT ["/bin/sh", "-c", "timeout 10 dhclient -4 -d -v -p 6768"]
