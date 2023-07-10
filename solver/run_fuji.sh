#!/bin/bash

# Usage: set $START, $END
# Example: PROBLEM_ID=1 ANSWER_FILE=./test.json SOLVER=test ./tools/submission.sh
# 自分たちのサーバーの提出用エンドポイントを叩きます。ICFPC2023のサーバーにも提出されます。
# `gcloud auth login` しておく必要があります

# あと↓のコマンドも走らせてください。
# $ gcloud beta run services proxy fastapi-iam-auth --port=8080 --region asia-northeast1

TOKEN=$(gcloud auth print-access-token)
OPTIMIZER=fuji
TIMEOUT=${TIMEOUT:-30}

for PROBLEM_ID in $(cat ../problems/list.txt | head -n 50); do

  echo $OPTIMIZER PROBLEM_ID=$PROBLEM_ID TIMEOUT=$TIMEOUT

  curl -H "Authorization: Bearer ${TOKEN}" "http://localhost:8080/api/best_solutions?id=${PROBLEM_ID}" >/tmp/best.json
  SOLVER="$(cat /tmp/best.json | jq -r .solver)-fuji"
  cat /tmp/best.json | jq -r .contents > /tmp/solution.json
  cargo run --bin $OPTIMIZER --release -- --input ../problems/problem-${PROBLEM_ID}.json --solution /tmp/solution.json --output /tmp/out.json --timeout $TIMEOUT
  if [ $? -eq 0 ]; then
    echo Posting
    echo file=@/tmp/out.json "http://localhost:8080/api/solutions/submit?id=${PROBLEM_ID}&solver=${SOLVER}"
    curl -X POST -H "Authorization: Bearer ${TOKEN}" -F file=@/tmp/out.json "http://localhost:8080/api/solutions/submit?id=${PROBLEM_ID}&solver=${SOLVER}"
  else
    echo Failed
  fi
  echo

done
