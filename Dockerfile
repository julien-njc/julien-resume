FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        fonts-dejavu-core \
        pandoc \
        poppler-utils \
        python3 \
        python3-docx \
        wkhtmltopdf \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
