all: webapp wasm

APP_NAME := sveltekit-webapp
PROJECT_NAME := negainoido-icfpc2023
REGION := asia-northeast1
UNAME := $(shell uname)

webapp: wasm
	npm ci
	npm run build

wasm:
	cd wasm-sample && wasm-pack build --target web

fmt:
	cd wasm-sample && cargo fmt
	npm run format

docker: webapp
	# Darwin is not supported
	test $(UNAME) == "Linux"
	docker build -t $(APP_NAME) .
	docker tag $(APP_NAME) gcr.io/$(PROJECT_NAME)/$(APP_NAME):dev
	docker push gcr.io/$(PROJECT_NAME)/$(APP_NAME):dev

deploy: docker
	gcloud run deploy sveltekit-webapp-stg --image gcr.io/$(PROJECT_NAME)/$(APP_NAME):dev --port 3000 --region $(REGION)

proxy:
	gcloud beta run services proxy sveltekit-webapp-stg --port=3000 --region asia-northeast1
