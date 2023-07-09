from io import StringIO
import os

import pandas
import requests

import streamlit as st
import streamlit.components.v1 as components
from streamlit.runtime.uploaded_file_manager import UploadedFile
from streamlit.logger import get_logger

from api import API

st.set_page_config(layout="wide")

NUM_PROBLEM = 90
logger = get_logger(__name__)


api = API()

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

st.markdown("[streamlit](https://icfpc2023.negainoido.com/1)")

st.write("## Submissions")
st.write("### Summary")
df_summary = df.dropna(subset=["score"])
df_summary = df_summary.loc[df_summary.groupby("problem_id")["score"].idxmax()]
st.dataframe(df_summary, hide_index=True)
score_sum = df_summary["score"][df_summary["score"] > 0].sum()
st.info(f"Sum score = {score_sum:,}")

st.write("### by problem")
filter_problem_id = int(
    st.number_input(
        "problem_id",
        key="filter_problem_id",
        value=1,
        min_value=1,
        max_value=NUM_PROBLEM,
    )
)
df = df[df["problem_id"] == filter_problem_id]
st.write(f"{len(df)} records")
score_min = min(0, float(df["score"].min() or 0))
score_max = max(1000, float(df["score"].max() or 1000) * 1.1)
st.dataframe(
    df,
    hide_index=True,
    column_config={
        "score": st.column_config.ProgressColumn(
            "score",
            format="%d",
            min_value=score_min,
            max_value=score_max,
        ),
    },
)
with st.expander("debug"):
    st.write((score_min, score_max))
    st.write(df)

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
