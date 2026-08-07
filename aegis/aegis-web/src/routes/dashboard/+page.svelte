<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '$lib/stores';
  import { api } from '$lib/api/client';
  import { showToast } from '$lib/stores';
  import { Target, Zap, AlertTriangle, Server, Activity } from 'lucide-svelte';

  let stats = { total_campaigns: 0, active_campaigns: 0, total_workers: 0, active_workers: 0, total_crashes: 0, unique_crashes: 0 };
  let campaigns: any[] = [];
  let loading = true;

  onMount(async () => {
    try {
      const token = auth.getToken();
      if (!token) return;
      const [dashRes, campRes] = await Promise.all([api.getDashboard(token), api.listCampaigns(token)]);
      stats = dashRes;
      campaigns = campRes.campaigns?.filter((c: any) => c.status === 'running').slice(0, 5) || [];
    } catch (err: any) { showToast('Failed to load dashboard', 'error'); }
    finally { loading = false; }
  });

  function formatDuration(startTime: string | null): string {
    if (!startTime) return '-';
    const diff = Date.now() - new Date(startTime).getTime();
    return `${Math.floor(diff / 3600000)}h ${Math.floor((diff % 3600000) / 60000)}m`;
  }
</script>

<svelte:head><title>Dashboard - Aegis</title></svelte:head>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold text-white">Dashboard</h1>
      <p class="text-gray-400 mt-1">Real-time overview of fuzzing operations</p>
    </div>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-64"><div class="w-8 h-8 border-2 border-aegis-500 border-t-transparent rounded-full animate-spin"></div></div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-400">Active Campaigns</p>
            <p class="text-2xl font-bold text-white mt-1">{stats.active_campaigns}</p>
            <p class="text-xs text-gray-500">of {stats.total_campaigns} total</p>
          </div>
          <Target class="w-10 h-10 text-aegis-500" />
        </div>
      </div>
      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-400">Active Workers</p>
            <p class="text-2xl font-bold text-white mt-1">{stats.active_workers}</p>
            <p class="text-xs text-gray-500">of {stats.total_workers} total</p>
          </div>
          <Server class="w-10 h-10 text-aegis-500" />
        </div>
      </div>
      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-400">Unique Crashes</p>
            <p class="text-2xl font-bold text-white mt-1">{stats.unique_crashes}</p>
            <p class="text-xs text-gray-500">{stats.total_crashes} total</p>
          </div>
          <AlertTriangle class="w-10 h-10 text-red-500" />
        </div>
      </div>
      <div class="card">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm text-gray-400">Execs / Second</p>
            <p class="text-2xl font-bold text-white mt-1">0</p>
            <p class="text-xs text-gray-500">across all campaigns</p>
          </div>
          <Zap class="w-10 h-10 text-yellow-500" />
        </div>
      </div>
    </div>

    <div class="card">
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-lg font-semibold text-white flex items-center gap-2"><Target class="w-5 h-5 text-aegis-500" /> Active Campaigns</h2>
        <a href="/campaigns" class="text-sm text-aegis-400 hover:text-aegis-300">View all →</a>
      </div>
      {#if campaigns.length === 0}
        <div class="text-center py-8 text-gray-500"><Activity class="w-12 h-12 mx-auto mb-3 opacity-50" /><p>No active campaigns</p></div>
      {:else}
        <div class="space-y-3">
          {#each campaigns as campaign}
            <div class="flex items-center justify-between p-3 bg-gray-800/50 rounded-lg">
              <div class="flex items-center gap-3">
                <div class="w-2 h-2 rounded-full bg-green-500 animate-pulse"></div>
                <div>
                  <p class="font-medium text-white">{campaign.name}</p>
                  <p class="text-xs text-gray-500">{campaign.execs_per_sec?.toFixed(1) || 0} exec/s | {campaign.coverage_percent?.toFixed(1) || 0}% coverage | {formatDuration(campaign.start_time)}</p>
                </div>
              </div>
              <span class="badge bg-green-900/30 text-green-400">{campaign.status}</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
