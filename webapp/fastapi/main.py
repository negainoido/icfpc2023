import json
import os

import pymysql
import requests
from google.cloud import storage
from google.cloud.sql.connector import Connector
from pydantic import BaseModel

from fastapi import FastAPI, UploadFile

secrets = json.loads(os.environ.get("SECRET", ""))
app = FastAPI()
connector = Connector()
storage_client = storage.Client()


def get_icfpc_token():
    data = {
        "username_or_email": secrets["icfpc"]["username"],
        "password": secrets["icfpc"]["password"],
    }
    resp = requests.post(
        "https://api.icfpcontest.com/login",
        json=data,
        headers={"Content-Type": "application/json"},
    )
    json_resp = resp.json()
    return json_resp["Success"]


icfpc_token = get_icfpc_token()


def submit_solution_to_icfpc(problem_id: int, contents: str):
    data = {"problem_id": problem_id, "contents": contents}
    resp = requests.post(
        "https://api.icfpcontest.com/submission",
        json=data,
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {icfpc_token}",
        },
    )

    return resp.text.replace('"', '')


def get_submission_from_icfpc(submission_id: str):
    id = submission_id.replace('"', '')
    resp = requests.get(
        f"https://api.icfpcontest.com/submission?submission_id={id}",
        headers={
            "Content-Type": "application/json",
            "Authorization": f"Bearer {icfpc_token}",
        },
    )
    json = resp.json()
    print(json)

    return json

class Scores:
    def __init__(self):
        self.con_type = secrets["database"]["type"]
        if self.con_type == "cloudsql":
            self.name = secrets["database"]["name"]
        elif self.con_type == "tcp":
            self.host = secrets["database"]["host"]
            self.port = secrets["database"]["port"]

        self.user = secrets["database"]["user"]
        self.password = secrets["database"]["password"]
        self.database = secrets["database"]["database"]
        self.bucket_name = secrets["bucket_name"]
        self.blob_prefix = secrets["blob_prefix"]
        self._init_table()

    def con(self):
        if self.con_type == "cloudsql":
            return connector.connect(
                self.name,
                "pymysql",
                user=self.user,
                password=self.password,
                db=self.database,
            )
        elif self.con_type == "tcp":
            return pymysql.connect(
                host=self.host,
                port=self.port,
                user=self.user,
                password=self.password,
                database=self.database,
            )
        else:
            raise Exception("invalid con_type:", self.con_type)

    def _init_table(self):
        sql = """
        CREATE TABLE IF NOT EXISTS solutions(
            id INT(11) AUTO_INCREMENT NOT NULL PRIMARY KEY, 
            problem_id INT(11) NOT NULL,
            status VARCHAR(255) NULL,
            submission_id VARCHAR(255) NOT NULL,
            solver VARCHAR(255) NOT NULL,
            score INT NULL,
            ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        );
        """
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql)
            con.commit()

    def show(self):
        sql = """
        SELECT id, problem_id, submission_id, solver, status, score, ts
        FROM solutions
        ORDER BY score DESC
        LIMIT 100
        ;
        """
        rows: list[tuple[int, str, int, str]]
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql)
                rows = cur.fetchall()  # type: ignore
            con.commit()
        return rows

    # solutionsのうち、scoreがnullのものを返す
    def get_empty_status_entries(self):
        sql = """
        SELECT id, submission_id
        FROM solutions
        WHERE status IS NULL
        LIMIT 50
        ;
        """
        rows: list[tuple[int, str]]
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql)
                rows = cur.fetchall()
            con.commit()
        return rows


    def update_status(self, id: int, status: str, score: int = None):
        if score is None:
            sql = """
                    UPDATE solutions
                    SET status = %s
                    WHERE id = %s
                    ;
                    """
            with scores.con() as con:
                with con.cursor() as cur:
                    cur.execute(sql, (status, id))
                con.commit()
        else:
            sql = """
                    UPDATE solutions
                    SET status = %s, score = %s
                    WHERE id = %s
                    ;
                    """
            with scores.con() as con:
                with con.cursor() as cur:
                    cur.execute(sql, (status, score, id))
                con.commit()

    def upload(self, problem_id: int, submission_id: str, solver: str, content: str):
        # insert into db and get id
        sql = """
        INSERT INTO solutions(solver, problem_id, submission_id) VALUES (%s, %s, %s);
        """
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql, (solver, problem_id, submission_id))
                _ = cur.fetchall()  # type: ignore
                id = cur.lastrowid
            con.commit()
        bucket = storage_client.bucket(self.bucket_name)
        blob = bucket.blob(f"{self.blob_prefix}/{id}.json")
        with blob.open("w") as f:
            f.write(content)  # type: ignore


scores = Scores()


@app.get("/api/hello")
def get_hello():
    return {"message": "Hello, World!"}


@app.get("/api/solutions/show")
def get_scores_show():
    return scores.show()


@app.post("/api/solutions/submit")
def post_submit(id: int, file: UploadFile, solver: str = "unknown"):
    # read uploaded file
    content = file.file.read().decode("utf-8")
    submission_id = submit_solution_to_icfpc(id, content)
    scores.upload(id, submission_id, solver, content)
    return {"submission_id": submission_id}


@app.post("/api/solutions/update_score")
def update_score():
    rows = scores.get_empty_status_entries()
    for id, submission_id in rows:
        resp = get_submission_from_icfpc(submission_id)
        if "Success" not in resp:
            if "Failure" not in resp:
                if resp["Failure"] == "Submission not found!":
                    scores.update_status(id, "submission_not_found")
                else:
                    print("unknown status:", resp)
            continue
        submission = resp["Success"]
        if submission["submission"]["score"] == "Processing":
            if submission["submission"]["contents"] == "":
                scores.update_status(id, "empty_submission")
        elif "Failure" in submission["submission"]["score"]:
            scores.update_status(id, "failed")
        elif "Success" in submission["submission"]["score"]:
            scores.update_status(id, "success", submission["submission"]["score"]["Success"])
        else:
            print("unknown status:", submission["submission"]["score"])

    return {"status": "ok"}
