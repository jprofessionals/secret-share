<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { t, initI18n } from '$lib/i18n/index.svelte';
  import LanguageSwitcher from '$lib/components/LanguageSwitcher.svelte';
  import { page } from '$app/state';

  let { children } = $props();

  onMount(() => {
    initI18n();
  });

  function handleShareClick(e: MouseEvent) {
    if (page.url.pathname === '/create') {
      e.preventDefault();
      if (confirm(t('layout.nav.confirmNewSecret'))) {
        window.location.href = '/create';
      }
    }
  }
</script>

<svelte:head>
  <title>{t('app.title')}</title>
</svelte:head>

<div class="min-h-screen bg-gray-50 flex flex-col antialiased">
  <nav class="bg-white border-b border-gray-200 sticky top-0 z-50 shadow-sm">
    <div class="max-w-7xl mx-auto px-6 lg:px-12">
      <div class="flex items-center justify-between h-16">
        <a href="/" class="flex items-center space-x-2 hover:opacity-80 transition-opacity">
          <span class="text-2xl">🔐</span>
          <h1 class="text-xl font-bold text-gray-900 tracking-tight">
            SecretShare
          </h1>
        </a>
        <div class="flex items-center space-x-1">
          <LanguageSwitcher />
          <a
            href="/"
            class="px-4 py-2 text-sm font-medium text-gray-700 hover:text-gray-900 hover:bg-gray-100 rounded-md transition-colors"
          >
            {t('layout.nav.home')}
          </a>
          <a
            href="/create"
            onclick={handleShareClick}
            class="px-4 py-2 text-sm font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-md transition-colors shadow-sm"
          >
            {t('layout.nav.share')}
          </a>
        </div>
      </div>
    </div>
  </nav>

  <main class="flex-1 w-full max-w-7xl mx-auto px-6 lg:px-12 py-8 sm:py-12">
    {@render children()}
  </main>

  <footer class="mt-auto py-6 bg-white border-t border-gray-200">
    <div class="max-w-7xl mx-auto px-6 lg:px-12">
      <div class="text-center space-y-1">
        <p class="text-sm font-semibold text-gray-900">SecretShare</p>
        <p class="text-xs text-gray-600">
          {t('layout.footer.tagline')}
        </p>
      </div>
    </div>
  </footer>
</div>
