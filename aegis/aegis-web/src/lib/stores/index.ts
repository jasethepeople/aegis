import { writable, derived } from 'svelte/store';
import { browser } from '$app/environment';

interface User { id: string; email: string; name: string; role: string; }

function createAuthStore() {
  const { subscribe, set } = writable<User | null>(null);
  return {
    subscribe,
    login: (user: User, token: string) => {
      if (browser) { localStorage.setItem('aegis_token', token); localStorage.setItem('aegis_user', JSON.stringify(user)); }
      set(user);
    },
    logout: () => {
      if (browser) { localStorage.removeItem('aegis_token'); localStorage.removeItem('aegis_user'); }
      set(null);
    },
    init: () => {
      if (browser) {
        const token = localStorage.getItem('aegis_token');
        const user = localStorage.getItem('aegis_user');
        if (token && user) set(JSON.parse(user));
      }
    },
    getToken: () => browser ? localStorage.getItem('aegis_token') : null
  };
}

export const auth = createAuthStore();
export const isAuthenticated = derived(auth, $auth => $auth !== null);

export const campaigns = writable<any[]>([]);
export const toast = writable<{id: string; message: string; type: string} | null>(null);

export function showToast(message: string, type: string = 'info') {
  const id = Math.random().toString(36).substring(2);
  toast.set({ id, message, type });
  setTimeout(() => toast.set(null), 5000);
}
