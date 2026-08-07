<script lang="ts">
  import { onMount } from 'svelte';
  import { auth, showToast } from '$lib/stores';
  import { api } from '$lib/api/client';
  import { Server } from 'lucide-svelte';

  let workers: any[] = [];
  let loading = true;

  onMount(async () => {
    try {
      const token = auth.getToken();
      if (!token) return;
      const res = await api.listWorkers(token);
      workers = res.workers || [];
    } catch (err: any) { showToast('Failed to load workers', 'error'); }
    finally { loading = false; }
  });

  function getStatusColor(status: string): string {
    switch (status) {
      case 'online': return 'text-green-400';
      case 'busy': return 'text-yellow-400';
      case 'offline': return 'text-red-400';
      default: return 'text-gray-400';
    }
  }
</script>

<svelte:head><title>Workers - Aegis</title></svelte:head>

<div class="space-y-6">
  <div>
    <h1 class="text-2xl font-bold text-white">Workers</h1>
    <p class="text-gray-400 mt-1">Fuzzing worker nodes</p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-64"><div class="w-8 h-8 border-2 border-aegis-500 border-t-transparent rounded-full animate-spin"></div></div>
  {:else if workers.length === 0}
    <div class="card text-center py-12">
      <Server class="w-16 h-16 text-gray-600 mx-auto mb-4" />
      <h3 class="text-lg font-medium text-white">No workers</h3>
      <p class="text-gray-400">Start a worker to begin fuzzing</p>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each workers as worker}
        <div class="card">
          <div class="flex items-center justify-between mb-3">
            <h3 class="font-semibold text-white">{worker.hostname}</h3>
            <div class="flex items-center gap-2">
              <div class="w-2 h-2 rounded-full {worker.status === 'online' ? 'bg-green-500' : worker.status === 'busy' ? 'bg-yellow-500' : 'bg-red-500'}"></div>
              <span class="text-sm {getStatusColor(worker.status)}">{worker.status}</span>
            </div>
          </div>
          <p class="text-xs text-gray-500">{worker.address}</p>
          {#if worker.current_campaign}
            <div class="mt-3 p-2 bg-aegis-900/20 rounded">
              <p class="text-xs text-aegis-400">Active on campaign: {worker.current_campaign}</p>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
