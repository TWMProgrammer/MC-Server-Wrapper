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
vi.mock('framer-motion', async () => {
    const React = await import('react');
    const motionProps = [
        'layout', 'layoutId', 'initial', 'animate', 'whileHover',
        'whileTap', 'transition', 'exit', 'variants', 'whileInView',
        'viewport', 'onLayoutAnimationComplete', 'onAnimationStart',
        'onAnimationComplete', 'onUpdate', 'onDragStart', 'onDragEnd',
        'onDrag', 'onDirectionLock', 'onDragTransitionEnd', 'drag',
        'dragControls', 'dragListener', 'dragConstraints', 'dragElastic',
        'dragMomentum', 'dragPropagation', 'dragSnapToOrigin',
        'layoutDependency', 'onViewportEnter', 'onViewportLeave'
    ];

    const filterProps = (props: any) => {
        const filtered = { ...props };
        motionProps.forEach(prop => delete filtered[prop]);
        return filtered;
    };

    const motion = new Proxy({}, {
        get: (_target, key: string) => {
            return React.forwardRef(({ children, ...props }: any, ref: any) => {
                const Tag = key as any;
                return React.createElement(Tag, { ...filterProps(props), ref }, children);
            });
        }
    });

    return {
        motion,
        AnimatePresence: ({ children }: any) => <>{children}</>,
    };
});

// Mock ansi-to-react
vi.mock('ansi-to-react', () => ({
    __esModule: true,
    default: ({ children }: any) => <span>{children}</span>,
}));

const ConsoleHistoryTestWrapper = ({
    commandHistory,
    mockOnCommandChange,
    consoleEndRef,
    onSendCommand
}: {
    commandHistory: string[],
    mockOnCommandChange: (val: string) => void,
    consoleEndRef: React.RefObject<HTMLDivElement | null>,
    onSendCommand: (e: React.FormEvent) => void
}) => {
    const [cmd, setCmd] = React.useState('current typed');
    return (
        <Console
            logs={[]}
            consoleEndRef={consoleEndRef}
            command={cmd}
            commandHistory={commandHistory}
            onCommandChange={(val) => {
                setCmd(val);
                mockOnCommandChange(val);
            }}
            onSendCommand={onSendCommand}
        />
    );
};

describe('Console', () => {
    const consoleEndRef = React.createRef<HTMLDivElement>();
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

        const scrollContainer = container.querySelector('.overflow-auto')!;

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

    it('implements virtualization structure when wrapping is disabled', () => {
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

        // Disable wrapping
        const wrapCheckbox = screen.getByText('Wrap');
        fireEvent.click(wrapCheckbox);

        // Check for the virtualization wrapper (div with explicit height style)
        // We look for the inner container that has the total height set
        // The style string might vary in spacing, so we look for the attribute
        const virtualWrapper = container.querySelector('div[style*="height:"]');
        expect(virtualWrapper).not.toBeNull();
    });

    it('renders wrapped logs by default without virtualization', () => {
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

        // Should NOT have virtualization wrapper by default
        const virtualWrapper = container.querySelector('div[style*="height:"]');
        expect(virtualWrapper).toBeNull();

        // Should have simple flex container
        const flexContainer = container.querySelector('.flex.flex-col');
        expect(flexContainer).not.toBeNull();
    });

    it('handles up and down arrow keys for command history', async () => {
        const commandHistory = ['command1', 'command2', 'command3'];
        const mockOnCommandChange = vi.fn();

        render(
            <ConsoleHistoryTestWrapper
                commandHistory={commandHistory}
                mockOnCommandChange={mockOnCommandChange}
                consoleEndRef={consoleEndRef}
                onSendCommand={onSendCommand}
            />
        );

        // Press Up arrow
        fireEvent.keyDown(screen.getByPlaceholderText(/Enter server command/i), { key: 'ArrowUp' });
        await waitFor(() => expect(mockOnCommandChange).toHaveBeenCalledWith('command3'));

        // Press Up arrow again
        fireEvent.keyDown(screen.getByPlaceholderText(/Enter server command/i), { key: 'ArrowUp' });
        await waitFor(() => expect(mockOnCommandChange).toHaveBeenCalledWith('command2'));

        // Press Down arrow
        fireEvent.keyDown(screen.getByPlaceholderText(/Enter server command/i), { key: 'ArrowDown' });
        await waitFor(() => expect(mockOnCommandChange).toHaveBeenCalledWith('command3'));

        // Press Down arrow to return to original typed command
        fireEvent.keyDown(screen.getByPlaceholderText(/Enter server command/i), { key: 'ArrowDown' });
        await waitFor(() => expect(mockOnCommandChange).toHaveBeenCalledWith('current typed'));
    });
});
