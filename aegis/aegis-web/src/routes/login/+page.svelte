<script lang="ts">
  import { goto } from '$app/navigation';
  import { auth, showToast } from '$lib/stores';
  import { api } from '$lib/api/client';
  import { Shield, Mail, Lock, Eye, EyeOff } from 'lucide-svelte';

  let email = '', password = '', showPassword = false, loading = false;

  async function handleLogin() {
    loading = true;
    try {
      const res = await api.login(email, password);
      auth.login({ id: '1', email, name: email.split('@')[0], role: 'admin' }, res.token);
      showToast('Welcome back!', 'success');
      goto('/dashboard');
    } catch (err: any) {
      showToast(err.message || 'Login failed', 'error');
    } finally { loading = false; }
  }
</script>

<div class="min-h-screen flex items-center justify-center bg-gray-950 px-4">
  <div class="w-full max-w-md">
    <div class="text-center mb-8">
      <Shield class="w-16 h-16 text-aegis-500 mx-auto mb-4" />
      <h1 class="text-3xl font-bold text-white mb-2">Aegis</h1>
      <p class="text-gray-400">Production-grade fuzzing platform</p>
    </div>
    <div class="card">
      <h2 class="text-xl font-semibold text-white mb-6">Sign In</h2>
      <form on:submit|preventDefault={handleLogin} class="space-y-4">
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-1">Email</label>
          <div class="relative">
            <Mail class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
            <input type="email" bind:value={email} class="input pl-10" placeholder="you@example.com" required />
          </div>
        </div>
        <div>
          <label class="block text-sm font-medium text-gray-300 mb-1">Password</label>
          <div class="relative">
            <Lock class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
            <input type={showPassword ? 'text' : 'password'} bind:value={password} class="input pl-10 pr-10" placeholder="••••••••" required />
            <button type="button" on:click={() => showPassword = !showPassword} class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500">
              {#if showPassword}<EyeOff class="w-5 h-5" />{:else}<Eye class="w-5 h-5" />{/if}
            </button>
          </div>
        </div>
        <button type="submit" class="btn-primary w-full flex items-center justify-center gap-2" disabled={loading}>
          {#if loading}<div class="w-5 h-5 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>{:else}Sign In{/if}
        </button>
      </form>
    </div>
  </div>
</div>
