import pandas as pd
import numpy as np
from sklearn.manifold import TSNE
import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt

import json
import os


def plot(problem_id: int):
    print("problem_id:", problem_id)
    with open(f"./resource/problems/problem-{problem_id}.json") as f:
        problem = json.load(f)
    df = pd.DataFrame(problem["attendees"])
    data = df.tastes.tolist()
    matrix = np.array(data)
    tsne = TSNE(
        n_components=2, perplexity=15, random_state=42, init="random", learning_rate=200
    )
    vis_dims = tsne.fit_transform(matrix)
    x = [x for x, y in vis_dims]
    y = [y for x, y in vis_dims]
    plt.figure()
    plt.scatter(x, y, alpha=0.3)
    plt.savefig(f"./resource/img/tastes-{problem_id}.png")


if __name__ == "__main__":
    for i in range(1, 91):
        plot(i)
