#!/bin/bash

set -euo pipefail

# Usage: set $START, $END
# Example: PROBLEM_ID=1 ANSWER_FILE=./test.json SOLVER=test ./tools/submission.sh
# 自分たちのサーバーの提出用エンドポイントを叩きます。ICFPC2023のサーバーにも提出されます。
# `gcloud auth login` しておく必要があります

# あと↓のコマンドも走らせてください。
# $ gcloud beta run services proxy fastapi-iam-auth --port=8080 --region asia-northeast1

cd "$(dirname "$0")/.."
tmp_dir="$(mktemp -d)"
TOKEN=$(gcloud auth print-access-token)
optimizer=${OPTIMIZER:-yamanobori_optimizer}
start=${START:-1}
end=${END:-90}

pushd ./solver
cargo build --release
popd

echo "tmp_dir: $tmp_dir"
for problem_id in $(seq "$start" "$end"); do
  echo "problem id: $problem_id"
  resp=$(curl -H "Authorization: Bearer ${TOKEN}" "http://localhost:8080/api/best_solutions?id=${problem_id}")
  echo "target id: $(echo "$resp" | jq -r '.id')"
  solver="$(echo "$resp" | jq -r '.solver')-opt"
  contents="$(echo "$resp" | jq -r '.contents')"
  echo "$contents" > "$tmp_dir/source-$problem_id.json"
  output="$tmp_dir/output-$problem_id.json"
  "./solver/target/release/$optimizer" --input "problems/problem-$problem_id.json" --solution "$tmp_dir/source-$problem_id.json" --output "$output"
  curl -X POST -H "Authorization: Bearer ${TOKEN}" -F file=@"$output" "http://localhost:8080/api/solutions/submit?id=${problem_id}&solver=${solver}"
done



