import { describe, it, expect } from 'vitest';
import { supportsPlugins, supportsMods, getAvailableTabs, ALL_TABS } from '../../ui/utils/tabUtils';

describe('tabUtils', () => {
  describe('supportsPlugins', () => {
    it('returns true for supported loaders', () => {
      expect(supportsPlugins('paper')).toBe(true);
      expect(supportsPlugins('Purpur')).toBe(true);
      expect(supportsPlugins('SPIGOT')).toBe(true);
      expect(supportsPlugins('bukkit')).toBe(true);
      expect(supportsPlugins('velocity')).toBe(true);
    });

    it('returns false for unsupported loaders or empty input', () => {
      expect(supportsPlugins('fabric')).toBe(false);
      expect(supportsPlugins('forge')).toBe(false);
      expect(supportsPlugins('')).toBe(false);
      expect(supportsPlugins(undefined)).toBe(false);
    });
  });

  describe('supportsMods', () => {
    it('returns true for supported loaders', () => {
      expect(supportsMods('fabric')).toBe(true);
      expect(supportsMods('Forge')).toBe(true);
      expect(supportsMods('NEOFORGE')).toBe(true);
      expect(supportsMods('quilt')).toBe(true);
    });

    it('returns false for unsupported loaders or empty input', () => {
      expect(supportsMods('paper')).toBe(false);
      expect(supportsMods('vanilla')).toBe(false);
      expect(supportsMods('')).toBe(false);
      expect(supportsMods(undefined)).toBe(false);
    });
  });

  describe('getAvailableTabs', () => {
    it('returns all tabs except plugins/mods for vanilla', () => {
      const tabs = getAvailableTabs('vanilla');
      const tabIds = tabs.map(t => t.id);
      expect(tabIds).toContain('dashboard');
      expect(tabIds).toContain('console');
      expect(tabIds).not.toContain('plugins');
      expect(tabIds).not.toContain('mods');
    });

    it('includes plugins for paper', () => {
      const tabs = getAvailableTabs('paper');
      const tabIds = tabs.map(t => t.id);
      expect(tabIds).toContain('plugins');
      expect(tabIds).not.toContain('mods');
    });

    it('includes plugins for velocity', () => {
      const tabs = getAvailableTabs('velocity');
      const tabIds = tabs.map(t => t.id);
      expect(tabIds).toContain('plugins');
      expect(tabIds).not.toContain('mods');
    });

    it('includes mods for fabric', () => {
      const tabs = getAvailableTabs('fabric');
      const tabIds = tabs.map(t => t.id);
      expect(tabIds).toContain('mods');
      expect(tabIds).not.toContain('plugins');
    });

    it('returns correct number of tabs', () => {
      const vanillaTabs = getAvailableTabs('vanilla');
      const paperTabs = getAvailableTabs('paper');
      const fabricTabs = getAvailableTabs('fabric');
      
      expect(vanillaTabs.length).toBe(ALL_TABS.length - 2);
      expect(paperTabs.length).toBe(ALL_TABS.length - 1);
      expect(fabricTabs.length).toBe(ALL_TABS.length - 1);
    });
  });
});
