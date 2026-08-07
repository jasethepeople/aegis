<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { auth, campaigns, showToast } from '$lib/stores';
  import { api } from '$lib/api/client';
  import { Plus, Play, Pause, Square, Target, TrendingUp, AlertTriangle } from 'lucide-svelte';

  let loading = true;

  onMount(async () => {
    try {
      const token = auth.getToken();
      if (!token) return;
      const res = await api.listCampaigns(token);
      campaigns.set(res.campaigns || []);
    } catch (err: any) { showToast('Failed to load campaigns', 'error'); }
    finally { loading = false; }
  });

  async function startCampaign(id: string) {
    try { await api.startCampaign(auth.getToken()!, id); showToast('Campaign started', 'success'); await refresh(); }
    catch (err: any) { showToast('Failed to start', 'error'); }
  }
  async function pauseCampaign(id: string) {
    try { await api.pauseCampaign(auth.getToken()!, id); showToast('Campaign paused', 'success'); await refresh(); }
    catch (err: any) { showToast('Failed to pause', 'error'); }
  }
  async function stopCampaign(id: string) {
    try { await api.stopCampaign(auth.getToken()!, id); showToast('Campaign stopped', 'success'); await refresh(); }
    catch (err: any) { showToast('Failed to stop', 'error'); }
  }
  async function refresh() {
    const res = await api.listCampaigns(auth.getToken()!);
    campaigns.set(res.campaigns || []);
  }

  function getStatusColor(status: string): string {
    switch (status) {
      case 'running': return 'text-green-400 bg-green-900/30 border-green-800';
      case 'pending': return 'text-yellow-400 bg-yellow-900/30 border-yellow-800';
      case 'paused': return 'text-blue-400 bg-blue-900/30 border-blue-800';
      case 'completed': return 'text-gray-400 bg-gray-800 border-gray-700';
      default: return 'text-gray-400 bg-gray-800 border-gray-700';
    }
  }
</script>

<svelte:head><title>Campaigns - Aegis</title></svelte:head>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold text-white">Campaigns</h1>
      <p class="text-gray-400 mt-1">Manage and monitor fuzzing campaigns</p>
    </div>
    <button on:click={() => goto('/campaigns/new')} class="btn-primary flex items-center gap-2"><Plus class="w-4 h-4" /> New Campaign</button>
  </div>

  {#if loading}
    <div class="flex items-center justify-center h-64"><div class="w-8 h-8 border-2 border-aegis-500 border-t-transparent rounded-full animate-spin"></div></div>
  {:else if $campaigns.length === 0}
    <div class="card text-center py-12">
      <Target class="w-16 h-16 text-gray-600 mx-auto mb-4" />
      <h3 class="text-lg font-medium text-white mb-2">No campaigns found</h3>
      <button on:click={() => goto('/campaigns/new')} class="btn-primary">Create Campaign</button>
    </div>
  {:else}
    <div class="space-y-3">
      {#each $campaigns as campaign}
        <div class="card hover:border-gray-700 transition-colors">
          <div class="flex items-start justify-between">
            <div class="flex-1">
              <div class="flex items-center gap-3 mb-2">
                <h3 class="text-lg font-semibold text-white">{campaign.name}</h3>
                <span class="badge border {getStatusColor(campaign.status)}">{campaign.status}</span>
                {#if campaign.total_crashes > 0}
                  <span class="badge bg-red-900/50 text-red-400"><AlertTriangle class="w-3 h-3 mr-1" /> {campaign.total_crashes} crashes</span>
                {/if}
              </div>
              <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-4">
                <div><p class="text-xs text-gray-500">Executions</p><p class="text-sm font-medium text-white">{campaign.total_execs?.toLocaleString() || 0}</p></div>
                <div><p class="text-xs text-gray-500">Execs/sec</p><p class="text-sm font-medium text-white">{campaign.execs_per_sec?.toFixed(1) || 0}</p></div>
                <div><p class="text-xs text-gray-500">Coverage</p><p class="text-sm font-medium text-white">{campaign.coverage_percent?.toFixed(1) || 0}%</p></div>
                <div><p class="text-xs text-gray-500">Corpus</p><p class="text-sm font-medium text-white">{campaign.corpus_size || 0} entries</p></div>
              </div>
            </div>
            <div class="flex items-center gap-2 ml-4">
              {#if campaign.status === 'pending'}
                <button on:click={() => startCampaign(campaign.id)} class="btn-primary p-2" title="Start"><Play class="w-4 h-4" /></button>
              {:else if campaign.status === 'running'}
                <button on:click={() => pauseCampaign(campaign.id)} class="btn-secondary p-2" title="Pause"><Pause class="w-4 h-4" /></button>
                <button on:click={() => stopCampaign(campaign.id)} class="btn-secondary p-2 text-red-400" title="Stop"><Square class="w-4 h-4" /></button>
              {:else if campaign.status === 'paused'}
                <button on:click={() => startCampaign(campaign.id)} class="btn-primary p-2" title="Resume"><Play class="w-4 h-4" /></button>
                <button on:click={() => stopCampaign(campaign.id)} class="btn-secondary p-2 text-red-400" title="Stop"><Square class="w-4 h-4" /></button>
              {/if}
              <a href="/campaigns/{campaign.id}" class="btn-secondary p-2" title="Details"><TrendingUp class="w-4 h-4" /></a>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
