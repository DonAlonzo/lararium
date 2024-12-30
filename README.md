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
