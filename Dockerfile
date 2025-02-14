FROM rust:1.84.1-alpine as executable_builder

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
