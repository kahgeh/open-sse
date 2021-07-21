FROM rust:1.52 as runtime
ARG BASEPATH=/app
ARG SSE_VERSION=0.0.2
WORKDIR ${BASEPATH}}
RUN mkdir pkg
RUN curl -L -o pkg/sse-server.tar https://github.com/kahgeh/open-sse/releases/download/v${SSE_VERSION}/open-sse-server_${SSE_VERSION}_Linux_x86_64.tar.gz
RUN tar -xvf pkg/sse-server.tar
RUN chmod +x open-sse-server
ADD default.toml ./config/open-sse-server/
EXPOSE 80
CMD ["./open-sse-server"]