import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/svelte'
import userEvent from '@testing-library/user-event'
import CreateSecret from './CreateSecret.svelte'

describe('CreateSecret', () => {
  beforeEach(() => {
    vi.resetAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders the form initially', () => {
    render(CreateSecret)

    expect(screen.getByText('Del en hemmelighet')).toBeInTheDocument()
    expect(screen.getByLabelText(/Hemmelighet/)).toBeInTheDocument()
    expect(screen.getByLabelText(/Maksimalt antall visninger/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Opprett hemmelighet/ })).toBeInTheDocument()
  })

  it('shows error when submitting empty secret', async () => {
    const user = userEvent.setup()
    render(CreateSecret)

    const submitButton = screen.getByRole('button', { name: /Opprett hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Vennligst skriv inn en hemmelighet')).toBeInTheDocument()
    })
  })

  it('displays result after successful creation', async () => {
    const user = userEvent.setup()
    const mockResponse = {
      id: 'test-uuid-123',
      passphrase: 'word1-word2-word3',
      share_url: 'http://localhost:5173/secret/test-uuid-123',
      expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    }

    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockResponse),
    })

    render(CreateSecret)

    const textarea = screen.getByLabelText(/Hemmelighet/)
    await user.type(textarea, 'My secret message')

    const submitButton = screen.getByRole('button', { name: /Opprett hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Hemmelighet opprettet')).toBeInTheDocument()
    })

    expect(screen.getByText('word1-word2-word3')).toBeInTheDocument()
    expect(screen.getByDisplayValue(mockResponse.share_url)).toBeInTheDocument()
  })

  it('shows error message when API request fails', async () => {
    const user = userEvent.setup()
    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: false,
      status: 500,
    })

    render(CreateSecret)

    const textarea = screen.getByLabelText(/Hemmelighet/)
    await user.type(textarea, 'My secret message')

    const submitButton = screen.getByRole('button', { name: /Opprett hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Kunne ikke opprette hemmelighet')).toBeInTheDocument()
    })
  })

  it('reset button returns to form', async () => {
    const user = userEvent.setup()
    const mockResponse = {
      id: 'test-uuid-123',
      passphrase: 'word1-word2-word3',
      share_url: 'http://localhost:5173/secret/test-uuid-123',
      expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    }

    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockResponse),
    })

    render(CreateSecret)

    // Fill and submit form
    const textarea = screen.getByLabelText(/Hemmelighet/)
    await user.type(textarea, 'My secret message')

    const submitButton = screen.getByRole('button', { name: /Opprett hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Hemmelighet opprettet')).toBeInTheDocument()
    })

    // Click reset button
    const resetButton = screen.getByText('Del en ny hemmelighet')
    await user.click(resetButton)

    // Should be back to form
    expect(screen.getByText('Del en hemmelighet')).toBeInTheDocument()
    expect(screen.getByLabelText(/Hemmelighet/)).toBeInTheDocument()
  })

  it('sends correct payload to API', async () => {
    const user = userEvent.setup()
    const mockResponse = {
      id: 'test-uuid-123',
      passphrase: 'word1-word2-word3',
      share_url: 'http://localhost:5173/secret/test-uuid-123',
      expires_at: new Date().toISOString(),
    }

    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockResponse),
    })

    render(CreateSecret)

    const textarea = screen.getByLabelText(/Hemmelighet/)
    await user.type(textarea, 'Test secret')

    const submitButton = screen.getByRole('button', { name: /Opprett hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/secrets'),
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: expect.any(String),
        })
      )
    })

    const fetchCall = vi.mocked(global.fetch).mock.calls[0]
    const body = JSON.parse(fetchCall[1]?.body as string)
    expect(body.secret).toBe('Test secret')
    expect(body.extendable).toBe(true)
  })
})
