#!/bin/bash
# Serverで実行する用のスクリプトです。
# ローカルで実行する場合は以下のコマンドでcloud runのプロキシを立ててください
# gcloud beta run services proxy fastapi-iam-auth --port=8080 --region asia-northeast1

set -euo pipefail

cd "$(dirname "$0")/.."

BIN="./solver/target/release"
SOLVER=${1:?is not set}
SHA=${2:-""}
PNUM=${PNUM:-4}
DRY_RUN=${DRY_RUN:-""}

mkdir -p output

function run_and_eval() {
    echo evaluating $1
    set -euo pipefail
    problem=$(basename $1)
    problem_wo_ext=${problem%\.json}
    problem_id=${problem_wo_ext#problem-}
    local input="problems/$problem"
    local output="output/${problem/problem/answer}"
    local score="${output/\.json/.score.txt}"
    local submission="${output/\.json/.submission.json}"
    $BIN/$SOLVER --input $input --output $output
    $BIN/evaluator --input $input --solution $output | tee $score
    if [[ "$DRY_RUN" == "false" ]]; then
        curl -X POST -F file=@"${output}" \
            "http://localhost:8080/api/solutions/submit?id=${problem_id}&solver=${SOLVER}%28${SHA:0:5}%29" \
            | tee $submission
        echo
        echo submitted
    fi
}

export -f run_and_eval 
export BIN SOLVER SHA

error=0
(find problems -name '*.json' | sort -V | head -n 55 | xargs -P $PNUM -L 1 -I {} bash -c 'run_and_eval {}') \
  || error=1
cd output
../tools/summarize_result.sh
exit $error
