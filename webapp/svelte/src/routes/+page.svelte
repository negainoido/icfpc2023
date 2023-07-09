<script>
    import Log from '$lib/Log.svelte';
</script>

<section class="section">
    <div class="container">
        移動しました. <a href="/1"><code>/[problem_id]</code></a> にアクセスしてください.
        <br />
        開発者へ. ビジュアライザ/エディタ本体は `./src/routes/[problem_id]/+page.svelte` です.
    </div>
</section>

<section class="section">
    <div class="container">
        <ul>
            <li><a href="/1">ビジュアライザ/エディタ</a></li>
            <li><a href="https://icfpc2023.negainoido.com/streamlit">点数ボード</a></li>
            <li>
                <a href="https://icfpc2023.negainoido.com/streamlit/problems"
                    >問題が少し詳しく見れる</a
                >
            </li>
        </ul>
    </div>
</section>

<section class="section">
    <div class="container">
        <Log />
    </div>
</section>

<style>
    @import 'https://cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css';
</style>
