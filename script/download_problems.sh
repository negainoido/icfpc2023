#!/bin/bash

for i in $(seq 1 45); do
  curl -L http://api.icfpcontest.com/problem?problem_id=$i |
    jq -r '.Success' |
    jq -c . > /tmp/json
  cp /tmp/json problems/problem-$i.json
  cp /tmp/json webapp/streamlit/resource/problems/problem-$i.json
done
