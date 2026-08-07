const API_BASE = '/api/v1';

async function request(method: string, path: string, body?: any, token?: string) {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  const res = await fetch(`${API_BASE}${path}`, { method, headers, body: body ? JSON.stringify(body) : undefined });
  if (!res.ok) { const data = await res.json().catch(() => ({})); throw new Error(data.error || `HTTP ${res.status}`); }
  return res.status === 204 ? null : res.json();
}

export const api = {
  login: (email: string, password: string) => request('POST', '/auth/login', { email, password }),
  listCampaigns: (token: string) => request('GET', '/campaigns', undefined, token),
  createCampaign: (token: string, data: any) => request('POST', '/campaigns', data, token),
  getCampaign: (token: string, id: string) => request('GET', `/campaigns/${id}`, undefined, token),
  startCampaign: (token: string, id: string) => request('POST', `/campaigns/${id}/start`, undefined, token),
  pauseCampaign: (token: string, id: string) => request('POST', `/campaigns/${id}/pause`, undefined, token),
  stopCampaign: (token: string, id: string) => request('POST', `/campaigns/${id}/stop`, undefined, token),
  listTargets: (token: string) => request('GET', '/targets', undefined, token),
  createTarget: (token: string, data: any) => request('POST', '/targets', data, token),
  listWorkers: (token: string) => request('GET', '/workers', undefined, token),
  listCrashes: (token: string, campaignId?: string) => request('GET', `/crashes${campaignId ? `?campaign_id=${campaignId}` : ''}`, undefined, token),
  getDashboard: (token: string) => request('GET', '/dashboard', undefined, token),
};
