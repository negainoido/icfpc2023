from io import StringIO

import pandas
import requests

import streamlit as st
from streamlit.runtime.uploaded_file_manager import UploadedFile

NUM_PROBLEM = 55


class API:
    def __init__(self):
        self.url = "https://icfpc2023.negainoido.com"

    def _get(self, endpoint: str, data=None):
        headers = {
            "Content-Type": "application/json",
        }
        response = requests.get(f"{self.url}{endpoint}", data=data, headers=headers)
        return response.json()

    def show(self):
        data = self._get("/api/solutions/show")
        return data

    def submit(self, problem_id: int, solver: str, uploaded_file: UploadedFile):
        stringio = StringIO(uploaded_file.getvalue().decode("utf-8"))
        response = requests.post(
            f"{self.url}/api/solutions/submit?id={problem_id}&solver={solver}",
            files={"file": stringio},
        )
        return response.json()

    def update_score(self):
        response = requests.post(
            f"{self.url}/api/solutions/update_score",
        )
        return response.json()


api = API()
if st.button(":arrows_counterclockwise: refresh"):
    st.experimental_rerun()

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

st.write("## Submission")
if st.checkbox("add filter"):
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
score_max = df["score"].max() * 1.1
st.dataframe(
    df,
    hide_index=True,
    column_config={
        "score": st.column_config.ProgressColumn(
            "score",
            format="%d",
            min_value=0,
            max_value=score_max,
        ),
    },
)
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
