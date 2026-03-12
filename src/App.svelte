<script lang="ts">
  import FileTree from '$lib/components/FileTree.svelte';
  import Editor from '$lib/components/Editor.svelte';
  import AIPanel from '$lib/components/AIPanel.svelte';
  import TabBar from '$lib/components/TabBar.svelte';
  import Settings from '$lib/components/Settings.svelte';
  let settingsOpen = false;
</script>

<div class="app-shell">
  <header class="titlebar">Quillpaw</header>
  <aside class="left"><FileTree /></aside>
  <main>
    <TabBar />
    <Editor />
  </main>
  <aside class="right"><AIPanel /></aside>
  <footer class="status">Your notes. Your machine. Your den.</footer>
  <Settings bind:open={settingsOpen} />
</div>

<style>
  .app-shell { display:grid; grid-template-columns:280px 1fr 320px; grid-template-rows:36px 1fr 28px; height:100vh; }
  .titlebar { grid-column:1/4; background:var(--bg-panel); border-bottom:1px solid var(--border); display:flex; align-items:center; padding:0 var(--space-4); }
  .left, .right { background:var(--bg-panel); border-right:1px solid var(--border-subtle); overflow:auto; }
  .right { border-right:0; border-left:1px solid var(--border-subtle); }
  main { overflow:auto; }
  .status { grid-column:1/4; background:var(--bg-panel); border-top:1px solid var(--border); display:flex; align-items:center; padding:0 var(--space-4); color:var(--text-secondary); }
</style>
