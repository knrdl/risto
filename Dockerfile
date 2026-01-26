# platform parameter fixes https://github.com/docker/buildx/issues/395
FROM --platform=${BUILDPLATFORM:-linux/amd64} rust:1.93.0-alpine AS executable_builder

WORKDIR /usr/src/app
COPY . .
RUN cargo build --release && \
    strip target/release/risto


FROM scratch

WORKDIR /
EXPOSE 8080/tcp

# provide example data if no volume is mounted
COPY data /data
VOLUME [ "/data" ]

COPY --from=executable_builder --chmod=0555 --chown=0:0 /usr/src/app/target/release/risto /risto
COPY --chmod=0444 www /www

CMD [ "/risto" ]
