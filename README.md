# Lararium

## Build example extensions

```
cargo build --target wasm32-wasip2 --release -p jellyfin
cargo build --target wasm32-wasip2 --release -p kodi
```

## Run gateway

```
cargo run -p lararium-gateway --release
```

## Run station

```
cargo run -p lararium-station
```

# Relevant RFCs and specifications

```
https://datatracker.ietf.org/doc/html/rfc1035
https://datatracker.ietf.org/doc/html/rfc2131
https://datatracker.ietf.org/doc/html/rfc4506
https://datatracker.ietf.org/doc/html/rfc5531
https://datatracker.ietf.org/doc/html/rfc5661
https://datatracker.ietf.org/doc/html/rfc5905
https://datatracker.ietf.org/doc/html/rfc7530
https://datatracker.ietf.org/doc/html/rfc7531
https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html
```

## File System Layout

```txt
/system/dns/
/system/dhcp/
/system/nfs/
/system/ntp/
/applications/[app_name]/
              caldav
              calendar
              carddav
              contacts
              cinema
              drive
              git
              mail
              media
              mqtt
              photos
              recipes
              repository
              secrets
              webdav
/drive/*
       photos/
       videos/
       documents/
```
