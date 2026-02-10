export default {
  // App
  'app.title': 'SecretShare - Sikker deling av hemmeligheter',

  // Layout - Nav
  'layout.nav.home': 'Hjem',
  'layout.nav.share': 'Del hemmelighet',

  // Layout - Footer
  'layout.footer.tagline': 'End-to-end kryptert • Selvdestruerende • Åpen kildekode',

  // Home - Hero
  'home.hero.title': 'Del hemmeligheter sikkert',
  'home.hero.subtitle':
    'End-to-end kryptert deling av passord, API-nøkler og sensitiv informasjon.',

  // Home - Features
  'home.features.encryption.title': 'End-to-end kryptert',
  'home.features.encryption.description':
    'Hemmeligheter krypteres i nettleseren din med AES-256-GCM før de sendes til serveren.',
  'home.features.selfDestruct.title': 'Selvdestruerende',
  'home.features.selfDestruct.description':
    'Sett en utløpstid eller maksimalt antall visninger. Hemmeligheten slettes automatisk.',
  // Home - CTA
  'home.cta.button': 'Del en hemmelighet nå',
  'home.cta.subtitle': 'Ingen registrering kreves • Gratis • Open source',

  // Home - How it works
  'home.howItWorks.title': 'Hvordan det fungerer',
  'home.howItWorks.step1.title': 'Opprett hemmelighet',
  'home.howItWorks.step1.description':
    'Skriv inn informasjonen du vil dele og velg innstillinger for sikkerhet. Alt krypteres lokalt i nettleseren din.',
  'home.howItWorks.step2.title': 'Få delingslenke og nøkkel',
  'home.howItWorks.step2.description':
    'Du får en unik lenke og en 3-ords dekrypteringsnøkkel. Begge deler trengs for å åpne hemmeligheten.',
  'home.howItWorks.step3.title': 'Del på forskjellige kanaler',
  'home.howItWorks.step3.description':
    'Send lenken via én kanal (f.eks. Slack) og nøkkelen via en annen (f.eks. SMS) for maksimal sikkerhet.',
  'home.howItWorks.step4.title': 'Mottakeren åpner hemmeligheten',
  'home.howItWorks.step4.description':
    'Med lenken og nøkkelen dekrypteres hemmeligheten lokalt hos mottakeren. Deretter slettes den permanent.',

  // Create - Form
  'create.form.title': 'Del en hemmelighet',
  'create.form.subtitle': 'Kryptering skjer lokalt i nettleseren din',
  'create.form.secretLabel': 'Hemmelighet *',
  'create.form.secretPlaceholder':
    'Skriv inn passord, API-nøkkel eller annen sensitiv informasjon...',
  'create.form.maxViewsLabel': 'Maksimalt antall visninger',
  'create.form.expiresLabel': 'Utløper om',
  'create.form.expires.1h': '1 time',
  'create.form.expires.6h': '6 timer',
  'create.form.expires.24h': '24 timer',
  'create.form.expires.3d': '3 dager',
  'create.form.expires.7d': '7 dager',
  'create.form.extendableLabel': 'Tillat mottaker å forlenge',
  'create.form.extendableHint':
    'Når aktivert kan mottaker forlenge utløpstid og visningsgrense',
  'create.form.submitButton': 'Opprett hemmelighet',
  'create.form.submitting': 'Oppretter...',
  'create.form.errorEmpty': 'Vennligst skriv inn en hemmelighet',
  'create.form.errorCreate': 'Kunne ikke opprette hemmelighet',
  'create.form.errorGeneric': 'En feil oppstod',

  // Create - Result
  'create.result.title': 'Hemmelighet opprettet',
  'create.result.subtitle':
    'Del lenken og nøkkelen via forskjellige kanaler for maksimal sikkerhet',
  'create.result.passphraseLabel': 'Dekrypteringsnøkkel',
  'create.result.passphraseHint':
    'Del denne nøkkelen via en annen kanal enn lenken nedenfor',
  'create.result.shareLinkLabel': 'Delingslenke',
  'create.result.expires': 'Utløper:',
  'create.result.copyButton': 'Kopier',
  'create.result.copied': 'Kopiert!',
  'create.result.newSecret': 'Del en ny hemmelighet',

  // View - Form
  'view.form.title': 'Åpne hemmelighet',
  'view.form.subtitle': 'Skriv inn dekrypteringsnøkkelen for å se hemmeligheten',
  'view.form.passphraseLabel': 'Dekrypteringsnøkkel (3 ord)',
  'view.form.passphrasePlaceholder': 'ord1-ord2-ord3',
  'view.form.passphraseHint':
    'Nøkkelen består av tre ord separert med bindestreker',
  'view.form.submitButton': 'Åpne hemmelighet',
  'view.form.submitting': 'Åpner...',
  'view.form.errorEmpty': 'Vennligst skriv inn dekrypteringsnøkkelen',
  'view.form.errorRetrieve': 'Kunne ikke hente hemmelighet',
  'view.form.errorGeneric': 'En feil oppstod',

  // View - Result
  'view.result.title': 'Hemmelighet hentet',
  'view.result.viewsRemaining': 'Gjenværende visninger:',
  'view.result.expires': 'Utløper:',
  'view.result.secretLabel': 'Hemmelighet',
  'view.result.copyButton': 'Kopier',
  'view.result.copied': 'Kopiert!',
  'view.result.warningImportant': 'Viktig:',
  'view.result.warningDelete': 'Denne hemmeligheten vil bli slettet',
  'view.result.warningViewsLeft': 'etter {count} visning(er) til',
  'view.result.warningLastView': 'permanent etter denne visningen',
  'view.result.warningCopy': 'Kopier den hvis du trenger den senere.',
  'view.result.newSecret': '← Del en ny hemmelighet',

  // View - Extend
  'view.extend.title': 'Forleng hemmelighet',
  'view.extend.disabled': 'Forlengelse er deaktivert av avsenderen',
  'view.extend.addDays': 'Legg til dager',
  'view.extend.addViews': 'Legg til visninger',
  'view.extend.submitButton': 'Forleng hemmelighet',
  'view.extend.submitting': 'Forlenger...',
  'view.extend.errorEmpty': 'Vennligst angi dager eller visninger å legge til',
  'view.extend.errorForbidden': 'Denne hemmeligheten kan ikke forlenges',
  'view.extend.errorLimit': 'Forlengelse overskrider maksimale grenser',
  'view.extend.errorGeneric': 'Kunne ikke forlenge hemmelighet',
  'view.extend.errorNetwork': 'Nettverksfeil',
  'view.extend.success': 'Hemmelighet forlenget',
} as const;
