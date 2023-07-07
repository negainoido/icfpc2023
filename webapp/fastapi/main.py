import json
import requests
import os
import secrets as sec

import pymysql
from fastapi import FastAPI, HTTPException, UploadFile
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from pydantic import BaseModel
from google.cloud.sql.connector import Connector
from google.cloud import storage

secrets = json.loads(os.environ.get("SECRET", ""))
app = FastAPI()
connector = Connector()
storage_client = storage.Client()

def get_icfpc_token():
    data={ "username_or_email": secrets["icfpc"]["username"], "password": secrets["icfpc"]["password"] }
    resp = requests.post("https://api.icfpcontest.com/login", data=json.dumps(data), headers={"Content-Type": "application/json"})
    json_resp = resp.json()
    return json_resp["Success"]

icfpc_token = get_icfpc_token()

def submit_solution_to_icfpc(problem_id: int, contents: str):
    data = { "problem_id": problem_id, "contents": contents }
    resp = requests.post("https://api.icfpcontest.com/submission", data=json.dumps(data), headers={"Content-Type": "application/json", "Authorization": f"Bearer {icfpc_token}"})
    return resp.text

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
        print(secrets["database"])
        self._init_table()

    def con(self):
        if self.con_type == "cloudsql":
            return connector.connect(
                self.name,
                "pymysql",
                user=self.user,
                password=self.password,
                db=self.database
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
            raise Exception('invalid db_type:', self.db_type)

    def _init_table(self):
        sql = """
        CREATE TABLE IF NOT EXISTS solutions(
            id INT(11) AUTO_INCREMENT NOT NULL PRIMARY KEY, 
            problem_id INT(11) NOT NULL,
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
        SELECT *
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

    def upload(self, problem_id: int, submission_id: str, solver: str, content: str):
        # insert into db and get id
        sql = """
        INSERT INTO solutions(solver, problem_id, submission_id) VALUES (%s, %s, %s);
        """
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql, (solver, problem_id, submission_id))
                rows = cur.fetchall()  # type: ignore
                id = cur.lastrowid
            con.commit()
        bucket = storage_client.bucket(self.bucket_name)
        blob = bucket.blob(f'{self.blob_prefix}/{id}.json')
        with blob.open('w') as f:
            f.write(content)


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
    return {"status": "OK"}
