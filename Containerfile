FROM ubuntu:22.04 AS build
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
