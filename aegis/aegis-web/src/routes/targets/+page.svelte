<script lang="ts">
  import { onMount } from 'svelte';
  import { auth, showToast } from '$lib/stores';
  import { api } from '$lib/api/client';
  import { Plus, Target } from 'lucide-svelte';

  let targets: any[] = [];
  let loading = true;

  onMount(async () => {
    try {
      const token = auth.getToken();
      if (!token) return;
      const res = await api.listTargets(token);
      targets = res.targets || [];
    } catch (err: any) { showToast('Failed to load targets', 'error'); }
    finally { loading = false; }
  });
</script>

<svelte:head><title>Targets - Aegis</title></svelte:head>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold text-white">Targets</h1>
      <p class="text-gray-400 mt-1">Manage fuzzing targets</p>
    </div>
    <button class="btn-primary flex items-center gap-2"><Plus class="w-4 h-4" /> New Target</button>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-64"><div class="w-8 h-8 border-2 border-aegis-500 border-t-transparent rounded-full animate-spin"></div></div>
  {:else if targets.length === 0}
    <div class="card text-center py-12">
      <Target class="w-16 h-16 text-gray-600 mx-auto mb-4" />
      <h3 class="text-lg font-medium text-white">No targets</h3>
      <p class="text-gray-400">Create a target to start fuzzing</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each targets as target}
        <div class="card">
          <h3 class="font-semibold text-white">{target.name}</h3>
          <p class="text-sm text-gray-400 mt-1">{target.type}</p>
          <p class="text-xs text-gray-500 mt-2 font-mono">{target.command}</p>
          <div class="flex items-center gap-2 mt-4">
            {#if target.instrumentation?.use_sancov}<span class="badge bg-aegis-900/30 text-aegis-400 text-xs">SanCov</span>{/if}
            {#if target.instrumentation?.use_asan}<span class="badge bg-red-900/30 text-red-400 text-xs">ASan</span>{/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
