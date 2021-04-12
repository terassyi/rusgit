FROM rust:latest

RUN apt update -y && \
    apt install -y git

CMD ["/bin/bash"]
