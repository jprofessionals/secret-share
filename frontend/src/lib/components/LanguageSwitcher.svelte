<script lang="ts">
  import { getLocale, setLocale } from '$lib/i18n/index.svelte';

  function toggle() {
    setLocale(getLocale() === 'nb' ? 'en' : 'nb');
  }
</script>

<button
  onclick={toggle}
  class="px-3 py-1.5 text-xs font-semibold text-gray-600 hover:text-gray-900 bg-gray-100 hover:bg-gray-200 rounded-md transition-colors"
  aria-label="Switch language"
>
  {getLocale() === 'nb' ? 'EN' : 'NB'}
</button>
