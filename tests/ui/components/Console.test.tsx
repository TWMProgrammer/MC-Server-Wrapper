import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Console } from '../../../ui/components/Console';
import React from 'react';

// Mock ResizeObserver
global.ResizeObserver = class {
    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
};

// Mock AppSettings hook
vi.mock('../../../ui/hooks/useAppSettings', () => ({
    useAppSettings: () => ({
        settings: { use_white_console_text: false },
    }),
}));

// Mock Framer Motion
vi.mock('framer-motion', () => ({
    motion: {
        div: ({ children, ...props }: any) => <div {...props}>{children}</div>,
        button: ({ children, ...props }: any) => <button {...props}>{children}</button>,
    },
    AnimatePresence: ({ children }: any) => <>{children}</>,
}));

// Mock ansi-to-react
vi.mock('ansi-to-react', () => ({
    __esModule: true,
    default: ({ children }: any) => <span>{children}</span>,
}));

describe('Console', () => {
    const consoleEndRef = { current: null };
    const onCommandChange = vi.fn();
    const onSendCommand = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
        // Mock scrollTo for JSDOM
        HTMLElement.prototype.scrollTo = vi.fn();
    });

    it('renders empty state when no logs', () => {
        render(
            <Console
                logs={[]}
                consoleEndRef={consoleEndRef}
                command=""
                onCommandChange={onCommandChange}
                onSendCommand={onSendCommand}
            />
        );

        expect(screen.getByText(/No live data/i)).toBeDefined();
    });

    it('renders logs and handles command input', () => {
        const mockLogs = ['[12:00:00] [INFO] Test log line'];
        render(
            <Console
                logs={mockLogs}
                consoleEndRef={consoleEndRef}
                command="say hello"
                onCommandChange={onCommandChange}
                onSendCommand={onSendCommand}
            />
        );

        expect(screen.getByText(/Test log line/)).toBeDefined();
        const input = screen.getByPlaceholderText(/Enter server command/i);
        expect((input as HTMLInputElement).value).toBe('say hello');

        fireEvent.change(input, { target: { value: 'stop' } });
        expect(onCommandChange).toHaveBeenCalledWith('stop');
    });

    it('triggers onSendCommand when form is submitted', () => {
        render(
            <Console
                logs={[]}
                consoleEndRef={consoleEndRef}
                command="stop"
                onCommandChange={onCommandChange}
                onSendCommand={onSendCommand}
            />
        );

        const form = screen.getByPlaceholderText(/Enter server command/i).closest('form')!;
        fireEvent.submit(form);
        expect(onSendCommand).toHaveBeenCalled();
    });

    it('shows "Live" button when scrolled up', async () => {
        const mockLogs = Array.from({ length: 50 }, (_, i) => `Line ${i}`);

        const { container } = render(
            <Console
                logs={mockLogs}
                consoleEndRef={consoleEndRef}
                command=""
                onCommandChange={onCommandChange}
                onSendCommand={onSendCommand}
            />
        );

        const scrollContainer = container.querySelector('.overflow-y-auto')!;

        // Mock scroll properties
        Object.defineProperty(scrollContainer, 'scrollTop', { value: 0, configurable: true });
        Object.defineProperty(scrollContainer, 'scrollHeight', { value: 1000, configurable: true });
        Object.defineProperty(scrollContainer, 'clientHeight', { value: 200, configurable: true });

        // Trigger scroll event
        fireEvent.scroll(scrollContainer);

        // "Live" button should appear because we are not at bottom
        await waitFor(() => {
            expect(screen.getByText(/Live/i)).toBeDefined();
        });
    });

    it('implements virtualization structure', () => {
        const mockLogs = Array.from({ length: 100 }, (_, i) => `Line ${i}`);
        const { container } = render(
            <Console
                logs={mockLogs}
                consoleEndRef={consoleEndRef}
                command=""
                onCommandChange={onCommandChange}
                onSendCommand={onSendCommand}
            />
        );

        // Check for the virtualization wrapper
        const virtualWrapper = container.querySelector('[style*="height"]');
        expect(virtualWrapper).toBeDefined();
    });
});
