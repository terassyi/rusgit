FROM rust:latest

RUN apt update -y && \
    apt install -y git less bsdmainutils

CMD ["/bin/bash"]
