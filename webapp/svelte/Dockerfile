FROM node:20-bullseye-slim

WORKDIR /app
COPY ./package*.json ./
COPY ./build ./build

CMD ["node", "build"]
