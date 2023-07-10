#!/bin/bash

set -euo pipefail

# Usage: set $PROBLEM_ID
# Example: PROBLEM_ID=1 ANSWER_FILE=./test.json SOLVER=test ./tools/submission.sh
# 自分たちのサーバーの提出用エンドポイントを叩きます。ICFPC2023のサーバーにも提出されます。
# `gcloud auth login` しておく必要があります

# あと↓のコマンドも走らせてください。
# $ gcloud beta run services proxy fastapi-iam-auth --port=8080 --region asia-northeast1

cd "$(dirname "$0")/.."
TS=$(echo "2023-07-09T14:10:00" | jq -Rr '@uri')
tmp_dir="$(mktemp -d)"
TOKEN=$(gcloud auth print-access-token)

echo "tmp_dir: $tmp_dir"
for problem_id in $(seq 57 57); do
  echo "problem id: $problem_id"
  resp=$(curl -H "Authorization: Bearer ${TOKEN}" "http://localhost:8080/api/best_solutions?id=${problem_id}&ts=${TS}")
  echo "target id: $(echo "$resp" | jq -r '.id')"
  solver="$(echo "$resp" | jq -r '.solver')-opt"
  contents="$(echo "$resp" | jq -r '.contents')"
  echo "$contents" > "$tmp_dir/source-$problem_id.json"
  output="$tmp_dir/output-$problem_id.json"
  ./solver/target/release/optimizer --input "problems/problem-$problem_id.json" --source "$tmp_dir/source-$problem_id.json" --output "$output"
  # curl -X POST -H "Authorization: Bearer ${TOKEN}" -F file=@"$output" "http://localhost:8080/api/solutions/submit?id=${problem_id}&solver=${solver}"
done



