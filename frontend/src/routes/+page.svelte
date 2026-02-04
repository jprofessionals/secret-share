<script lang="ts">
  import { goto } from '$app/navigation';
</script>

<div class="max-w-6xl mx-auto space-y-16">
  <!-- Hero Section -->
  <div class="text-center space-y-6">
    <div class="text-5xl sm:text-6xl">🔐</div>
    <h2 class="text-4xl sm:text-5xl md:text-6xl font-extrabold text-gray-900 tracking-tight">
      Del hemmeligheter sikkert
    </h2>
    <p class="text-lg sm:text-xl text-gray-600 max-w-2xl mx-auto">
      End-to-end kryptert deling av passord, API-nøkler og sensitiv informasjon.
    </p>
  </div>

  <!-- Feature Cards -->
  <div class="grid sm:grid-cols-2 lg:grid-cols-3 gap-6">
    <div class="bg-white p-6 rounded-lg border border-gray-200 hover:border-gray-300 transition-colors">
      <div class="text-3xl mb-3">🔒</div>
      <h3 class="text-base font-semibold mb-2 text-gray-900">End-to-end kryptert</h3>
      <p class="text-sm text-gray-600 leading-relaxed">
        Hemmeligheter krypteres i nettleseren din med AES-256-GCM før de sendes til serveren.
      </p>
    </div>

    <div class="bg-white p-6 rounded-lg border border-gray-200 hover:border-gray-300 transition-colors">
      <div class="text-3xl mb-3">⏱️</div>
      <h3 class="text-base font-semibold mb-2 text-gray-900">Selvdestruerende</h3>
      <p class="text-sm text-gray-600 leading-relaxed">
        Sett en utløpstid eller maksimalt antall visninger. Hemmeligheten slettes automatisk.
      </p>
    </div>

    <div class="bg-white p-6 rounded-lg border border-gray-200 hover:border-gray-300 transition-colors sm:col-span-2 lg:col-span-1">
      <div class="text-3xl mb-3">🛡️</div>
      <h3 class="text-base font-semibold mb-2 text-gray-900">2-faktor autentisering</h3>
      <p class="text-sm text-gray-600 leading-relaxed">
        Ekstra sikkerhetslag med SMS eller e-post verifisering før hemmeligheten kan åpnes.
      </p>
    </div>
  </div>

  <!-- CTA Button -->
  <div class="text-center space-y-3">
    <button
      onclick={() => goto('/create')}
      class="inline-flex items-center justify-center px-6 py-3 text-base font-semibold text-white bg-indigo-600 hover:bg-indigo-700 rounded-lg transition-colors shadow-sm"
    >
      Del en hemmelighet nå
    </button>
    <p class="text-xs text-gray-500">Ingen registrering kreves • Gratis • Open source</p>
  </div>

  <!-- How It Works Section -->
  <div class="bg-white p-8 sm:p-10 rounded-lg border border-gray-200 space-y-6">
    <h3 class="text-xl sm:text-2xl font-bold text-gray-900">
      Hvordan det fungerer
    </h3>
    <div class="space-y-5">
      <div class="flex items-start gap-4">
        <span class="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">1</span>
        <div class="flex-1 space-y-1">
          <h4 class="text-sm font-semibold text-gray-900">Opprett hemmelighet</h4>
          <p class="text-sm text-gray-600">Skriv inn informasjonen du vil dele og velg innstillinger for sikkerhet. Alt krypteres lokalt i nettleseren din.</p>
        </div>
      </div>
      <div class="flex items-start gap-4">
        <span class="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">2</span>
        <div class="flex-1 space-y-1">
          <h4 class="text-sm font-semibold text-gray-900">Få delingslenke og nøkkel</h4>
          <p class="text-sm text-gray-600">Du får en unik lenke og en 3-ords dekrypteringsnøkkel. Begge deler trengs for å åpne hemmeligheten.</p>
        </div>
      </div>
      <div class="flex items-start gap-4">
        <span class="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">3</span>
        <div class="flex-1 space-y-1">
          <h4 class="text-sm font-semibold text-gray-900">Del på forskjellige kanaler</h4>
          <p class="text-sm text-gray-600">Send lenken via én kanal (f.eks. Slack) og nøkkelen via en annen (f.eks. SMS) for maksimal sikkerhet.</p>
        </div>
      </div>
      <div class="flex items-start gap-4">
        <span class="flex-shrink-0 w-8 h-8 bg-indigo-600 text-white rounded-full flex items-center justify-center text-sm font-semibold">4</span>
        <div class="flex-1 space-y-1">
          <h4 class="text-sm font-semibold text-gray-900">Mottakeren åpner hemmeligheten</h4>
          <p class="text-sm text-gray-600">Med lenken og nøkkelen dekrypteres hemmeligheten lokalt hos mottakeren. Deretter slettes den permanent.</p>
        </div>
      </div>
    </div>
  </div>
</div>
