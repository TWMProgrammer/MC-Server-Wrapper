import { renderHook, act, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { useServer } from '../../../ui/hooks/useServer';

// Mock Tauri APIs
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

describe('useServer', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    (window as any).__TAURI_INTERNALS__ = {};
    vi.useFakeTimers();
    (listen as any).mockReturnValue(Promise.resolve(() => {}));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('loads instances on mount', async () => {
    const mockInstances = [
      { id: '1', name: 'Server 1', status: 'Stopped' }
    ];
    (invoke as any).mockResolvedValue(mockInstances);

    const { result } = renderHook(() => useServer());

    // With fake timers, we need to advance them if there are any timeouts/intervals
    // but loadInstances is just a promise in useEffect.
    // However, useServer has an interval that starts immediately.
    
    await act(async () => {
      await Promise.resolve(); // allow useEffect to run
    });

    expect(result.current.instances).toHaveLength(1);
    expect(result.current.instances[0].name).toBe('Server 1');
    expect(result.current.loading).toBe(false);
  });

  it('synchronizes server status', async () => {
    const mockInstances = [
      { id: '1', name: 'Server 1', status: 'Stopped' }
    ];
    (invoke as any).mockImplementation((cmd: string) => {
      if (cmd === 'list_instances') return Promise.resolve(mockInstances);
      if (cmd === 'get_server_status') return Promise.resolve('Running');
      return Promise.resolve(null);
    });

    const { result } = renderHook(() => useServer());

    await act(async () => {
      await Promise.resolve();
    });

    // Fast-forward time to trigger the status check interval
    await act(async () => {
      vi.advanceTimersByTime(2000);
    });

    // We need another tick for the state update to settle
    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.instances[0].status).toBe('Running');
  });

  it('handles log streaming', async () => {
    let logHandler: any;
    (listen as any).mockImplementation((event: string, handler: any) => {
      if (event === 'server-log') {
        logHandler = handler;
      }
      return Promise.resolve(() => {});
    });

    const { result } = renderHook(() => useServer());

    await act(async () => {
      if (logHandler) {
        logHandler({ payload: { instance_id: '1', line: 'Server started' } });
      }
    });

    expect(result.current.logs['1']).toContain('Server started');
  });
});
