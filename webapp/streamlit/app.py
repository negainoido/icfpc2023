import json
import os

import pandas
import requests
import streamlit


def read_secrets():
    if SECRET := os.environ.get("SECRET"):
        return json.loads(SECRET)
    return streamlit.secrets


secrets = read_secrets()


class API:
    def __init__(self):
        self.endpoint = secrets["backend"]

    def hello(self):
        url = f"{self.endpoint}/hello"
        res = requests.get(url)
        return res.json()

    def show(self):
        url = f"{self.endpoint}/scores/show"
        res = requests.get(url)
        return res.json()

    def append(self, name: str, score: int, auth: str):
        url = f"{self.endpoint}/scores/append"
        data = {
            "name": name,
            "score": score,
        }
        headers = {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {auth}",
        }
        res = requests.post(url, json=data, headers=headers)
        return res.json()


streamlit.write("# Hello, World!")


api = API()
streamlit.write(api.hello())

# score dashboard
if streamlit.button(":arrows_counterclockwise: refresh"):
    streamlit.experimental_rerun()

rows = api.show()
streamlit.write(f"{len(rows)} records")
streamlit.dataframe(
    pandas.DataFrame(rows, columns=["id", "name", "score", "timestamp"]),
    hide_index=True,
    column_config={
        "score": streamlit.column_config.ProgressColumn(
            "score",
            format="%d",
            min_value=0,
            max_value=1000,
        ),
    },
)

# submit new record
streamlit.write("## submit new record")
file = streamlit.file_uploader("your file (dummy)")  # dummy
name = streamlit.text_input("name", str(file.name) if file else "")
score = int(
    streamlit.number_input("score", min_value=0, max_value=1000, step=1, value=0)
)
auth = streamlit.text_input("auth", value="aaa")
if streamlit.button("Submit"):
    if not name or not score:
        streamlit.warning("name and positive score are required")
        streamlit.stop()
    res = api.append(name, score, auth=auth)
    streamlit.json(res)
    if res.get("status") == "OK":
        streamlit.balloons()
