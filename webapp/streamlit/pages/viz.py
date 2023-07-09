import streamlit as st

NUM_PROBLEM = 90
query_params = st.experimental_get_query_params()


st.info(
    f"""
:building_construction:
[/svelte](https://icfpc2023.negainoido.com/{int(query_params.get('id',[1])[0])})
でちゃんと作り直した
"""
)
