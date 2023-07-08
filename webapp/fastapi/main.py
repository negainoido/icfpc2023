import json
import os

import pymysql
import requests
from google.cloud import storage
from google.cloud.sql.connector import Connector
from pydantic import BaseModel

from fastapi import FastAPI, UploadFile, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware

secrets = json.loads(os.environ.get("SECRET", ""))
app = FastAPI()
app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://icfpc2023.negainoido.com",
        "https://icfpc2023.negainoido.com",
        "http://localhost",
        "http://localhost:8000",
        "http://localhost:8080",
        "http://localhost:8888",
    ],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)
app.add_middleware(GZipMiddleware, minimum_size=1000000)
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

    return resp.text.replace('"', "")


def get_submission_from_icfpc(submission_id: str):
    id = submission_id.replace('"', "")
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
        """
        problem_id ごとに上位を取ってくる
        """
        sql = """
        SELECT id, problem_id, submission_id, solver, status, score, ts
        FROM (
          SELECT
            id, problem_id, submission_id, solver, status, score, ts,
            ROW_NUMBER() OVER(PARTITION BY problem_id ORDER BY score DESC) as rn
          FROM solutions
        ) t
        WHERE t.rn <= 20
        ;
        """
        rows: list[tuple[int, int, str, str, str, int, str]]
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql)
                rows = cur.fetchall()  # type: ignore
            con.commit()
        return rows

    # solutionsから特定IDのものを返す
    # GCPのbucketからファイルの中身も取ってくる
    def get_solution(self, id: int):
        sql = """
        SELECT id, problem_id, submission_id, solver, status, score, ts
        FROM solutions
        WHERE id = %s
        LIMIT 1
        ;
        """
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql, (id,))
                row = cur.fetchone()
            con.commit()
        if row is None:
            return None

        blob = storage_client.get_bucket(self.bucket_name).get_blob(
            f"{self.blob_prefix}/{row[0]}.json"
        )

        return {
            "id": row[0],
            "problem_id": row[1],
            "submission_id": row[2],
            "solver": row[3],
            "status": row[4],
            "score": row[5],
            "ts": row[6],
            "contents": blob.download_as_string().decode("utf-8"),
        }

    # solutionsのうち、scoreがnullのものを返す
    def get_empty_status_entries(self):
        sql = """
        SELECT id, submission_id
        FROM solutions
        WHERE status IS NULL
        LIMIT 50
        ;
        """
        rows: list[tuple[int, int, str, str, str, int, str]]
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

    # problem_id ごとにscoreが最大のものを取ってくる
    def get_best_solution(self, problem_id: int):
        sql = """
        SELECT id, problem_id, submission_id, solver, status, score, ts
        FROM solutions
        WHERE problem_id = %s AND status = 'success'
        ORDER BY score DESC
        LIMIT 1
        """
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql, (problem_id,))
                row = cur.fetchone()
            con.commit()
        if row is None:
            return None

        blob = storage_client.get_bucket(self.bucket_name).get_blob(
            f"{self.blob_prefix}/{row[0]}.json"
        )

        return {
            "id": row[0],
            "problem_id": row[1],
            "submission_id": row[2],
            "solver": row[3],
            "status": row[4],
            "score": row[5],
            "ts": row[6],
            "contents": blob.download_as_string().decode("utf-8"),
        }


scores = Scores()


@app.get("/api/hello")
def get_hello():
    return {"message": "Hello, World!"}


@app.get("/api/problem")
def get_problem(problem_id: int):
    with open(f"./resource/problems/problem-{problem_id}.json", "rt") as f:
        return Response(content=f.read(), media_type="application/json")


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


@app.get("/api/solutions")
def get_solutions(id: int):
    item = scores.get_solution(id)
    if item is None:
        return {"message": "not found"}
    return item


@app.get("/api/best_solutions")
def get_best_solutions(id: int):
    item = scores.get_best_solution(id)
    if item is None:
        return {"message": "not found"}
    return item


@app.post("/api/solutions/update_score")
def update_score():
    rows = scores.get_empty_status_entries()
    update_count = 0
    processing_count = 0
    error_count = 0
    for id, submission_id in rows:
        resp = get_submission_from_icfpc(submission_id)
        if "Success" not in resp:
            if "Failure" in resp:
                if resp["Failure"] == "Submission not found!":
                    scores.update_status(id, "submission_not_found")
                else:
                    print("unknown status:", resp)
            error_count += 1
            continue
        submission = resp["Success"]
        if submission["submission"]["score"] == "Processing":
            if submission["contents"] == "":
                scores.update_status(id, "empty_submission")
            processing_count += 1
        elif "Failure" in submission["submission"]["score"]:
            scores.update_status(id, "failed")
            update_count += 1
        elif "Success" in submission["submission"]["score"]:
            scores.update_status(
                id, "success", submission["submission"]["score"]["Success"]
            )
            update_count += 1
        else:
            print("unknown status:", submission["submission"]["score"])
            error_count += 1

    return {"updated": update_count, "processing": processing_count, "error": error_count}
