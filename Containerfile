FROM alpine:3.21.0 AS alpine

FROM alpine AS kodi
RUN apk add --no-cache kodi=21.1-r3
CMD ["kodi"]

FROM alpine AS jellyfin
RUN apk add --no-cache \
    jellyfin=10.10.3-r0 \
    ffmpeg=6.1.2-r1
CMD ["jellyfin", "--nowebclient"]
