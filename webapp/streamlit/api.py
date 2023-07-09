from io import StringIO
import os

import requests

from streamlit.runtime.uploaded_file_manager import UploadedFile
from streamlit.logger import get_logger

ENV = os.getenv("ENV") or "dev"
logger = get_logger(__name__)


class API:
    def __init__(self):
        if ENV == "dev":
            # ローカルで立てる時にはCloud Run Proxyを立ててください。
            # gcloud beta run services proxy fastapi-iam-auth --port=8080 --region asia-northeast1
            self.url = "http://localhost:8080"
        else:
            self.url = "https://fastapi-f4mnmafhja-an.a.run.app"

    def _get(self, endpoint: str, data=None, params=None):
        headers = {
            "Content-Type": "application/json",
        }
        response = requests.get(
            f"{self.url}{endpoint}", data=data, headers=headers, params=params
        )
        if response.status_code != 200:
            logger.error(
                "response is not ok: %s %s %s",
                self.url + endpoint,
                params,
                response.status_code,
            )
            return None
        return response.json()

    def show(self):
        data = self._get("/api/solutions/show")
        return data

    def solution(self, solution_id: int):
        data = self._get("/api/solutions", params={"id": solution_id})
        return data

    def submit(self, problem_id: int, solver: str, uploaded_file: UploadedFile):
        stringio = StringIO(uploaded_file.getvalue().decode("utf-8"))
        response = requests.post(
            f"{self.url}/api/solutions/submit?id={problem_id}&solver={solver}",
            files={"file": stringio},
        )
        return response.json()

    def update_score(self):
        response = requests.post(
            f"{self.url}/api/solutions/update_score",
        )
        return response.json()
