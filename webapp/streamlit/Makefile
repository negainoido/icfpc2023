APPNAME := streamlit

deploy:
	gcloud run deploy $(APPNAME) --region "asia-northeast1" --source . --allow-unauthenticated --quiet
