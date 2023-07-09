import numpy as np
import pandas
import requests

import streamlit as st
import streamlit.components.v1 as components
from api import API
from streamlit.logger import get_logger

st.set_page_config(layout="wide")

NUM_PROBLEM = 90
logger = get_logger(__name__)


api = API()
query_params = st.experimental_get_query_params()
query_id = int(query_params.get("id", [1])[0]) or 1

rows = api.show()
df = pandas.DataFrame(
    rows,
    columns=[
        "id",
        "problem_id",
        "submittion_id",
        "solver",
        "status",
        "score",
        "timestamp",
    ],
)

html_string = """
<h1 id="countdown"></h1>
<script language="javascript">
  document.getElementById("countdown").style.color = "red";
  setInterval(() => {
      let targetTime = new Date('2023-07-10T21:00:00');
      let now = new Date();
      let diffMs = targetTime - now;
      let h = Math.floor(diffMs / 3600000); // hours
      let m = Math.floor((diffMs % 3600000) / 60000); // minutes
      let s = Math.round(((diffMs % 3600000) % 60000) / 1000); // seconds
      document.getElementById("countdown").innerText = `⏰ ${h}:${m}:${s}`;
  }, 1000);
</script>
"""
components.html(html_string, height=70)
with st.sidebar:
    components.html(html_string, height=70)

st.markdown(f":link: [svelte](https://icfpc2023.negainoido.com/{query_id})")

st.write("## Submissions")
st.write("### Summary")
df_summary = df.dropna(subset=["score"])
df_summary = df_summary.loc[df_summary.groupby("problem_id")["score"].idxmax()]
df_summary["thumbnail"] = "./app/static/img/" + df["problem_id"].astype(str) + ".png"
df_summary["tastes"] = (
    "./app/static/img/tastes-" + df["problem_id"].astype(str) + ".png"
)
score_sum = df_summary["score"][df_summary["score"] > 0].sum()


def add_link(df: pandas.DataFrame):
    df["link"] = (
        "https://icfpc2023.negainoido.com/"
        + df["problem_id"].astype(str)
        + "?solution_id="
        + df["id"].astype(str)
    )


add_link(df_summary)

df_summary["score_ratio"] = df_summary["score"].astype(int) / score_sum * 100

column_config = {
    "thumbnail": st.column_config.ImageColumn("thumbnail"),
    "tastes": st.column_config.ImageColumn("tastes"),
    "link": st.column_config.LinkColumn("link", width="small"),
}
st.dataframe(df_summary, column_config=column_config, hide_index=True)
st.info(f"Sum score = {score_sum:,}")

st.write("### by problem")
filter_problem_id = int(
    st.number_input(
        "problem_id",
        key="filter_problem_id",
        value=int(query_params.get("id", [1])[0]) or 1,
        min_value=1,
        max_value=NUM_PROBLEM,
    )
)
st.experimental_set_query_params(id=filter_problem_id)
df = df[df["problem_id"] == filter_problem_id]
st.write(f"{len(df)} records")
score_min = min(0, float(df["score"].min() or 0))
score_max = max(1000, float(df["score"].max() or 1000) * 1.1)
add_link(df)

st.dataframe(df, hide_index=True, column_config=column_config)

st.write("## Update score")
st.write("未取得なスコアをすべて更新するボタン")
if st.button("update score"):
    st.json(api.update_score())

st.write("## Submit new file")
problem_id = int(
    st.number_input("problem_id", value=1, min_value=1, max_value=NUM_PROBLEM)
)
solver = st.text_input("solver name", value="default")
jsonfile = st.file_uploader("JSON File")
if problem_id and solver and jsonfile:
    if st.button("submit"):
        st.json(api.submit(problem_id, solver, jsonfile))
