import { Instance } from '../types'

export interface ServerType {
  id: string;
  name: string;
  description: string;
  category: 'Playable Server' | 'Network Proxy' | 'Other';
  icon: React.ReactNode;
  badge?: string;
  badgeColor?: string;
}

export interface MCVersion {
  id: string;
  type: string;
  url: string;
  releaseTime: string;
}

export interface VersionManifest {
  latest: {
    release: string;
    snapshot: string;
  };
  versions: MCVersion[];
}

export interface ModLoader {
  name: string;
  versions: string[];
}

export interface CreateInstanceModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreated: (instance: Instance) => void;
}

export type Tab = 'custom' | 'import' | 'modrinth' | 'curseforge';
