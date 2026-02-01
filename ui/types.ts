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

export type TabId = 'dashboard' | 'console' | 'logs' | 'plugins' | 'mods' | 'players' | 'config' | 'backups' | 'scheduler' | 'settings';

export interface PlayerEntry {
  uuid: string;
  name: string;
}

export interface OpEntry {
  uuid: string;
  name: string;
  level: number;
  bypassesPlayerLimit: boolean;
}

export interface BannedPlayerEntry {
  uuid: string;
  name: string;
  created: string;
  source: string;
  expires: string;
  reason: string;
}

export interface BannedIpEntry {
  ip: string;
  created: string;
  source: string;
  expires: string;
  reason: string;
}

export interface UserCacheEntry {
  uuid: string;
  name: string;
  expiresOn: string;
}

export interface AllPlayerLists {
  whitelist: PlayerEntry[];
  ops: OpEntry[];
  banned_players: BannedPlayerEntry[];
  banned_ips: BannedIpEntry[];
  user_cache: UserCacheEntry[];
}

export interface BackupInfo {
  name: string;
  path: string;
  size: number;
  created_at: string;
}
