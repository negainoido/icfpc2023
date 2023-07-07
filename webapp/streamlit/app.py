from io import StringIO

import pandas
import requests

import streamlit as st
from streamlit.runtime.uploaded_file_manager import UploadedFile


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


api = API()
if st.button(":arrows_counterclockwise: refresh"):
    st.experimental_rerun()

rows = api.show()

st.write("## Submission")
st.write(f"{len(rows)} records")
st.dataframe(
    pandas.DataFrame(
        rows,
        columns=["id", "problem_id", "submittion_id", "solver", "score", "timestamp"],
    ),
    hide_index=True,
    column_config={
        "score": st.column_config.ProgressColumn(
            "score",
            format="%d",
            min_value=0,
        ),
    },
)

st.write("## Submit new file")
problem_id = int(st.number_input("problem_id", value=1, min_value=1, max_value=45))
solver = st.text_input("solver name", value="default")
jsonfile = st.file_uploader("JSON File")
if problem_id and solver and jsonfile:
    if st.button("submit"):
        st.json(api.submit(problem_id, solver, jsonfile))
