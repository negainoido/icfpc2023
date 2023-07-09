import gc
import json

import pandas
import plotly.express as px
import requests
from PIL import Image, ImageDraw

import streamlit as st

NUM_PROBLEM = 90
query_params = st.experimental_get_query_params()


class Problem:
    @classmethod
    def get_from_file(cls, problem_id: int):
        with open(f"./resource/problems/problem-{problem_id}.json", "rt") as f:
            data = json.load(f)
            return data

    @classmethod
    def get_from_web(cls, problem_id: int):
        res = requests.get(
            "http://api.icfpcontest.com/problem",
            json={"problem_id": problem_id},
            headers={
                "Accept": "application/json",
            },
        )
        st.code(res.text)
        data_raw = res.json().get("Success")
        data = json.loads(data_raw)
        return data


class Figure:
    @classmethod
    def draw(cls, data):
        room = Image.new(
            "RGBA",
            (
                data.get("room_width"),
                data.get("room_height"),
            ),
            (210, 210, 210),
        )
        draw = ImageDraw.Draw(room)
        # stage
        draw.rectangle(
            (
                (data.get("stage_bottom_left")[0], data.get("stage_bottom_left")[1]),
                (
                    data.get("stage_bottom_left")[0] + data.get("stage_width"),
                    data.get("stage_bottom_left")[1] + data.get("stage_height"),
                ),
            ),
            outline=(0, 0, 0),
            fill=(240, 240, 240),
        )
        # attendees
        size = min(data.get("room_width"), data.get("room_height"))
        w = size / 300
        for a in data.get("attendees"):
            x = a.get("x")
            y = a.get("y")
            draw.ellipse(((x - w, y - w), (x + w, y + w)), fill="#ff0000")
        return room


problem_id = int(
    st.number_input(
        "problem_id",
        value=int(query_params.get("id", [1])[0]) or 1,
        min_value=1,
        max_value=NUM_PROBLEM,
    )
)
st.experimental_set_query_params(id=problem_id)
data = Problem.get_from_file(problem_id)

st.image(f"resource/img/{problem_id}.png")
# st.image(Figure.draw(data))
st.json(
    {
        "room_width": data.get("room_width"),
        "room_height": data.get("room_height"),
        "stage_width": data.get("stage_width"),
        "stage_height": data.get("stage_height"),
        "stage_bottom_left": data.get("stage_bottom_left"),
        "num_attendees": len(data.get("attendees")),
        "num_musicians": len(data.get("musicians")),
    }
)

st.write("##### attendees")
st.dataframe(
    pandas.DataFrame(data.get("attendees")),
    # column_config={
    #     "tastes": st.column_config.BarChartColumn(
    #         "tastes",
    #         width="medium",
    #     ),
    # },
)
st.write("##### tastes")
st.image(f"resource/img/tastes-{problem_id}.png")

ms = data.get("musicians")


def histogram(ms):
    from collections import defaultdict

    hist = defaultdict(int)
    for i in ms:
        hist[i] += 1
    hist = sorted(list(hist.items()))
    return pandas.DataFrame([{"楽器": x, "度数": y} for x, y in hist])


st.bar_chart(histogram(ms), x="楽器", y="度数")

st.write("### problem JSON spec")
st.json(data, expanded=False)

del data
gc.collect()
