import json
import os
import secrets as sec

import pymysql
from fastapi import FastAPI, HTTPException, Security
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from pydantic import BaseModel
from google.cloud.sql.connector import Connector

secrets = json.loads(os.environ.get("SECRET", ""))
app = FastAPI()
connector = Connector()


def check(auth: str):
    if sec.compare_digest(auth, secrets["auth"]) is not True:
        raise HTTPException(status_code=401)


class Scores:
    def __init__(self):
        self.con_type = secrets["database"]["type"]
        if self.con_type == "cloudsql":
            self.name = secrets["database"]["name"]
        elif self.con_type == "tcp":
            self.host = secrets["database"]["host"]

        self.user = secrets["database"]["user"]
        self.password = secrets["database"]["password"]
        self.database = secrets["database"]["database"]
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
                user=self.user,
                password=self.password,
                database=self.database,
            )
        else:
            raise Exception('invalid db_type:', self.db_type)

    def _init_table(self):
        sql = """
        CREATE TABLE IF NOT EXISTS scores(
            id INT(11) AUTO_INCREMENT NOT NULL PRIMARY KEY, 
            name VARCHAR(30) NOT NULL,
            score INT NOT NULL,
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
        FROM scores
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

    def append(self, name: str, score: int):
        sql = """
        INSERT INTO scores(name, score) VALUES (%s, %s);
        """
        with self.con() as con:
            with con.cursor() as cur:
                cur.execute(sql, (name, score))
                rows = cur.fetchall()  # type: ignore
            con.commit()
        return rows


scores = Scores()


@app.get("/api/hello")
def get_hello():
    return {"message": "Hello, World!"}


@app.get("/api/scores/show")
def get_scores_show():
    return scores.show()


class ScoresAppend(BaseModel):
    name: str
    score: int


@app.post("/api/scores/append")
def post_scores_append(
    item: ScoresAppend, auth: HTTPAuthorizationCredentials = Security(HTTPBearer())
):
    check(auth.credentials)
    scores.append(item.name, item.score)
    return {"status": "OK"}
