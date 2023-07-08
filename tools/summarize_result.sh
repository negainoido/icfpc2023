#!/bin/bash

printf "|problem_id|score|best|comment|\n" > summary.md
printf "|----------|-----|----|-------|\n" >> summary.md
for i in $(seq 1 55); do
  best=$(curl "localhost:8080/api/best_solutions?id=$i" -s | jq -r .score)
  score=$(cat answer-$i.score.txt | sed -e 's/Score: //')
  re='^-?[0-9]+$'
  if [[ $score =~ $re ]] ; then
    if [[ $score -ge $best ]]; then
        comment="new best!($(($score * 100 / $best))%%)"
    else
        comment="$(($score * 100 / $best))%%"
    fi
  else
    comment=$score
    score=""
  fi
  printf "|$i|$score|$best|$comment|\n" >> summary.md
done
jq -Rs '.' summary.md | jq -c '{summary:"Batch Result", text_description:.}' > summary.json
