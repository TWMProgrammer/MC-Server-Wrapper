import { renderHook, act, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AppSettingsProvider, useAppSettings } from '../../../ui/hooks/useAppSettings';
import React from 'react';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/app', () => ({
  getVersion: vi.fn().mockResolvedValue('1.0.0'),
}));

import { invoke } from '@tauri-apps/api/core';
import { getVersion } from '@tauri-apps/api/app';

describe('useAppSettings', () => {
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <AppSettingsProvider>{children}</AppSettingsProvider>
  );

  beforeEach(() => {
    vi.clearAllMocks();
    document.documentElement.classList.remove('dark');
    document.documentElement.style.removeProperty('--primary');
  });

  it('loads settings on mount', async () => {
    const mockSettings = {
      accent_color: 'Blue',
      theme: 'dark',
      scaling: 0.9,
    };
    (invoke as any).mockResolvedValueOnce(mockSettings);

    const { result } = renderHook(() => useAppSettings(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    expect(result.current.settings.scaling).toBe(0.9);
    expect(result.current.settings.theme).toBe('dark');
  });

  it('updates settings and persists them', async () => {
    const mockSettings = {
      accent_color: 'Blue',
      theme: 'dark',
      scaling: 0.8,
    };
    (invoke as any).mockResolvedValueOnce(mockSettings);

    const { result } = renderHook(() => useAppSettings(), { wrapper });

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false);
    });

    await act(async () => {
      await result.current.setScaling(1.1);
    });

    expect(result.current.settings.scaling).toBe(1.1);
    expect(invoke).toHaveBeenCalledWith('update_app_settings', expect.objectContaining({
      settings: expect.objectContaining({ scaling: 1.1 })
    }));
  });

  it('applies theme and accent color to document element', async () => {
    const mockSettings = {
      accent_color: 'Emerald',
      theme: 'dark',
      scaling: 0.8,
    };
    (invoke as any).mockResolvedValueOnce(mockSettings);

    renderHook(() => useAppSettings(), { wrapper });

    await waitFor(() => {
      expect(document.documentElement.classList.contains('dark')).toBe(true);
      expect(document.documentElement.style.getPropertyValue('--primary')).toBe('160 84% 39%');
    });
  });
});
