import json

import pandas
import requests
from streamlit_agraph import Config, Edge, Node, agraph

import streamlit as st

NUM_PROBLEM = 55


class Problem:
    @classmethod
    def get_from_file(cls, problem_id: int):
        with open(f"./resource/problems/problem-{problem_id}.json", "rt") as f:
            data = json.load(f)
            return data


class API:
    def __init__(self):
        self.url = "https://icfpc2023.negainoido.com"

    def _get(self, endpoint: str, data=None):
        headers = {
            "Content-Type": "application/json",
        }
        response = requests.get(f"{self.url}{endpoint}", params=data, headers=headers)
        return response.json()

    def show(self):
        data = self._get("/api/solutions/show")
        return data

    def solution(self, solution_id: int):
        data = self._get("/api/solutions", data={"id": solution_id})
        return data


def viz(problem, solution):
    nodes_m = []
    nodes_a = []
    edges = []
    size = max(problem.get("room_width"), problem.get("room_height"))
    scale = 1200 / size
    config = Config(
        height=problem.get("room_height") * scale,
        width=problem.get("room_width") * scale,
        physics=False,
    )
    for i, a in enumerate(problem.get("attendees")):
        x = a["x"] * scale
        y = a["y"] * scale
        nodes_a.append(Node(id=f"a-{i}", chosen=False, fixed=True, x=x, y=y, size=3))
    for i, m in enumerate(solution.get("placements")):
        x = m["x"] * scale
        y = m["y"] * scale
        nodes_m.append(
            Node(id=f"m-{i}", chosen=False, color="green", fixex=True, x=x, y=y, size=3)
        )
    mid = int(
        st.number_input("musician_id", value=0, min_value=0, max_value=len(nodes_m) - 1)
    )
    for nodea in nodes_a:
        edges.append(Edge(source=nodea.id, target=nodes_m[mid].id))
    agraph(nodes_a + nodes_m, edges, config)


api = API()


st.write("### Select solution")
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
problem_id = int(
    st.number_input(
        "problem_id",
        value=1,
        min_value=1,
        max_value=NUM_PROBLEM,
    )
)
df = df[df["problem_id"] == problem_id]
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
    solution = api.solution(solution_id)
    solution = json.loads(solution.get("contents"))
    problem = Problem.get_from_file(problem_id)
    viz(problem, solution)
    st.json(
        {
            "room_width": problem.get("room_width"),
            "room_height": problem.get("room_height"),
            "stage_width": problem.get("stage_width"),
            "stage_height": problem.get("stage_height"),
            "stage_bottom_left": problem.get("stage_bottom_left"),
            "num_attendees": len(problem.get("attendees")),
            "num_musicians": len(problem.get("musicians")),
        }
    )
