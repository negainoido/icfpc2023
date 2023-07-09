#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")/.."

BIN="./solver/target/release"
SOLVER=nobishiro
SHA=${2:-""}
PNUM=${PNUM:-4}
DRY_RUN=${DRY_RUN:-""}

function run_and_eval() {
    echo evaluating $1
    set -euo pipefail
    problem=$(basename $1)
    problem_wo_ext=${problem%\.json}
    problem_id=${problem_wo_ext#problem-}
    local input="problems/$problem"
    local output="output/nobishiro-${problem_id}.txt"
    opts=""
    if [[ $problem_id -gt 55 ]]; then
        opts="--sync-effect"
    fi
    $BIN/$SOLVER --input $input $opts > $output
}

export -f run_and_eval 
export BIN SOLVER SHA

error=0
(find problems -name '*.json' | sort -V | head -n 90 | xargs -P $PNUM -L 1 -I {} bash -c 'run_and_eval {}') \
  || error=1
exit $error
