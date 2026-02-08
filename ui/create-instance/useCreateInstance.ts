import { useState, useEffect, useMemo } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { Instance } from '../types'
import { VersionManifest, ModLoader, Tab } from './types'
import { useToast } from '../hooks/useToast'

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
    setError(null);
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
    try {
      setLoading(true);
      if (selectedServerType === 'bedrock') {
        const manifest = await invoke<VersionManifest>('get_bedrock_versions');
        setManifest(manifest);
      } else if (selectedServerType === 'velocity') {
        // For Velocity, we first get the versions, then we'll show builds for the latest version
        // Actually, the user wants to see builds directly in the list.
        // Let's fetch the builds for the latest stable velocity version.
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

  async function handleCreate() {
    console.log('handleCreate called', { name, selectedVersion });
    if (activeTab === 'import') {
      return handleImport();
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
    nameExists
  };
}
