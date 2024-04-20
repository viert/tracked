FROM ubuntu:jammy
RUN apt update && apt install -y ca-certificates
COPY target/release/tracked /usr/sbin/

RUN mkdir /etc/tracked
ADD tracked.toml /etc/tracked/

RUN mkdir -p /var/lib/tracked/db

ENTRYPOINT [ "/usr/sbin/tracked" ]
