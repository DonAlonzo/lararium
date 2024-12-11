FROM ubuntu:22.04 AS kodi
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

FROM alpine:3.21.0 AS vlc
RUN apk add --no-cache vlc
CMD ["vlc"]
