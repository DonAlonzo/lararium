# Lararium

## Build example extensions

```
cargo build -p jellyfin --target wasm32-wasip2 --release
cargo build -p kodi --target wasm32-wasip2 --release
```

## Run gateway

```
cargo run -p lararium-gateway --release
```

## Run station

```
cargo run -p lararium-station
```
