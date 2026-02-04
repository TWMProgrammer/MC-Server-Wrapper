import { render, screen, fireEvent, waitFor, within } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { ModMarketplace } from '../../../ui/mods/ModMarketplace';
import React from 'react';

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
    observe = vi.fn();
    unobserve = vi.fn();
    disconnect = vi.fn();
};

// Mock Lucide icons
vi.mock('lucide-react', () => ({
    Search: () => <div data-testid="icon-search" />,
    Download: () => <div data-testid="icon-download" />,
    ExternalLink: () => <div data-testid="icon-external" />,
    Star: () => <div data-testid="icon-star" />,
    User: () => <div data-testid="icon-user" />,
    Package: () => <div data-testid="icon-package" />,
    Filter: () => <div data-testid="icon-filter" />,
    RefreshCw: () => <div data-testid="icon-refresh" />,
    Globe: () => <div data-testid="icon-globe" />,
    ChevronRight: () => <div data-testid="icon-chevron-right" />,
    ChevronLeft: () => <div data-testid="icon-chevron-left" />,
    Check: () => <div data-testid="icon-check" />,
    Tag: () => <div data-testid="icon-tag" />,
    Layers: () => <div data-testid="icon-layers" />,
    Cpu: () => <div data-testid="icon-cpu" />,
    Calendar: () => <div data-testid="icon-calendar" />,
    Clock: () => <div data-testid="icon-clock" />,
    ShieldCheck: () => <div data-testid="icon-shield" />,
    X: () => <div data-testid="icon-x" />,
    LayoutGrid: () => <div data-testid="icon-grid" />,
    List: () => <div data-testid="icon-list" />,
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

// Mock components
vi.mock('../../../ui/components/Select', () => ({
    Select: ({ value, onChange, options }: any) => (
        <select value={value} onChange={(e) => onChange(e.target.value)} data-testid="mock-select">
            {options.map((opt: any) => (
                <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
        </select>
    ),
}));

vi.mock('../../../ui/mods/ModDetailsModal', () => ({
    ModDetailsModal: () => <div data-testid="mod-details-modal" />,
}));

vi.mock('../../../ui/mods/ModReviewModal', () => ({
    ModReviewModal: ({ onConfirm, selectedMods, isInstalling }: any) => (
        <div data-testid="mod-review-modal">
            <h2>Review and Confirm</h2>
            {selectedMods.map((mod: any) => <div key={mod.id}>{mod.title}</div>)}
            <button onClick={() => onConfirm(selectedMods)} disabled={isInstalling}>
                {isInstalling ? 'Installing...' : `Install ${selectedMods.length} Mods`}
            </button>
        </div>
    ),
}));

vi.mock('../../../ui/hooks/useGridPageSize', () => ({
    useGridPageSize: () => 16,
}));

const mockInstance = {
    id: 'test-instance',
    name: 'Test Instance',
    version: '1.20.1',
    mod_loader: 'Fabric',
};

const mockProjects = Array.from({ length: 16 }, (_, i) => ({
    id: `mod-${i + 1}`,
    title: `Test Mod ${i + 1}`,
    description: `Description ${i + 1}`,
    icon_url: null,
    author: `Author ${i + 1}`,
    downloads: 1000 - i * 10,
    follows: 100 - i,
    provider: 'Modrinth',
    categories: i === 0 ? ['optimization'] : ['technology'],
}));

describe('ModMarketplace', () => {
    const instanceId = 'test-instance';

    beforeEach(() => {
        vi.clearAllMocks();
        mockInvoke.mockImplementation((command, args) => {
            if (command === 'list_instances') return Promise.resolve([mockInstance]);
            if (command === 'get_instance') return Promise.resolve(mockInstance);
            if (command === 'search_mods') return Promise.resolve(mockProjects);
            if (command === 'get_mod_dependencies') return Promise.resolve([]);
            if (command === 'install_mod') return Promise.resolve();
            return Promise.reject(new Error(`Unknown command: ${command}`));
        });
    });

    it('renders and performs initial search', async () => {
        render(<ModMarketplace instanceId={instanceId} />);

        await waitFor(() => {
            expect(screen.getByText('Test Mod 1')).toBeDefined();
            expect(screen.getByText('Test Mod 2')).toBeDefined();
        });

        expect(mockInvoke).toHaveBeenCalledWith('search_mods', expect.objectContaining({
            options: expect.objectContaining({
                game_version: '1.20.1',
                loader: 'Fabric',
            }),
            provider: 'Modrinth',
        }));
    });

    it('filters by category', async () => {
        render(<ModMarketplace instanceId={instanceId} />);
        await waitFor(() => screen.getByText('Test Mod 1'));
        mockInvoke.mockClear();

        const optimizationBtn = screen.getByTestId('category-optimization');
        fireEvent.click(optimizationBtn);

        await waitFor(() => {
            const calls = mockInvoke.mock.calls;
            const hasCategoryCall = calls.some(call =>
                call[0] === 'search_mods' &&
                call[1].options.facets?.some((f: string) => f.includes('optimization'))
            );
            expect(hasCategoryCall).toBe(true);
        });
    });

    it('searches for mods when query is submitted', async () => {
        render(<ModMarketplace instanceId={instanceId} />);
        await waitFor(() => screen.getByText('Test Mod 1'));
        mockInvoke.mockClear();

        const searchInput = screen.getByPlaceholderText(/Search mods/i);
        fireEvent.change(searchInput, { target: { value: 'test query' } });

        const searchForm = searchInput.closest('form');
        if (searchForm) fireEvent.submit(searchForm);

        await waitFor(() => {
            const calls = mockInvoke.mock.calls.filter(call => call[0] === 'search_mods');
            const hasSearchCall = calls.some(call =>
                call[1].options.query === 'test query'
            );
            expect(hasSearchCall).toBe(true);
        });
    });

    it('handles mod selection and installation review', async () => {
        render(<ModMarketplace instanceId={instanceId} />);
        await waitFor(() => screen.getByText('Test Mod 1'));

        const modCard = screen.getByText('Test Mod 1').closest('[data-testid="mod-card"]') as HTMLElement;
        const selectBtn = within(modCard).getByText('Select');
        fireEvent.click(selectBtn);

        await waitFor(() => {
            expect(screen.getByText(/Review & Confirm/)).toBeDefined();
        });

        fireEvent.click(screen.getByText(/Review & Confirm/));

        await waitFor(() => {
            expect(screen.getByTestId('mod-review-modal')).toBeDefined();
        });

        const confirmBtn = screen.getByText(/Install 1 Mods/);
        fireEvent.click(confirmBtn);

        await waitFor(() => {
            expect(mockInvoke).toHaveBeenCalledWith('install_mod', expect.objectContaining({
                projectId: 'mod-1',
            }));
            expect(mockShowToast).toHaveBeenCalledWith(expect.stringContaining('installed'), 'success');
        });
    });

    it('handles pagination', async () => {
        render(<ModMarketplace instanceId={instanceId} />);
        await waitFor(() => screen.getByText('Test Mod 1'));
        mockInvoke.mockClear();

        const nextBtn = screen.getByTestId('next-page-btn');
        fireEvent.click(nextBtn);

        await waitFor(() => {
            const calls = mockInvoke.mock.calls;
            const hasPageCall = calls.some(call =>
                call[0] === 'search_mods' &&
                call[1].options.offset === 16
            );
            expect(hasPageCall).toBe(true);
        });
    });
});
