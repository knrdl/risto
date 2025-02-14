# platform parameter fixes https://github.com/docker/buildx/issues/395
FROM --platform=${BUILDPLATFORM:-linux/amd64} rust:1.84.1-alpine AS executable_builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release && \
    strip target/release/risto


FROM scratch

EXPOSE 8080/tcp

# provide example data if no volume is mounted
COPY data /data
VOLUME [ "/data" ]
WORKDIR /data

COPY --from=executable_builder --chmod=0444 --chown=0:0 /usr/src/app/target/release/risto /bin/risto

CMD [ "/bin/risto" ]
