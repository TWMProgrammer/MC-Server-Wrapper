import { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Instance } from '../types'
import { VersionManifest, ModLoader, Tab } from './types'

export function useCreateInstance(isOpen: boolean, onCreated: (instance: Instance) => void, onClose: () => void) {
  const [activeTab, setActiveTab] = useState<Tab>('custom');
  const [selectedServerType, setSelectedServerType] = useState<string | null>(null);
  const [manifest, setManifest] = useState<VersionManifest | null>(null);
  const [loading, setLoading] = useState(true);
  const [loadingModLoaders, setLoadingModLoaders] = useState(false);
  const [search, setSearch] = useState('');
  const [showSnapshots, setShowSnapshots] = useState(false);
  const [name, setName] = useState('');
  const [selectedVersion, setSelectedVersion] = useState<string | null>(null);
  const [modLoaders, setModLoaders] = useState<ModLoader[]>([]);
  const [selectedLoader, setSelectedLoader] = useState<string>('none');
  const [selectedLoaderVersion, setSelectedLoaderVersion] = useState<string | null>(null);
  const [creating, setCreating] = useState(false);

  const [importSourcePath, setImportSourcePath] = useState<string | null>(null);
  const [importServerType, setImportServerType] = useState<string>('vanilla');
  const [availableJars, setAvailableJars] = useState<string[]>([]);
  const [selectedJar, setSelectedJar] = useState<string | null>(null);
  const [serverPropertiesExists, setServerPropertiesExists] = useState<boolean>(true);
  const [rootWithinZip, setRootWithinZip] = useState<string | null>(null);
  const [importProgress, setImportProgress] = useState<{ current: number, total: number, message: string } | null>(null);

  const resetForm = () => {
    setActiveTab('custom');
    setSelectedServerType(null);
    setSearch('');
    setName('');
    setShowSnapshots(false);
    setSelectedVersion(null);
    setSelectedLoader('none');
    setSelectedLoaderVersion(null);
    setImportSourcePath(null);
    setImportServerType('vanilla');
    setAvailableJars([]);
    setSelectedJar(null);
    setServerPropertiesExists(true);
    setRootWithinZip(null);
    setImportProgress(null);
  };

  useEffect(() => {
    let unlisten: any;
    
    const setupListener = async () => {
      unlisten = await listen<{ current: number, total: number, message: string }>('import-progress', (event) => {
        setImportProgress(event.payload);
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    if (isOpen) {
      loadVersions();
    }
  }, [isOpen]);

  useEffect(() => {
    if (selectedVersion) {
      loadModLoaders(selectedVersion);
    } else {
      setModLoaders([]);
      setSelectedLoader('none');
      setSelectedLoaderVersion(null);
    }
  }, [selectedVersion]);

  useEffect(() => {
    if (selectedServerType === 'forge') {
      setSelectedLoader('forge');
    } else if (selectedServerType === 'fabric') {
      setSelectedLoader('fabric');
    } else if (selectedServerType === 'vanilla') {
      setSelectedLoader('none');
    } else if (selectedServerType) {
      setSelectedLoader(selectedServerType);
    } else {
      setSelectedLoader('none');
    }
  }, [selectedServerType]);

  async function loadVersions() {
    try {
      setLoading(true);
      const m = await invoke<VersionManifest>('get_minecraft_versions');
      setManifest(m);
    } catch (e) {
      console.error('Failed to load versions', e);
    } finally {
      setLoading(false);
    }
  }

  async function loadModLoaders(version: string) {
    const isModded = ['forge', 'fabric', 'neoforge', 'paper', 'purpur'].includes(selectedServerType || '');
    if (!isModded) {
      setModLoaders([]);
      return;
    }

    try {
      setLoadingModLoaders(true);
      const loaders = await invoke<ModLoader[]>('get_mod_loaders', { mcVersion: version });
      setModLoaders(loaders);
      const currentLoader = loaders.find(l => l.name.toLowerCase() === (selectedServerType?.toLowerCase()));
      if (currentLoader && currentLoader.versions.length > 0) {
        setSelectedLoaderVersion(currentLoader.versions[0]);
      }
    } catch (e) {
      console.error('Failed to load mod loaders', e);
    } finally {
      setLoadingModLoaders(false);
    }
  }

  const filteredVersions = useMemo(() => {
    if (!manifest) return [];
    return manifest.versions.filter(v => {
      const matchesSearch = v.id.toLowerCase().includes(search.toLowerCase());
      const isRelease = v.type === 'release';
      const isSnapshot = v.type === 'snapshot';

      if (!matchesSearch) return false;
      if (isRelease) return true;
      if (isSnapshot && showSnapshots) return true;
      return false;
    });
  }, [manifest, search, showSnapshots]);

  async function handleCreate() {
    if (activeTab === 'import') {
      return handleImport();
    }
    if (!name || !selectedVersion) return;

    try {
      setCreating(true);
      const instance = await invoke<Instance>('create_instance_full', {
        name,
        version: selectedVersion,
        modLoader: selectedLoader === 'none' ? null : selectedLoader,
        loaderVersion: selectedLoaderVersion,
      });
      onCreated(instance);
      resetForm();
      onClose();
    } catch (e) {
      console.error('Failed to create instance', e);
    } finally {
      setCreating(false);
    }
  }

  async function handleImport() {
    if (!name || !importSourcePath || !selectedJar) return;

    try {
      setCreating(true);
      const instance = await invoke<Instance>('import_instance', {
        name,
        sourcePath: importSourcePath,
        jarName: selectedJar,
        serverType: importServerType,
        rootWithinZip,
      });
      onCreated(instance);
      resetForm();
      onClose();
    } catch (e) {
      console.error('Failed to import instance', e);
    } finally {
      setCreating(false);
    }
  }

  return {
    activeTab,
    setActiveTab,
    selectedServerType,
    setSelectedServerType,
    loading,
    loadingModLoaders,
    search,
    setSearch,
    showSnapshots,
    setShowSnapshots,
    name,
    setName,
    selectedVersion,
    setSelectedVersion,
    modLoaders,
    selectedLoaderVersion,
    setSelectedLoaderVersion,
    creating,
    handleCreate,
    filteredVersions,
    resetForm,
    importSourcePath,
    setImportSourcePath,
    importServerType,
    setImportServerType,
    availableJars,
    setAvailableJars,
    selectedJar,
    setSelectedJar,
    serverPropertiesExists,
    setServerPropertiesExists,
    rootWithinZip,
    setRootWithinZip,
    importProgress
  };
}
