#!/bin/bash

set -euxo pipefail

# Usage: set $PROBLEM_ID, $ANSWER_FILE, and $SOLVER (optional)
# Example: PROBLEM_ID=1 ANSWER_FILE=./test.json SOLVER=test ./tools/submission.sh
# 自分たちのサーバーの提出用エンドポイントを叩きます。ICFPC2023のサーバーにも提出されます。
# `gcloud auth login` しておく必要があります

# あと↓のコマンドも走らせてください。
# $ gcloud beta run services proxy fastapi-iam-auth --port=8080 --region asia-northeast1

SOLVER=${SOLVER:-default}
ANSWER_FILE=$(readlink -f "$ANSWER_FILE")

TOKEN=$(gcloud auth print-access-token)
# curl -X POST -H "Authorization: Bearer ${TOKEN}" -F file=@"${ANSWER_FILE}" "https://icfpc2023.negainoido.com/api/solutions/submit?id=${PROBLEM_ID}&solver=${SOLVER}"
curl -X POST -H "Authorization: Bearer ${TOKEN}" -F file=@"${ANSWER_FILE}" "http://localhost:8080/api/solutions/submit?id=${PROBLEM_ID}&solver=${SOLVER}"
