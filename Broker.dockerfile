ARG BASEPATH=/app
ARG COMPONENT="open-sse-broker"
FROM rust:1.53.0 as planner
ARG BASEPATH
WORKDIR ${BASEPATH}
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM rust:1.53.0 as cacher
ARG BASEPATH
WORKDIR ${BASEPATH}
RUN cargo install cargo-chef
COPY --from=planner ${BASEPATH}/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM rust:1.53.0 as builder
ARG COMPONENT
ARG BASEPATH
WORKDIR ${BASEPATH}
COPY . .
# Copy over the cached dependencies
COPY --from=cacher ${BASEPATH}/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release --bin ${COMPONENT}

FROM rust:1.53.0 as runtime
ARG COMPONENT
ARG BASEPATH
WORKDIR ${BASEPATH}
ENV SSE_HTTP__PORT=80
EXPOSE 80
COPY --from=builder ${BASEPATH}/target/release/${COMPONENT} .
RUN chmod +x ./${COMPONENT}
ADD config config
RUN mkdir -p config/${COMPONENT}
COPY config/open-sse-broker/default.toml config/open-sse-broker/

CMD ["./open-sse-broker"]
