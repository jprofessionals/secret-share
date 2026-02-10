<script lang="ts">
  import { goto } from '$app/navigation';
  import { t } from '$lib/i18n/index.svelte';
</script>

<div class="max-w-6xl mx-auto space-y-16">
  <!-- Hero Section -->
  <div class="text-center space-y-6">
    <div class="text-5xl sm:text-6xl">🔐</div>
    <h2 class="text-4xl sm:text-5xl md:text-6xl font-extrabold text-gray-900 tracking-tight">
      {t('home.hero.title')}
    </h2>
    <p class="text-lg sm:text-xl text-gray-600 max-w-2xl mx-auto">
      {t('home.hero.subtitle')}
    </p>
  </div>

  <!-- Feature Cards -->
  <div class="grid sm:grid-cols-2 gap-6 max-w-3xl mx-auto">
    <div class="bg-white p-6 rounded-lg border border-gray-200 hover:border-gray-300 transition-colors">
      <div class="text-3xl mb-3">🔒</div>
      <h3 class="text-base font-semibold mb-2 text-gray-900">{t('home.features.encryption.title')}</h3>
      <p class="text-sm text-gray-600 leading-relaxed">
        {t('home.features.encryption.description')}
      </p>
    </div>

    <div class="bg-white p-6 rounded-lg border border-gray-200 hover:border-gray-300 transition-colors">
      <div class="text-3xl mb-3">⏱️</div>
      <h3 class="text-base font-semibold mb-2 text-gray-900">{t('home.features.selfDestruct.title')}</h3>
      <p class="text-sm text-gray-600 leading-relaxed">
        {t('home.features.selfDestruct.description')}
      </p>
    </div>

  </div>

  <!-- CTA Button -->
  <div class="text-center space-y-3">
    <button
      onclick={() => goto('/create')}
      class="inline-flex items-center justify-center px-6 py-3 text-base font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition-colors shadow-sm"
    >
      {t('home.cta.button')}
    </button>
    <p class="text-xs text-gray-500">{t('home.cta.subtitle')}</p>
  </div>

  <!-- How It Works Section -->
  <div class="bg-white p-8 sm:p-10 rounded-lg border border-gray-200 space-y-6">
    <h3 class="text-xl sm:text-2xl font-bold text-gray-900">
      {t('home.howItWorks.title')}
    </h3>
    <div class="space-y-5">
      <div class="flex items-start gap-4">
        <span class="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">1</span>
        <div class="flex-1 space-y-1">
          <h4 class="text-sm font-semibold text-gray-900">{t('home.howItWorks.step1.title')}</h4>
          <p class="text-sm text-gray-600">{t('home.howItWorks.step1.description')}</p>
        </div>
      </div>
      <div class="flex items-start gap-4">
        <span class="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">2</span>
        <div class="flex-1 space-y-1">
          <h4 class="text-sm font-semibold text-gray-900">{t('home.howItWorks.step2.title')}</h4>
          <p class="text-sm text-gray-600">{t('home.howItWorks.step2.description')}</p>
        </div>
      </div>
      <div class="flex items-start gap-4">
        <span class="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">3</span>
        <div class="flex-1 space-y-1">
          <h4 class="text-sm font-semibold text-gray-900">{t('home.howItWorks.step3.title')}</h4>
          <p class="text-sm text-gray-600">{t('home.howItWorks.step3.description')}</p>
        </div>
      </div>
      <div class="flex items-start gap-4">
        <span class="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">4</span>
        <div class="flex-1 space-y-1">
          <h4 class="text-sm font-semibold text-gray-900">{t('home.howItWorks.step4.title')}</h4>
          <p class="text-sm text-gray-600">{t('home.howItWorks.step4.description')}</p>
        </div>
      </div>
    </div>
  </div>
</div>
