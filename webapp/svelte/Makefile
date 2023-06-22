all: webapp wasm

APP_NAME := sveltekit-webapp
PROJECT_NAME := negainoido-icfpc2023
REGION := asia-northeast1

webapp: wasm
	npm ci
	npm run build

wasm:
	cd wasm-sample && wasm-pack build --target web

fmt:
	cd wasm-sample && cargo fmt
	npm run format

docker:
	docker build -t $(APP_NAME) .
	docker tag $(APP_NAME) gcr.io/$(PROJECT_NAME)/$(APP_NAME)
	docker push gcr.io/$(PROJECT_NAME)/$(APP_NAME)

deploy: docker
	gcloud run deploy sveltekit-webapp --image gcr.io/$(PROJECT_NAME)/$(APP_NAME):latest --port 3000 --region $(REGION)
