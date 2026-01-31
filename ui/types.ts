export interface Instance {
  id: string;
  name: string;
  version: string;
  path: string;
  created_at: string;
  last_run?: string;
  mod_loader?: string;
  loader_version?: string;
  server_type?: string;
  ip?: string;
  port?: number;
  description?: string;
  max_players?: number;
  status?: string;
}

export interface ResourceUsage {
  cpu_usage: number;
  memory_usage: number;
  timestamp?: number;
}

export type TabId = 'dashboard' | 'console' | 'logs' | 'plugins' | 'mods' | 'players' | 'backups' | 'scheduler' | 'settings';
