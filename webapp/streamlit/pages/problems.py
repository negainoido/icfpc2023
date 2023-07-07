import json
from collections import defaultdict

import plotly.express as px
from PIL import Image, ImageDraw

import streamlit as st


class Problem:
    @classmethod
    def get(cls, problem_id: int):
        data = json.load(open(f"./resource/problems/problem-{problem_id}.json", "rt"))
        return data


class Figure:
    @classmethod
    def musicians_histogram(cls, data):
        h = defaultdict(int)
        for i in data.get("musicians"):
            h[i] += 1
        h = list(h.items())
        h.sort()
        return dict(h)

    @classmethod
    def draw(cls, data):
        room = Image.new(
            "RGBA",
            (
                data.get("room_width"),
                data.get("room_height"),
            ),
            (200, 200, 200),
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
            fill=(220, 220, 220, 200),
        )
        # attendees
        for a in data.get("attendees"):
            x = a.get("x")
            y = a.get("y")
            w = 8
            draw.rectangle(((x - w, y - w), (x + w, y + w)), fill=(255, 0, 0))
        return room


problem_id = int(st.number_input("problem_id", value=1, min_value=1, max_value=100))
data = Problem.get(problem_id)
st.image(Figure.draw(data))

st.write("### musicians")
ms = data.get("musicians")
st.plotly_chart(px.histogram(ms))
st.write(Figure.musicians_histogram(data))

st.write("### attendees")
st.json(
    {
        "num": len(data.get("attendees")),
    }
)

st.write("### problem JSON spec")
st.write(data)
