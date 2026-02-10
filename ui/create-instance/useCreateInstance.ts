import { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Instance, Project, ProjectVersion, ModpackProgress } from '../types'
import { VersionManifest, ModLoader, Tab } from './types'
import { useToast } from '../hooks/useToast'
import { useDebounce } from '../hooks/useDebounce'

export function useCreateInstance(isOpen: boolean, onCreated: (instance: Instance) => void, onClose: () => void) {
  const { showToast } = useToast();
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
  const [error, setError] = useState<string | null>(null);
  const [nameExists, setNameExists] = useState(false);
  const [startAfterCreation, setStartAfterCreation] = useState(false);

  const [importSourcePath, setImportSourcePath] = useState<string | null>(null);
  const [importServerType, setImportServerType] = useState<string>('vanilla');
  const [availableJars, setAvailableJars] = useState<string[]>([]);
  const [selectedJar, setSelectedJar] = useState<string | null>(null);
  const [availableScripts, setAvailableScripts] = useState<string[]>([]);
  const [selectedScript, setSelectedScript] = useState<string | null>(null);
  const [serverPropertiesExists, setServerPropertiesExists] = useState<boolean>(true);
  const [bypassServerPropertiesCheck, setBypassServerPropertiesCheck] = useState<boolean>(false);
  const [rootWithinZip, setRootWithinZip] = useState<string | null>(null);
  const [importProgress, setImportProgress] = useState<{ current: number, total: number, message: string } | null>(null);

  const [modpackResults, setModpackResults] = useState<Project[]>([]);
  const [searchingModpacks, setSearchingModpacks] = useState(false);
  const [selectedModpack, setSelectedModpack] = useState<Project | null>(null);
  const [modpackVersions, setModpackVersions] = useState<ProjectVersion[]>([]);
  const [selectedModpackVersion, setSelectedModpackVersion] = useState<string | null>(null);
  const [loadingModpackVersions, setLoadingModpackVersions] = useState(false);
  const [modpackProgress, setModpackProgress] = useState<ModpackProgress | null>(null);

  const resetForm = () => {
    setActiveTab('custom');
    setSelectedServerType(null);
    setSearch('');
    setName('');
    setShowSnapshots(false);
    setSelectedVersion(null);
    setSelectedLoader('none');
    setSelectedLoaderVersion(null);
    setStartAfterCreation(false);
    setImportSourcePath(null);
    setImportServerType('vanilla');
    setAvailableJars([]);
    setSelectedJar(null);
    setAvailableScripts([]);
    setSelectedScript(null);
    setServerPropertiesExists(true);
    setBypassServerPropertiesCheck(false);
    setRootWithinZip(null);
    setImportProgress(null);
    setModpackProgress(null);
    setError(null);
    setModpackResults([]);
    setSelectedModpack(null);
    setModpackVersions([]);
    setSelectedModpackVersion(null);
  };

  useEffect(() => {
    let unlistenImport: any;
    let unlistenModpack: any;
    
    const setupListeners = async () => {
      unlistenImport = await listen<{ current: number, total: number, message: string }>('import-progress', (event) => {
        setImportProgress(event.payload);
      });

      unlistenModpack = await listen<ModpackProgress>('modpack-installation-progress', (event) => {
        setModpackProgress(event.payload);
      });
    };

    setupListeners();

    return () => {
      if (unlistenImport) unlistenImport();
      if (unlistenModpack) unlistenModpack();
    };
  }, []);

  useEffect(() => {
    if (isOpen) {
      loadVersions();
    }
  }, [isOpen, selectedServerType]);

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
    } else if (selectedServerType === 'quilt') {
      setSelectedLoader('quilt');
    } else if (selectedServerType === 'vanilla') {
      setSelectedLoader('none');
    } else if (selectedServerType) {
      setSelectedLoader(selectedServerType);
    } else {
      setSelectedLoader('none');
    }
  }, [selectedServerType]);

  useEffect(() => {
    if (!name) {
      setNameExists(false);
      return;
    }

    const timer = setTimeout(async () => {
      try {
        const exists = await invoke<boolean>('check_instance_name_exists', { name });
        setNameExists(exists);
      } catch (e) {
        console.error('Failed to check if instance name exists', e);
      }
    }, 300);

    return () => clearTimeout(timer);
  }, [name]);

  async function loadVersions() {
    if (!selectedServerType) {
      setLoading(false);
      return;
    }

    try {
      setLoading(true);
      setError(null);
      if (selectedServerType === 'bedrock') {
        const manifest = await invoke<VersionManifest>('get_bedrock_versions');
        setManifest(manifest);
      } else if (selectedServerType === 'velocity') {
        const versions = await invoke<string[]>('get_velocity_versions');
        const latestVersion = versions[0];
        const builds = await invoke<string[]>('get_velocity_builds', { version: latestVersion });
        setManifest({
          latest: { release: builds[0], snapshot: builds[0] },
          versions: builds.map(b => ({
            id: b,
            type: 'release',
            url: '',
            releaseTime: new Date().toISOString(),
          }))
        });
      } else if (selectedServerType === 'bungeecord') {
        const versions = await invoke<string[]>('get_bungeecord_versions');
        setManifest({
          latest: { release: versions[0], snapshot: versions[0] },
          versions: versions.map(v => ({
            id: v,
            type: 'release',
            url: '',
            releaseTime: new Date().toISOString(),
          }))
        });
      } else {
        const m = await invoke<VersionManifest>('get_minecraft_versions');
        setManifest(m);
      }
    } catch (e) {
      console.error('Failed to load versions', e);
      setError('Failed to load versions. Please check your internet connection.');
    } finally {
      setLoading(false);
    }
  }

  async function loadModLoaders(version: string) {
    const isModded = ['forge', 'fabric', 'quilt', 'neoforge', 'paper', 'purpur', 'velocity', 'bungeecord'].includes(selectedServerType || '');
    if (!isModded) {
      setModLoaders([]);
      return;
    }

    try {
      setLoadingModLoaders(true);
      const loaders = await invoke<ModLoader[]>('get_mod_loaders', { 
        mcVersion: version,
        serverType: selectedServerType 
      });
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

  useEffect(() => {
    if (filteredVersions.length === 1 && !selectedVersion) {
      setSelectedVersion(filteredVersions[0].id);
    }
  }, [filteredVersions, selectedVersion]);

  const debouncedSearch = useDebounce(search, 500);

  useEffect(() => {
    if (activeTab === 'modrinth') {
      searchModpacks(debouncedSearch);
    }
  }, [debouncedSearch, activeTab]);

  async function searchModpacks(query: string) {
    try {
      setSearchingModpacks(true);
      const results = await invoke<Project[]>('search_mods', {
        options: {
          query,
          project_type: 'modpack'
        },
        provider: 'Modrinth'
      });
      setModpackResults(results);
    } catch (e) {
      console.error('Failed to search modpacks', e);
      showToast('Failed to search modpacks', 'error');
    } finally {
      setSearchingModpacks(false);
    }
  }

  async function loadModpackVersions(projectId: string) {
    try {
      setLoadingModpackVersions(true);
      const versions = await invoke<ProjectVersion[]>('get_mod_versions', {
        projectId,
        provider: 'Modrinth'
      });
      setModpackVersions(versions);
      if (versions.length > 0) {
        setSelectedModpackVersion(versions[0].id);
      }
    } catch (e) {
      console.error('Failed to load modpack versions', e);
      showToast('Failed to load modpack versions', 'error');
    } finally {
      setLoadingModpackVersions(false);
    }
  }

  useEffect(() => {
    if (selectedModpack) {
      loadModpackVersions(selectedModpack.id);
    } else {
      setModpackVersions([]);
      setSelectedModpackVersion(null);
    }
  }, [selectedModpack]);

  async function handleCreate() {
    console.log('handleCreate called', { name, selectedVersion });
    if (activeTab === 'import') {
      return handleImport();
    }

    if (activeTab === 'modrinth') {
      if (!name || !selectedModpack || !selectedModpackVersion || nameExists) return;
      
      const version = modpackVersions.find(v => v.id === selectedModpackVersion);
      if (!version) return;

      try {
        setCreating(true);
        setError(null);
        const instance = await invoke<Instance>('create_instance_from_modpack', {
          name,
          version,
          startAfterCreation: startAfterCreation
        });
        showToast(`Successfully created instance "${name}"`, 'success');
        onCreated(instance);
        resetForm();
        onClose();
      } catch (e) {
        console.error('Failed to create instance from modpack', e);
        setError(e instanceof Error ? e.message : String(e));
      } finally {
        setCreating(false);
      }
      return;
    }

    if (!name || !selectedVersion || nameExists) {
      console.log('handleCreate validation failed', { name, selectedVersion, nameExists });
      return;
    }

    try {
      setCreating(true);
      setError(null);
      
      let version = selectedVersion;
      let modLoader = selectedLoader === 'none' ? null : selectedLoader;
      let loaderVersion = selectedLoaderVersion;

      // Special handling for Velocity/BungeeCord where "version" in UI is actually the build
      if (selectedServerType === 'velocity') {
        // We need the MC version (velocity version) and the build
        const versions = await invoke<string[]>('get_velocity_versions');
        version = versions[0]; // Use latest velocity version
        modLoader = 'velocity';
        loaderVersion = selectedVersion; // The selected build number
      } else if (selectedServerType === 'bungeecord') {
        version = 'latest';
        modLoader = 'bungeecord';
        loaderVersion = selectedVersion; // The selected build (though usually just 'latest')
      }

      const instance = await invoke<Instance>('create_instance_full', {
        name,
        version,
        modLoader: modLoader,
        loaderVersion: loaderVersion,
        startAfterCreation: startAfterCreation,
      });
      showToast(`Successfully created instance "${name}"`, 'success');
      onCreated(instance);
      resetForm();
      onClose();
    } catch (e) {
      console.error('Failed to create instance', e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setCreating(false);
    }
  }

  async function handleImport() {
    console.log('handleImport called', { name, importSourcePath, selectedJar });
    if (!name || !importSourcePath || !selectedJar || nameExists) {
      console.log('handleImport validation failed', { name, importSourcePath, selectedJar, nameExists });
      return;
    }

    try {
      setCreating(true);
      setError(null);

      const instance = await invoke<Instance>('import_instance', {
        name,
        sourcePath: importSourcePath,
        jarName: selectedJar,
        serverType: importServerType,
        rootWithinZip,
        scriptPath: selectedScript,
      });
      showToast(`Successfully imported instance "${name}"`, 'success');
      onCreated(instance);
      resetForm();
      onClose();
    } catch (e) {
      console.error('Failed to import instance', e);
      setError(e instanceof Error ? e.message : String(e));
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
    startAfterCreation,
    setStartAfterCreation,
    creating,
    error,
    setError,
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
    availableScripts,
    setAvailableScripts,
    selectedScript,
    setSelectedScript,
    serverPropertiesExists,
    setServerPropertiesExists,
    bypassServerPropertiesCheck,
    setBypassServerPropertiesCheck,
    rootWithinZip,
    setRootWithinZip,
    importProgress,
    nameExists,
    modpackResults,
    searchingModpacks,
    selectedModpack,
    setSelectedModpack,
    modpackVersions,
    selectedModpackVersion,
    setSelectedModpackVersion,
    loadingModpackVersions,
    modpackProgress
  };
}
