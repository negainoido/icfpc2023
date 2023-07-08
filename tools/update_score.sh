#!/bin/bash

set -euxo pipefail

# Example: ./tools/update_scores.sh
# ジャッジサーバーからスコアを取得して、自分たちのDBを更新します。
# `gcloud auth login` しておく必要があります


TOKEN=$(gcloud auth print-access-token)
curl -X POST -H "Authorization: Bearer ${TOKEN}" "https://icfpc2023.negainoido.com/api/solutions/update_score"
