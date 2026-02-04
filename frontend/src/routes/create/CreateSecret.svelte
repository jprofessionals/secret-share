<script lang="ts">
  let secret = $state('');
  let maxViews = $state(1);
  let expiresInHours = $state(24);
  let extendable = $state(true);
  let loading = $state(false);
  let result: {
    id: string;
    passphrase: string;
    share_url: string;
    expires_at: string;
  } | null = $state(null);
  let error = $state('');
  let copySuccess = $state('');

  const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:3000';

  async function createSecret(e: Event) {
    e.preventDefault();
    if (!secret.trim()) {
      error = 'Vennligst skriv inn en hemmelighet';
      return;
    }

    loading = true;
    error = '';

    try {
      const response = await fetch(`${API_URL}/api/secrets`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          secret,
          max_views: maxViews > 0 ? maxViews : null,
          expires_in_hours: expiresInHours,
          extendable,
        }),
      });

      if (!response.ok) {
        throw new Error('Kunne ikke opprette hemmelighet');
      }

      result = await response.json();

      // Clear form
      secret = '';
    } catch (err) {
      error = err instanceof Error ? err.message : 'En feil oppstod';
    } finally {
      loading = false;
    }
  }

  function copyToClipboard(text: string, type: string) {
    navigator.clipboard.writeText(text);
    copySuccess = type;
    setTimeout(() => {
      copySuccess = '';
    }, 2000);
  }

  function reset() {
    result = null;
    error = '';
  }
</script>


<div class="max-w-3xl mx-auto">
  {#if !result}
    <div class="bg-white rounded-lg border border-gray-200 p-6 sm:p-8 shadow-sm">
      <div class="mb-6 space-y-2">
        <h2 class="text-2xl sm:text-3xl font-bold text-gray-900 tracking-tight">
          Del en hemmelighet
        </h2>
        <p class="text-sm text-gray-600">Kryptering skjer lokalt i nettleseren din</p>
      </div>

      <form onsubmit={createSecret} class="space-y-5">
        <div>
          <label for="secret" class="block text-sm font-semibold text-gray-700 mb-2">
            Hemmelighet *
          </label>
          <textarea
            id="secret"
            data-testid="secret-input"
            bind:value={secret}
            rows={6}
            class="w-full px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-gray-300 font-mono text-sm"
            placeholder="Skriv inn passord, API-nokkel eller annen sensitiv informasjon..."
          ></textarea>
        </div>

        <div class="grid md:grid-cols-2 gap-4">
          <div>
            <label for="maxViews" class="block text-sm font-semibold text-gray-700 mb-2">
              Maksimalt antall visninger
            </label>
            <input
              id="maxViews"
              data-testid="max-views-input"
              type="number"
              bind:value={maxViews}
              min="1"
              max="100"
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-gray-300"
            />
          </div>

          <div>
            <label for="expiresIn" class="block text-sm font-semibold text-gray-700 mb-2">
              Utloper om
            </label>
            <select
              id="expiresIn"
              data-testid="expires-select"
              bind:value={expiresInHours}
              class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-gray-300"
            >
              <option value={1}>1 time</option>
              <option value={6}>6 timer</option>
              <option value={24}>24 timer</option>
              <option value={72}>3 dager</option>
              <option value={168}>7 dager</option>
            </select>
          </div>
        </div>

        <div class="border-t border-gray-200 pt-5">
          <label class="flex items-center">
            <input
              type="checkbox"
              data-testid="extendable-checkbox"
              bind:checked={extendable}
              class="w-4 h-4 text-indigo-600 border-gray-300 rounded focus:ring-indigo-500"
            />
            <span class="ml-2 text-sm text-gray-700">Tillat mottaker a forlenge</span>
          </label>
          <p class="text-xs text-gray-500 mt-1 ml-6">
            Nar aktivert kan mottaker forlenge utlopstid og visningsgrense
          </p>
        </div>

        {#if error}
          <div data-testid="error-message" class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
            {error}
          </div>
        {/if}

        <button
          type="submit"
          data-testid="submit-button"
          disabled={loading}
          class="w-full bg-indigo-600 text-white px-6 py-3 rounded-lg font-semibold hover:bg-indigo-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {loading ? 'Oppretter...' : 'Opprett hemmelighet'}
        </button>
      </form>
    </div>
  {:else}
    <div class="bg-white rounded-lg border border-gray-200 p-8">
      <div class="mb-8">
        <h2 class="text-3xl font-bold text-gray-900 mb-2">
          Hemmelighet opprettet
        </h2>
        <p class="text-gray-600">
          Del lenken og nokkelen via forskjellige kanaler for maksimal sikkerhet
        </p>
      </div>

      <div class="space-y-6">
        <div class="bg-yellow-50 border border-yellow-300 rounded-lg p-5">
          <div class="block text-sm font-semibold text-gray-900 mb-3">
            Dekrypteringsnokkel
          </div>
          <div class="flex items-center gap-2">
            <code data-testid="passphrase-display" class="flex-1 bg-white px-4 py-3 rounded-lg border border-yellow-400 text-lg font-mono font-semibold text-gray-900 select-all">
              {result.passphrase}
            </code>
            <button
              data-testid="copy-passphrase-button"
              onclick={() => copyToClipboard(result.passphrase, 'passphrase')}
              class="px-4 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium whitespace-nowrap"
            >
              {copySuccess === 'passphrase' ? 'Kopiert!' : 'Kopier'}
            </button>
          </div>
          <p class="text-sm text-yellow-800 mt-3">
            Del denne nokkelen via en annen kanal enn lenken nedenfor
          </p>
        </div>

        <div class="bg-blue-50 border border-blue-300 rounded-lg p-5">
          <div class="block text-sm font-semibold text-gray-900 mb-3">
            Delingslenke
          </div>
          <div class="flex items-center gap-2">
            <input
              type="text"
              data-testid="share-url-display"
              value={result.share_url}
              readonly
              class="flex-1 bg-white px-4 py-3 rounded-lg border border-blue-400 text-sm font-mono select-all"
            />
            <button
              data-testid="copy-url-button"
              onclick={() => copyToClipboard(result.share_url, 'url')}
              class="px-4 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors font-medium whitespace-nowrap"
            >
              {copySuccess === 'url' ? 'Kopiert!' : 'Kopier'}
            </button>
          </div>
        </div>

        <div class="bg-gray-50 border border-gray-200 rounded-lg p-5">
          <p class="text-sm text-gray-600">
            <strong>Utloper:</strong> {new Date(result.expires_at).toLocaleString('nb-NO')}
          </p>
        </div>
      </div>

      <div class="mt-8 text-center">
        <button
          onclick={reset}
          class="text-indigo-600 hover:text-indigo-800 font-semibold"
        >
          Del en ny hemmelighet
        </button>
      </div>
    </div>
  {/if}
</div>
