FROM rustembedded/cross:x86_64-unknown-linux-gnu-0.2.1

RUN apt-get -y update && \
    apt-get install -y libssl-dev
