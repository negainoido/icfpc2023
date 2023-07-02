FROM rust:1.70-slim-bullseye as wasm-builder

WORKDIR /workspace
RUN cargo install wasm-pack
COPY wasm-sample .
RUN wasm-pack build --target web

FROM node:20-bullseye-slim as builder

WORKDIR /workspace
COPY . .
COPY --from=wasm-builder /workspace/pkg ./wasm-sample/pkg
RUN npm ci \
 && npm run build

FROM node:20-bullseye-slim

WORKDIR /app
COPY --from=builder /workspace/package*.json ./
COPY --from=builder /workspace/build ./build

CMD ["node", "build"]
