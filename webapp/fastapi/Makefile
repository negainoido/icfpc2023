APPNAME := fastapi

deploy:
	gcloud run deploy $(APPNAME) --region "asia-northeast1" --source . --allow-unauthenticated --quiet

run-local:
	bash ./run-local.sh
