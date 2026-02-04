import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/svelte'
import userEvent from '@testing-library/user-event'
import ViewSecret from './ViewSecret.svelte'

describe('ViewSecret', () => {
  const testId = 'test-secret-id-123'

  beforeEach(() => {
    vi.resetAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('renders passphrase input initially', () => {
    render(ViewSecret, { props: { id: testId } })

    expect(screen.getByRole('heading', { name: 'Åpne hemmelighet' })).toBeInTheDocument()
    expect(screen.getByLabelText(/Dekrypteringsnøkkel/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Åpne hemmelighet/ })).toBeInTheDocument()
  })

  it('shows error for empty passphrase', async () => {
    const user = userEvent.setup()
    render(ViewSecret, { props: { id: testId } })

    const submitButton = screen.getByRole('button', { name: /Åpne hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Vennligst skriv inn dekrypteringsnøkkelen')).toBeInTheDocument()
    })
  })

  it('displays secret after successful retrieval', async () => {
    const user = userEvent.setup()
    const mockResponse = {
      secret: 'This is the decrypted secret',
      views_remaining: 4,
      extendable: true,
      expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    }

    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockResponse),
    })

    render(ViewSecret, { props: { id: testId } })

    const passphraseInput = screen.getByLabelText(/Dekrypteringsnøkkel/)
    await user.type(passphraseInput, 'word1-word2-word3')

    const submitButton = screen.getByRole('button', { name: /Åpne hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Hemmelighet hentet')).toBeInTheDocument()
    })

    expect(screen.getByText('This is the decrypted secret')).toBeInTheDocument()
    expect(screen.getByText('4')).toBeInTheDocument()
  })

  it('shows error for invalid passphrase', async () => {
    const user = userEvent.setup()
    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: false,
      json: () => Promise.resolve({ error: 'Invalid passphrase' }),
    })

    render(ViewSecret, { props: { id: testId } })

    const passphraseInput = screen.getByLabelText(/Dekrypteringsnøkkel/)
    await user.type(passphraseInput, 'wrong-passphrase-here')

    const submitButton = screen.getByRole('button', { name: /Åpne hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Invalid passphrase')).toBeInTheDocument()
    })
  })

  it('shows error for not found secret', async () => {
    const user = userEvent.setup()
    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: false,
      json: () => Promise.resolve({ error: 'Secret not found' }),
    })

    render(ViewSecret, { props: { id: testId } })

    const passphraseInput = screen.getByLabelText(/Dekrypteringsnøkkel/)
    await user.type(passphraseInput, 'any-passphrase-here')

    const submitButton = screen.getByRole('button', { name: /Åpne hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Secret not found')).toBeInTheDocument()
    })
  })

  it('shows warning when no views remaining', async () => {
    const user = userEvent.setup()
    const mockResponse = {
      secret: 'Last view secret',
      views_remaining: 0,
      extendable: true,
      expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    }

    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve(mockResponse),
    })

    render(ViewSecret, { props: { id: testId } })

    const passphraseInput = screen.getByLabelText(/Dekrypteringsnøkkel/)
    await user.type(passphraseInput, 'word1-word2-word3')

    const submitButton = screen.getByRole('button', { name: /Åpne hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(screen.getByText('Hemmelighet hentet')).toBeInTheDocument()
    })

    expect(screen.getByText(/permanent etter denne visningen/)).toBeInTheDocument()
  })

  it('sends correct API request', async () => {
    const user = userEvent.setup()
    global.fetch = vi.fn().mockResolvedValueOnce({
      ok: true,
      json: () => Promise.resolve({
        secret: 'test',
        views_remaining: 1,
        extendable: true,
        expires_at: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
      }),
    })

    render(ViewSecret, { props: { id: testId } })

    const passphraseInput = screen.getByLabelText(/Dekrypteringsnøkkel/)
    await user.type(passphraseInput, 'my-passphrase-here')

    const submitButton = screen.getByRole('button', { name: /Åpne hemmelighet/ })
    await user.click(submitButton)

    await waitFor(() => {
      expect(global.fetch).toHaveBeenCalledWith(
        expect.stringContaining(`/api/secrets/${testId}`),
        expect.objectContaining({
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
        })
      )
    })

    const fetchCall = vi.mocked(global.fetch).mock.calls[0]
    const body = JSON.parse(fetchCall[1]?.body as string)
    expect(body.passphrase).toBe('my-passphrase-here')
  })
})
