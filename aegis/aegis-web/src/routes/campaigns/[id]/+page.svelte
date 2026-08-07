<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { auth, showToast } from '$lib/stores';
  import { api } from '$lib/api/client';
  import { Play, Pause, Square, Target, TrendingUp, Zap, AlertTriangle, Clock, Activity, Download, Shield } from 'lucide-svelte';

  let campaign: any = null;
  let crashes: any[] = [];
  let loading = true;
  let activeTab = 'overview';
  const campaignId = $page.params.id;

  onMount(async () => {
    try {
      const token = auth.getToken();
      if (!token) return;
      [campaign, crashes] = await Promise.all([
        api.getCampaign(token, campaignId),
        api.listCrashes(token, campaignId)
      ]);
      crashes = crashes.crashes || [];
    } catch (err: any) { showToast('Failed to load campaign', 'error'); }
    finally { loading = false; }
  });

  function formatDuration(startTime: string | null): string {
    if (!startTime) return '-';
    const diff = Date.now() - new Date(startTime).getTime();
    return `${Math.floor(diff / 3600000)}h ${Math.floor((diff % 3600000) / 60000)}m`;
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'running': return 'text-green-400 bg-green-900/30';
      case 'pending': return 'text-yellow-400 bg-yellow-900/30';
      case 'paused': return 'text-blue-400 bg-blue-900/30';
      default: return 'text-gray-400 bg-gray-800';
    }
  }
</script>

<svelte:head><title>{campaign?.name || 'Campaign'} - Aegis</title></svelte:head>

{#if loading}
  <div class="flex items-center justify-center h-64"><div class="w-8 h-8 border-2 border-aegis-500 border-t-transparent rounded-full animate-spin"></div></div>
{:else if !campaign}
  <div class="card text-center py-12"><h3 class="text-lg font-medium text-white">Campaign not found</h3></div>
{:else}
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <div class="flex items-center gap-3 mb-2">
          <h1 class="text-2xl font-bold text-white">{campaign.name}</h1>
          <span class="badge {getStatusColor(campaign.status)}">{campaign.status}</span>
        </div>
        <p class="text-gray-400">Target: {campaign.target_id}</p>
      </div>
      <div class="flex items-center gap-2">
        {#if campaign.status === 'running'}
          <button class="btn-secondary flex items-center gap-2"><Pause class="w-4 h-4" /> Pause</button>
          <button class="btn-secondary flex items-center gap-2 text-red-400"><Square class="w-4 h-4" /> Stop</button>
        {:else if campaign.status === 'paused'}
          <button class="btn-primary flex items-center gap-2"><Play class="w-4 h-4" /> Resume</button>
        {:else if campaign.status === 'pending'}
          <button class="btn-primary flex items-center gap-2"><Play class="w-4 h-4" /> Start</button>
        {/if}
      </div>
    </div>

    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div class="card"><p class="text-sm text-gray-400">Executions</p><p class="text-2xl font-bold text-white">{campaign.total_execs?.toLocaleString() || 0}</p><p class="text-xs text-gray-500">{campaign.execs_per_sec?.toFixed(1) || 0} / sec</p></div>
      <div class="card"><p class="text-sm text-gray-400">Coverage</p><p class="text-2xl font-bold text-white">{campaign.coverage_percent?.toFixed(1) || 0}%</p><p class="text-xs text-gray-500">{campaign.edges_found || 0} edges</p></div>
      <div class="card"><p class="text-sm text-gray-400">Crashes</p><p class="text-2xl font-bold text-white">{campaign.unique_crashes || 0}</p><p class="text-xs text-gray-500">{campaign.total_crashes || 0} total</p></div>
      <div class="card"><p class="text-sm text-gray-400">Duration</p><p class="text-2xl font-bold text-white">{formatDuration(campaign.start_time)}</p><p class="text-xs text-gray-500">{campaign.corpus_size || 0} corpus</p></div>
    </div>

    <div class="border-b border-gray-800">
      <div class="flex gap-6">
        {#each ['overview', 'coverage', 'crashes', 'metrics'] as tab}
          <button on:click={() => activeTab = tab} class="py-3 text-sm font-medium capitalize transition-colors {activeTab === tab ? 'text-aegis-400 border-b-2 border-aegis-400' : 'text-gray-400 hover:text-gray-300'}">{tab}</button>
        {/each}
      </div>
    </div>

    {#if activeTab === 'overview'}
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="card">
          <h3 class="text-lg font-semibold text-white mb-4">Configuration</h3>
          <div class="space-y-3 text-sm">
            <div class="flex justify-between"><span class="text-gray-400">Workers</span><span class="text-white">{campaign.parallel_workers || 1}</span></div>
            <div class="flex justify-between"><span class="text-gray-400">Max Iterations</span><span class="text-white">{campaign.max_iterations || 'Unlimited'}</span></div>
            <div class="flex justify-between"><span class="text-gray-400">Stop on Crash</span><span class="text-white">{campaign.stop_on_first_crash ? 'Yes' : 'No'}</span></div>
          </div>
        </div>
        <div class="card">
          <h3 class="text-lg font-semibold text-white mb-4">Live Activity</h3>
          <div class="flex items-center gap-3 p-3 bg-gray-800/50 rounded-lg">
            <Activity class="w-5 h-5 text-green-500 animate-pulse" />
            <div><p class="text-sm text-white">Campaign is {campaign.status}</p><p class="text-xs text-gray-500">Processing {campaign.execs_per_sec?.toFixed(1) || 0} execs/sec</p></div>
          </div>
        </div>
      </div>
    {:else if activeTab === 'coverage'}
      <div class="card">
        <h3 class="text-lg font-semibold text-white mb-4">Coverage Heatmap</h3>
        <p class="text-sm text-gray-400 mb-4">Edge coverage visualization</p>
        <div class="grid grid-cols-10 gap-1">
          {#each Array(50) as _, i}
            {@const hit = Math.random() > 0.3}
            {@const intensity = hit ? Math.random() : 0}
            <div class="h-8 rounded-sm transition-all hover:scale-110 cursor-pointer" style="background-color: {hit ? `rgba(14, 165, 233, ${intensity})` : 'rgb(31, 41, 55)'}"></div>
          {/each}
        </div>
        <div class="flex items-center gap-4 mt-4 text-xs text-gray-400">
          <div class="flex items-center gap-2"><div class="w-4 h-4 bg-gray-800 rounded"></div><span>Not covered</span></div>
          <div class="flex items-center gap-2"><div class="w-4 h-4 bg-aegis-500 rounded"></div><span>Covered</span></div>
        </div>
      </div>
    {:else if activeTab === 'crashes'}
      <div class="space-y-3">
        {#if crashes.length === 0}
          <div class="card text-center py-12"><Shield class="w-16 h-16 text-gray-600 mx-auto mb-4" /><h3 class="text-lg font-medium text-white">No crashes yet</h3></div>
        {:else}
          {#each crashes as crash}
            <div class="card">
              <div class="flex items-center justify-between">
                <div><h4 class="font-medium text-white">{crash.crash_type}</h4><p class="text-sm text-gray-500">Stack hash: {crash.stack_hash}</p></div>
                <button class="btn-secondary p-2"><Download class="w-4 h-4" /></button>
              </div>
            </div>
          {/each}
        {/if}
      </div>
    {:else if activeTab === 'metrics'}
      <div class="card">
        <h3 class="text-lg font-semibold text-white mb-4">Performance Metrics</h3>
        <p class="text-gray-400">Metrics data will appear here during active campaigns</p>
      </div>
    {/if}
  </div>
{/if}
