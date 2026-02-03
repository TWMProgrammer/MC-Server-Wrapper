import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { Select } from '../../../ui/components/Select';

// Mock the hook to provide default settings
vi.mock('../../../ui/hooks/useAppSettings', () => ({
  useAppSettings: () => ({
    settings: { scaling: 1 }
  }),
  AppSettingsProvider: ({ children }: { children: React.ReactNode }) => children
}));

const mockOptions = [
  { value: 'option1', label: 'Option 1' },
  { value: 'option2', label: 'Option 2' },
  { value: 'option3', label: 'Option 3' },
];

describe('Select Component', () => {
  const mockOnChange = vi.fn();

  beforeEach(() => {
    mockOnChange.mockClear();
  });

  it('renders correctly with placeholder', () => {
    render(
      <Select
        placeholder="Select an option"
        options={mockOptions}
        value=""
        onChange={mockOnChange}
      />
    );

    expect(screen.getByText('Select an option')).toBeDefined();
  });

  it('shows options when clicked', async () => {
    render(
      <Select
        options={mockOptions}
        value=""
        onChange={mockOnChange}
      />
    );

    const trigger = screen.getByRole('button');
    fireEvent.click(trigger);

    await waitFor(() => {
      expect(screen.getByText('Option 1')).toBeDefined();
      expect(screen.getByText('Option 2')).toBeDefined();
      expect(screen.getByText('Option 3')).toBeDefined();
    });
  });

  it('calls onChange when an option is selected', async () => {
    render(
      <Select
        options={mockOptions}
        value=""
        onChange={mockOnChange}
      />
    );

    const trigger = screen.getByRole('button');
    fireEvent.click(trigger);

    const option = screen.getByText('Option 1');
    fireEvent.click(option);

    expect(mockOnChange).toHaveBeenCalledWith('option1');
  });

  it('displays the selected option label', () => {
    render(
      <Select
        options={mockOptions}
        value="option2"
        onChange={mockOnChange}
      />
    );

    expect(screen.getByText('Option 2')).toBeDefined();
  });
});
