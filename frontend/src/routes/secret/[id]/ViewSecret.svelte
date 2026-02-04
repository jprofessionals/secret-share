<script lang="ts">
  let { id }: { id: string } = $props();

  let passphrase = $state('');
  let loading = $state(false);
  let error = $state('');
  let secret: {
    secret: string;
    views_remaining: number | null;
    extendable: boolean;
    expires_at: string;
  } | null = $state(null);
  let showSecret = $state(false);
  let copySuccess = $state(false);

  // Extension state
  let addDays = $state(0);
  let addViews = $state(0);
  let extending = $state(false);
  let extendError = $state('');
  let extendSuccess = $state('');

  const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

  async function retrieveSecret(e: Event) {
    e.preventDefault();
    if (!passphrase.trim()) {
      error = 'Vennligst skriv inn dekrypteringsnøkkelen';
      return;
    }

    loading = true;
    error = '';

    try {
      const response = await fetch(`${API_URL}/api/secrets/${id}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          passphrase,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Kunne ikke hente hemmelighet');
      }

      const data = await response.json();
      secret = data;
      showSecret = true;
    } catch (err) {
      error = err instanceof Error ? err.message : 'En feil oppstod';
    } finally {
      loading = false;
    }
  }

  async function extendSecret() {
    if (addDays <= 0 && addViews <= 0) {
      extendError = 'Vennligst angi dager eller visninger å legge til';
      return;
    }

    extending = true;
    extendError = '';
    extendSuccess = '';

    try {
      const response = await fetch(`${API_URL}/api/secrets/${id}/extend`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          passphrase,
          add_days: addDays > 0 ? addDays : null,
          add_views: addViews > 0 ? addViews : null,
        }),
      });

      if (!response.ok) {
        if (response.status === 403) {
          extendError = 'Denne hemmeligheten kan ikke forlenges';
        } else if (response.status === 400) {
          extendError = 'Forlengelse overskrider maksimale grenser';
        } else {
          extendError = 'Kunne ikke forlenge hemmelighet';
        }
        return;
      }

      const data = await response.json();
      if (secret) {
        secret.expires_at = data.expires_at;
        secret.views_remaining = data.max_views ? data.max_views - data.views : null;
      }
      extendSuccess = 'Hemmelighet forlenget';
      addDays = 0;
      addViews = 0;
    } catch (err) {
      extendError = 'Nettverksfeil';
    } finally {
      extending = false;
    }
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
    copySuccess = true;
    setTimeout(() => {
      copySuccess = false;
    }, 2000);
  }
</script>

<div class="max-w-3xl mx-auto">
  {#if !showSecret}
    <div class="bg-white rounded-lg border border-gray-200 p-6 sm:p-8 shadow-sm">
      <div class="mb-6 space-y-2">
        <h2 class="text-2xl sm:text-3xl font-bold text-gray-900 tracking-tight">
          Åpne hemmelighet
        </h2>
        <p class="text-sm text-gray-600">
          Skriv inn dekrypteringsnøkkelen for å se hemmeligheten
        </p>
      </div>

      <form onsubmit={retrieveSecret} class="space-y-5">
        <div class="space-y-2">
          <label for="passphrase" class="block text-sm font-medium text-gray-900">
            Dekrypteringsnøkkel (3 ord)
          </label>
          <input
            id="passphrase"
            type="text"
            bind:value={passphrase}
            data-testid="passphrase-input"
            class="block w-full px-3 py-2 text-base border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 text-center font-mono placeholder:text-gray-400"
            placeholder="ord1-ord2-ord3"
          />
          <p class="text-xs text-gray-500 text-center">
            Nøkkelen består av tre ord separert med bindestreker
          </p>
        </div>

        {#if error}
          <div data-testid="error-message" class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md text-sm">
            {error}
          </div>
        {/if}

        <button
          type="submit"
          disabled={loading}
          data-testid="retrieve-button"
          class="w-full inline-flex items-center justify-center px-4 py-2.5 text-sm font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-md transition-colors disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
        >
          {loading ? 'Åpner...' : 'Åpne hemmelighet'}
        </button>
      </form>
    </div>
  {:else if secret}
    <div class="bg-white rounded-lg border border-gray-200 p-8">
      <div class="mb-8">
        <h2 class="text-3xl font-bold text-gray-900 mb-2">
          Hemmelighet hentet
        </h2>
        <div class="space-y-1 text-gray-600">
          {#if secret.views_remaining !== null}
            <p data-testid="views-remaining">
              Gjenværende visninger: <strong>{secret.views_remaining}</strong>
            </p>
          {/if}
          <p data-testid="expires-at">
            Utløper: <strong>{new Date(secret.expires_at).toLocaleString('nb-NO')}</strong>
          </p>
        </div>
      </div>

      <div class="bg-gray-50 border border-gray-300 rounded-lg p-6 mb-6">
        <div class="flex justify-between items-center mb-4">
          <div class="text-sm font-semibold text-gray-700">
            Hemmelighet
          </div>
          <button
            onclick={() => copyToClipboard(secret.secret)}
            data-testid="copy-secret-button"
            class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors text-sm font-medium"
          >
            {copySuccess ? 'Kopiert!' : 'Kopier'}
          </button>
        </div>
        <pre data-testid="secret-content" class="bg-white p-4 rounded-lg border border-gray-200 overflow-x-auto whitespace-pre-wrap break-words font-mono text-sm">{secret.secret}</pre>
      </div>

      <div class="bg-yellow-50 border border-yellow-200 rounded-lg p-5 mb-6">
        <p class="text-yellow-800 text-sm">
          <strong>Viktig:</strong> Denne hemmeligheten vil bli slettet
          {#if secret.views_remaining !== null && secret.views_remaining > 0}
            etter {secret.views_remaining} visning{secret.views_remaining > 1 ? 'er' : ''} til
          {:else}
            permanent etter denne visningen
          {/if}.
          Kopier den hvis du trenger den senere.
        </p>
      </div>

      <!-- Extension section -->
      <div class="border-t border-gray-200 pt-6 mt-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Forleng hemmelighet</h3>

        {#if !secret.extendable}
          <div class="bg-gray-100 p-4 rounded-lg" data-testid="extend-disabled-message">
            <p class="text-gray-500 text-sm">Forlengelse er deaktivert av avsenderen</p>
          </div>
        {:else}
          <div class="space-y-4">
            <div class="grid md:grid-cols-2 gap-4">
              <div>
                <label for="addDays" class="block text-sm font-medium text-gray-700 mb-1">
                  Legg til dager
                </label>
                <input
                  type="number"
                  id="addDays"
                  data-testid="add-days-input"
                  bind:value={addDays}
                  min="0"
                  class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                />
              </div>

              <div>
                <label for="addViews" class="block text-sm font-medium text-gray-700 mb-1">
                  Legg til visninger
                </label>
                <input
                  type="number"
                  id="addViews"
                  data-testid="add-views-input"
                  bind:value={addViews}
                  min="0"
                  class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500"
                />
              </div>
            </div>

            {#if extendError}
              <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-md text-sm">
                {extendError}
              </div>
            {/if}

            {#if extendSuccess}
              <div data-testid="extend-success-message" class="bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-md text-sm">
                {extendSuccess}
              </div>
            {/if}

            <button
              onclick={extendSecret}
              data-testid="extend-button"
              disabled={extending || (addDays <= 0 && addViews <= 0)}
              class="w-full px-4 py-2.5 bg-indigo-600 text-white rounded-md font-semibold hover:bg-indigo-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {extending ? 'Forlenger...' : 'Forleng hemmelighet'}
            </button>
          </div>
        {/if}
      </div>

      <div class="text-center mt-8">
        <a
          href="/create"
          class="text-indigo-600 hover:text-indigo-800 font-semibold"
        >
          ← Del en ny hemmelighet
        </a>
      </div>
    </div>
  {/if}
</div>
