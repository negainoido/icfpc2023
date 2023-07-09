FROM python:3.11-slim

WORKDIR /app
COPY requirements.txt requirements.txt
RUN pip install -U pip wheel \
    && pip install -r requirements.txt
ENV ENV=prod

COPY . .
CMD ["streamlit", "run", "app.py", "--server.port", "8080", "--server.address", "0.0.0.0"]
