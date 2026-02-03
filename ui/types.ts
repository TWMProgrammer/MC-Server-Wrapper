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
  settings: InstanceSettings;
}

export interface InstanceSettings {
  description?: string;
  ram: number;
  ram_unit: string;
  port: number;
  force_save_all: boolean;
  autostart: boolean;
  java_path_override?: string;
  launch_method: LaunchMethod;
  startup_line: string;
  bat_file?: string;
  crash_handling: CrashHandlingMode;
  icon_path?: string;
}

export type LaunchMethod = 'StartupLine' | 'BatFile';

export type CrashHandlingMode = 'Nothing' | 'Elevated' | 'Aggressive';

export interface ResourceUsage {
  cpu_usage: number;
  memory_usage: number;
  timestamp?: number;
}

export type TabId = 'dashboard' | 'console' | 'logs' | 'plugins' | 'mods' | 'players' | 'config' | 'backups' | 'scheduler' | 'settings';

export type TransitionType = 'starting' | 'stopping' | 'restarting';

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
  status?: 'creating' | 'ready' | 'error';
  progress?: number;
}

export type ScheduleType = 'Backup' | 'Restart';

export interface ScheduledTask {
  id: string;
  instance_id: string;
  task_type: ScheduleType;
  cron: string;
  enabled: boolean;
  last_run?: string;
  next_run?: string;
}

export interface InstalledPlugin {
  name: string;
  filename: string;
  enabled: boolean;
  version?: string;
  author?: string;
  description?: string;
  source?: PluginSource;
}

export interface PluginSource {
  project_id: string;
  provider: PluginProvider;
  current_version_id?: string;
}

export interface PluginUpdate {
  filename: string;
  current_version?: string;
  latest_version: string;
  latest_version_id: string;
  project_id: string;
  provider: PluginProvider;
}

export type PluginProvider = 'Modrinth' | 'Spiget';

export type ModProvider = 'Modrinth' | 'CurseForge';

export type SortOrder = 'Relevance' | 'Downloads' | 'Follows' | 'Newest' | 'Updated';

export interface SearchOptions {
  query: string;
  facets?: string[];
  sort?: SortOrder;
  offset?: number;
  limit?: number;
  game_version?: string;
  loader?: string;
}

export interface Project {
  id: string;
  slug: string;
  title: string;
  description: string;
  downloads: number;
  icon_url?: string;
  author: string;
  provider: PluginProvider | ModProvider;
  categories?: string[];
}

export interface ResolvedDependency {
  project: Project;
  dependency_type: string;
}

export interface InstalledMod {
  name: string;
  filename: string;
  enabled: boolean;
  version?: string;
  author?: string;
  description?: string;
  loader?: string;
  source?: ModSource;
  icon_data?: string;
}

export interface ModSource {
  project_id: string;
  provider: ModProvider;
  current_version_id?: string;
}

export interface ModConfig {
  name: string;
  path: string;
  is_dir: boolean;
}

export interface ModUpdate {
  filename: string;
  current_version?: string;
  latest_version: string;
  latest_version_id: string;
  project_id: string;
  provider: ModProvider;
}
