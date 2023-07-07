#!/bin/bash

set -euxo pipefail
# docker run -p 3306:3306 --name some-mysql -e MYSQL_ROOT_PASSWORD=password -e MYSQL_DATABASE=icfpc2023 -d mysql:8.0
# gcloud auth application-default login

SECRET=$(cat .secrets.json) uvicorn main:app --port 8080
