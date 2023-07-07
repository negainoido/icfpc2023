#!/bin/bash

set -euxo pipefail

# Usage: set $PROBLEM_ID, $ANSWER_FILE, and $SOLVER (optional)
# Example: PROBLEM_ID=1 ANSWER_FILE=./test.json SOLVER=test ./tools/submission.sh
# 自分たちのサーバーの提出用エンドポイントを叩きます。ICFPC2023のサーバーにも提出されます。

SOLVER=${SOLVER:-default}
ANSWER_FILE=$(readlink -f "$ANSWER_FILE")

curl -X POST -F file=@${ANSWER_FILE} "https://icfpc2023.negainoido.com/api/solutions/submit?id=${PROBLEM_ID}&solver=${SOLVER}"