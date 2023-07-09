APPNAME := streamlit

run-local:
	streamlit run app.py --server.headless true --server.address 0.0.0.0 --server.port 8888

run-proxy:
	gcloud beta run services proxy streamlit-stg --port=8888 --region asia-northeast1

deploy:
	gcloud run deploy $(APPNAME) --region "asia-northeast1" --source . --allow-unauthenticated --quiet
