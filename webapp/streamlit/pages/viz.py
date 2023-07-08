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
        st.write(response)
        st.write(response.text)
        return response.json()

    def show(self):
        data = self._get("/api/solutions/show")
        return data

    def solution(self, solution_id: int):
        data = self._get("/api/solutions", data={"id": solution_id})
        return data


api = API()


def select_from_judgeserver():
    st.write("### Select solution from judgeserver")
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
    ids = list(df["id"])
    solution_id = st.selectbox("id", ids)
    if solution_id:
        st.write(api.solution(solution_id))


select_from_judgeserver()
