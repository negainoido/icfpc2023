import json

from PIL import Image, ImageDraw

NUM_PROBLEM = 90


class Problem:
    @classmethod
    def get_from_file(cls, problem_id: int):
        with open(f"./static/problems/problem-{problem_id}.json", "rt") as f:
            data = json.load(f)
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
        # pillars
        for p in data.get("pillars"):
            x, y = p["center"]
            r = p["radius"]
            draw.ellipse(((x - r, y - r), (x + r, y + r)), fill="#eee", outline="#333")
        return room


for i in reversed(range(1, 1 + NUM_PROBLEM)):
    print(i)
    data = Problem.get_from_file(i)
    img = Figure.draw(data)
    img.save(f"./static/img/{i}.png")
