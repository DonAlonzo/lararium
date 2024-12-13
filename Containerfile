FROM alpine:3.21.0 AS alpine
FROM ubuntu:22.04 AS ubuntu

FROM ubuntu AS kodi
RUN DEBIAN_FRONTEND=noninteractive apt update \
 && apt install -y \
    software-properties-common \
 && add-apt-repository ppa:team-xbmc/ppa \
 && apt update \
 && apt install -y \
    kodi \
 && apt remove -y \
    software-properties-common \
 && rm -rf /var/lib/apt/lists/*
CMD ["kodi"]

FROM alpine AS jellyfin
RUN apk add --no-cache \
    jellyfin=10.10.3-r0 \
    ffmpeg=6.1.2-r1
CMD ["jellyfin", "--nowebclient"]
