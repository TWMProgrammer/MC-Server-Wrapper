import { renderHook, act, screen, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { ToastProvider, useToast } from '../../../ui/hooks/useToast';
import React from 'react';

// Mock framer-motion to avoid animation delays
vi.mock('framer-motion', () => ({
  motion: {
    div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
  },
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

describe('useToast', () => {
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <ToastProvider>{children}</ToastProvider>
  );

  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('shows a toast and then it disappears', async () => {
    const { result } = renderHook(() => useToast(), { wrapper });

    act(() => {
      result.current.showToast('Test Message', 'success');
    });

    expect(screen.getByText('Test Message')).toBeDefined();

    act(() => {
      vi.advanceTimersByTime(3100);
    });

    // Give React a chance to update state after timer fires
    await act(async () => {
      await Promise.resolve();
    });

    expect(screen.queryByText('Test Message')).toBeNull();
  });

  it('queues multiple toasts', async () => {
    const { result } = renderHook(() => useToast(), { wrapper });

    act(() => {
      result.current.showToast('Message 1');
      result.current.showToast('Message 2');
    });

    expect(screen.getByText('Message 1')).toBeDefined();
    expect(screen.getByText('Message 2')).toBeDefined();

    act(() => {
      vi.advanceTimersByTime(3100);
    });

    // Give React a chance to update state after timer fires
    await act(async () => {
      await Promise.resolve();
    });

    expect(screen.queryByText('Message 1')).toBeNull();
    expect(screen.queryByText('Message 2')).toBeNull();
  });
});
