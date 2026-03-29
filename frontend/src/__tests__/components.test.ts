/**
 * @jest-environment jsdom
 */
import { render, screen, fireEvent, waitFor } from '@testing-library/svelte/svelte5';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import Dashboard from '../routes/admin/+page.svelte';
import ApiKeys from '../routes/apikeys/+page.svelte';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockReset();
  });

  it('shows loading state initially', () => {
    mockFetch.mockImplementation(() => new Promise(() => {})); // Never resolves
    render(Dashboard);
    const spinner = document.querySelector('.animate-spin');
    expect(spinner).toBeTruthy();
  });

  it('displays stats after loading', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        total_users: 100,
        total_accounts: 50,
        total_requests_today: 1000,
        total_revenue: 50000,
        active_users: 75,
        active_accounts: 40,
      }),
    });

    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('100')).toBeTruthy();
    }, { timeout: 5000 });
  });

  it('refreshes stats on button click', async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => ({
        total_users: 100,
        total_accounts: 50,
        total_requests_today: 1000,
        total_revenue: 50000,
        active_users: 75,
        active_accounts: 40,
      }),
    });

    render(Dashboard);

    await waitFor(() => {
      expect(screen.getByText('Refresh')).toBeTruthy();
    }, { timeout: 5000 });

    const refreshButton = screen.getByText('Refresh');
    await fireEvent.click(refreshButton);

    expect(mockFetch).toHaveBeenCalled();
  });
});

describe('API Keys', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockReset();
  });

  it('shows loading state initially', () => {
    mockFetch.mockImplementation(() => new Promise(() => {}));
    render(ApiKeys);
    const spinner = document.querySelector('.animate-spin');
    expect(spinner).toBeTruthy();
  });

  it('shows create button', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ data: [] }),
    });

    render(ApiKeys);

    await waitFor(() => {
      expect(screen.getByText(/create new key/i)).toBeTruthy();
    }, { timeout: 5000 });
  });
});
