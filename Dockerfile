FROM debian:8
EXPOSE 8080
CMD ["/redact-data"]
COPY target/release/ /
