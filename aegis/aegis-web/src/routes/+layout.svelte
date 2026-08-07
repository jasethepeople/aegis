<script lang="ts">
  import '../app.css';
  import { auth, isAuthenticated } from '$lib/stores';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { Menu, X, Shield, LogOut, User, LayoutDashboard, Target, Crosshair, AlertTriangle, Server } from 'lucide-svelte';

  let sidebarOpen = false;
  onMount(() => auth.init());
  $: if (!$isAuthenticated && $page.url.pathname !== '/login') goto('/login');

  const navItems = [
    { path: '/dashboard', label: 'Dashboard', icon: LayoutDashboard },
    { path: '/campaigns', label: 'Campaigns', icon: Target },
    { path: '/targets', label: 'Targets', icon: Crosshair },
    { path: '/crashes', label: 'Crashes', icon: AlertTriangle },
    { path: '/workers', label: 'Workers', icon: Server }
  ];
</script>

{#if $isAuthenticated}
<div class="flex h-screen bg-gray-950">
  <aside class="{sidebarOpen ? 'translate-x-0' : '-translate-x-full'} fixed inset-y-0 left-0 z-50 w-64 bg-gray-900 border-r border-gray-800 transition-transform lg:translate-x-0 lg:static">
    <div class="flex items-center justify-between p-4 border-b border-gray-800">
      <div class="flex items-center gap-3">
        <Shield class="w-8 h-8 text-aegis-500" />
        <span class="text-xl font-bold text-white">Aegis</span>
      </div>
      <button on:click={() => sidebarOpen = !sidebarOpen} class="lg:hidden text-gray-400"><X class="w-6 h-6" /></button>
    </div>
    <nav class="p-4 space-y-1">
      {#each navItems as item}
        <a href={item.path} class="flex items-center gap-3 px-4 py-3 rounded-lg text-sm font-medium transition-colors {$page.url.pathname.startsWith(item.path) ? 'bg-aegis-900/50 text-aegis-400' : 'text-gray-400 hover:bg-gray-800 hover:text-gray-200'}">
          <svelte:component this={item.icon} class="w-5 h-5" />
          {item.label}
        </a>
      {/each}
    </nav>
    <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-gray-800">
      <div class="flex items-center gap-3 mb-3">
        <div class="w-8 h-8 rounded-full bg-aegis-700 flex items-center justify-center"><User class="w-4 h-4 text-white" /></div>
        <div class="flex-1 min-w-0">
          <p class="text-sm font-medium text-white truncate">{$auth?.name || 'User'}</p>
          <p class="text-xs text-gray-500 truncate">{$auth?.email || ''}</p>
        </div>
      </div>
      <button on:click={() => { auth.logout(); goto('/login'); }} class="flex items-center gap-2 w-full px-4 py-2 text-sm text-gray-400 hover:text-red-400 hover:bg-red-900/20 rounded-lg transition-colors">
        <LogOut class="w-4 h-4" /> Logout
      </button>
    </div>
  </aside>
  <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
    <header class="flex items-center justify-between px-6 py-4 bg-gray-900 border-b border-gray-800">
      <button on:click={() => sidebarOpen = !sidebarOpen} class="lg:hidden text-gray-400"><Menu class="w-6 h-6" /></button>
      <h1 class="text-lg font-semibold text-white">{$page.data?.title || 'Aegis'}</h1>
      <span class="text-xs text-gray-500">v0.4.0</span>
    </header>
    <main class="flex-1 overflow-y-auto p-6"><slot /></main>
  </div>
</div>
{:else}
<slot />
{/if}
