import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { PluginConfigModal } from '../../../ui/plugins/PluginConfigModal';
import React from 'react';

// Mock Lucide icons
vi.mock('lucide-react', () => ({
    X: () => <div data-testid="icon-x" />,
    Save: () => <div data-testid="icon-save" />,
    RefreshCw: () => <div data-testid="icon-refresh" />,
    FileText: () => <div data-testid="icon-file" />,
    Settings: () => <div data-testid="icon-settings" />,
    AlertCircle: () => <div data-testid="icon-alert" />,
    Terminal: () => <div data-testid="icon-terminal" />,
    Maximize2: () => <div data-testid="icon-maximize" />,
    Minimize2: () => <div data-testid="icon-minimize" />,
    ChevronRight: () => <div data-testid="icon-chevron-right" />,
    ChevronDown: () => <div data-testid="icon-chevron-down" />,
    Folder: () => <div data-testid="icon-folder" />,
    Database: () => <div data-testid="icon-database" />,
    Plus: () => <div data-testid="icon-plus" />,
    Trash2: () => <div data-testid="icon-trash" />,
    MoreVertical: () => <div data-testid="icon-more" />,
}));

// Mock Tauri invoke
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
    invoke: (...args: any[]) => mockInvoke(...args),
}));

// Mock useToast
const mockShowToast = vi.fn();
vi.mock('../../../ui/hooks/useToast', () => ({
    useToast: () => ({
        showToast: mockShowToast,
    }),
}));

// Mock useAppSettings
vi.mock('../../../ui/hooks/useAppSettings', () => ({
    useAppSettings: () => ({
        settings: { scaling: 1 },
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

// Mock Monaco Editor
vi.mock('@monaco-editor/react', () => ({
    default: (props: any) => {
        const { value, onChange, onMount } = props;
        const [localValue, setLocalValue] = React.useState(value);

        React.useEffect(() => {
            setLocalValue(value);
        }, [value]);

        React.useEffect(() => {
            if (onMount) {
                onMount({
                    getValue: () => value, // Use the prop value directly, it should be updated by setContent
                });
            }
        }, [onMount, value]);

        return (
            <textarea
                data-testid="monaco-editor"
                value={localValue}
                onChange={(e) => {
                    const newValue = e.target.value;
                    setLocalValue(newValue);
                    if (onChange) onChange(newValue);
                }}
            />
        );
    },
    loader: { config: vi.fn() },
}));

describe('PluginConfigModal', () => {
    const mockPlugin = {
        name: 'TestPlugin',
        filename: 'TestPlugin.jar',
        version: '1.0.0',
        author: 'Author',
        description: 'Description',
        enabled: true,
    } as any;
    const instanceId = 'test-instance-id';
    const onClose = vi.fn();

    beforeEach(() => {
        vi.clearAllMocks();
        mockInvoke.mockImplementation(async (command, args) => {
            if (command === 'list_plugin_configs') {
                return { config_dir: 'TestPlugin', files: ['config.yml', 'settings.json'] };
            }
            if (command === 'read_text_file') {
                return 'key: value';
            }
            if (command === 'get_config_value') {
                return { key: 'value' };
            }
            if (command === 'save_text_file' || command === 'save_config_value') {
                return null;
            }
            return null;
        });
    });

    it('renders and loads configuration files', async () => {
        render(
            <PluginConfigModal
                plugin={mockPlugin}
                instanceId={instanceId}
                onClose={onClose}
            />
        );

        await waitFor(() => {
            expect(screen.getAllByText(/config.yml/).length).toBeGreaterThan(0);
            expect(screen.getByText(/settings.json/)).toBeDefined();
        });
    });

    it('switches to tree mode and loads parsed content', async () => {
        render(
            <PluginConfigModal
                plugin={mockPlugin}
                instanceId={instanceId}
                onClose={onClose}
            />
        );

        await waitFor(() => expect(screen.getAllByText(/config.yml/).length).toBeGreaterThan(0));

        const treeToggle = screen.getByTitle(/Switch to Tree Editor/);
        fireEvent.click(treeToggle);

        await waitFor(() => {
            expect(mockInvoke).toHaveBeenCalledWith('get_config_value', expect.objectContaining({
                format: 'Yaml'
            }));
            expect(screen.getByDisplayValue('value')).toBeDefined();
        });
    });

    it('saves content in text mode', async () => {
        render(
            <PluginConfigModal
                plugin={mockPlugin}
                instanceId={instanceId}
                onClose={onClose}
            />
        );

        // Wait for initial content to load
        await waitFor(() => {
            const editor = screen.getByTestId('monaco-editor') as HTMLTextAreaElement;
            if (editor.value !== 'key: value') throw new Error('Content not loaded yet');
        });

        const editor = screen.getByTestId('monaco-editor');
        fireEvent.change(editor, { target: { value: 'new: content' } });

        const saveButton = screen.getByText(/Save Changes/);

        // Wait for state to update
        await waitFor(() => {
            const currentEditor = screen.getByTestId('monaco-editor') as HTMLTextAreaElement;
            if (currentEditor.value !== 'new: content') throw new Error('Not updated yet');
        });

        fireEvent.click(saveButton);

        await waitFor(() => {
            const saveCall = mockInvoke.mock.calls.find(call => call[0] === 'save_text_file');
            expect(saveCall).toBeDefined();
            expect(saveCall![1]).toMatchObject({
                content: 'new: content'
            });
        }, { timeout: 3000 });
    });

    it('saves content in tree mode', async () => {
        render(
            <PluginConfigModal
                plugin={mockPlugin}
                instanceId={instanceId}
                onClose={onClose}
            />
        );

        await waitFor(() => expect(screen.getAllByText(/config.yml/).length).toBeGreaterThan(0));

        fireEvent.click(screen.getByTitle(/Switch to Tree Editor/));

        await waitFor(() => screen.getByDisplayValue('value'));

        const input = screen.getByDisplayValue('value');
        fireEvent.change(input, { target: { value: 'new_value' } });

        const saveButton = screen.getByText(/Save Changes/);
        fireEvent.click(saveButton);

        await waitFor(() => {
            expect(mockInvoke).toHaveBeenCalledWith('save_config_value', expect.objectContaining({
                value: { key: 'new_value' }
            }));
        });
    });
});
