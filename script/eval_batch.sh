#!/bin/bash

set -euo pipefail

cd "$(dirname "$0")/.."

BIN="./solver/target/release"
SOLVER=${1:?is not set}
PNUM=${PNUM:-4}

mkdir -p output

function run_and_eval() {
    echo evaluating $1
    problem=$(basename $1)
    local input="problems/$problem"
    local output="output/${problem/problem/answer}"
    local score="${output/\.json/.score.txt}"
    $BIN/$SOLVER --input $input --output $output
    $BIN/evaluator --input $input --solution $output | tee $score
}

export -f run_and_eval 
export BIN SOLVER

find problems -name '*.json' | xargs -P $PNUM -L 1 -I {} bash -c 'run_and_eval {}'

