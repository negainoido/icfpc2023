import numpy
from plotly import graph_objects

import streamlit

streamlit.write("# pages sample")
a = float(streamlit.slider("a", value=1.0, min_value=0.1, max_value=10.0))
b = float(streamlit.slider("b", value=1.0, min_value=0.1, max_value=10.0))
c = float(streamlit.slider("c", value=1.0, min_value=0.1, max_value=10.0))
d = float(streamlit.slider("d", value=1.0, min_value=0.1, max_value=10.0))
X, Y, Z = numpy.mgrid[-1:1:20j, -1:1:20j, -1:1:20j]
isovalue = -((X / a) ** 2 + (Y / b) ** 2 + (Z / c) ** 2 - d)

fig = graph_objects.Figure(
    data=graph_objects.Isosurface(
        x=X.flatten(),
        y=Y.flatten(),
        z=Z.flatten(),
        value=isovalue.flatten(),
        isomin=0.0,
        isomax=1.0,
        showscale=False,
    )
)
streamlit.write(fig)
