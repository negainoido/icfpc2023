#!/bin/bash

printf "|problem_id|score|best|comment|\n" > summary.md
printf "|----------|----:|---:|-------|\n" >> summary.md
for i in $(seq 1 90); do
  best=$(curl "localhost:8080/api/best_solutions?id=$i" -s | jq -r .score)
  score=$(cat answer-$i.score.txt | sed -e 's/Score: //')
  re='^-?[0-9]+$'
  if [[ $score =~ $re ]] ; then
    if [[ $score -ge $best ]]; then
        comment=$(printf "new best!(%.3e)" $(($score - $best)))
    else
        comment=$(printf "%.3e" $(($score - $best)))
    fi
  else
    comment=$score
    score=""
  fi
  printf "|%s|%'d|%'d|%s|\n" $i "$score" "$best" "$comment" >> summary.md
done
jq -Rs '.' summary.md | jq -c '{summary:"Batch Result", text_description:.}' > summary.json
