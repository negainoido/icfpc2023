#!/bin/bash

for i in $(seq 1 1000); do
  echo Fetching $i
  curl -sL http://api.icfpcontest.com/problem?problem_id=$i > /tmp/json
  if ( grep "Success" /tmp/json >/dev/null ); then
    echo Success
    cat /tmp/json |
      jq -r '.Success' |
      jq -c . > /tmp/json2
    cp /tmp/json2 problems/problem-$i.json
    cp /tmp/json2 webapp/streamlit/resource/problems/problem-$i.json
    cp /tmp/json2 webapp/fastapi/resource/problems/problem-$i.json
  else
    echo Failure
    break
  fi
done

echo Done
