export default {
  // App
  'app.title': 'SecretShare - Secure secret sharing',

  // Layout - Nav
  'layout.nav.home': 'Home',
  'layout.nav.share': 'Share secret',
  'layout.nav.confirmNewSecret': 'Leave this page and create a new secret?',

  // Layout - Footer
  'layout.footer.tagline': 'End-to-end encrypted • Self-destructing • Open source',

  // Home - Hero
  'home.hero.title': 'Share secrets securely',
  'home.hero.subtitle':
    'End-to-end encrypted sharing of passwords, API keys and sensitive information.',

  // Home - Features
  'home.features.encryption.title': 'End-to-end encrypted',
  'home.features.encryption.description':
    'Secrets are encrypted in your browser with AES-256-GCM before being sent to the server.',
  'home.features.selfDestruct.title': 'Self-destructing',
  'home.features.selfDestruct.description':
    'Set an expiry time or maximum number of views. The secret is deleted automatically.',
  // Home - CTA
  'home.cta.button': 'Share a secret now',
  'home.cta.subtitle': 'No registration required • Free • Open source',

  // Home - How it works
  'home.howItWorks.title': 'How it works',
  'home.howItWorks.step1.title': 'Create a secret',
  'home.howItWorks.step1.description':
    'Enter the information you want to share and choose security settings. Everything is encrypted locally in your browser.',
  'home.howItWorks.step2.title': 'Get sharing link and key',
  'home.howItWorks.step2.description':
    'You receive a unique link and a 3-word decryption key. Both are needed to open the secret.',
  'home.howItWorks.step3.title': 'Share via different channels',
  'home.howItWorks.step3.description':
    'Send the link via one channel (e.g. Slack) and the key via another (e.g. SMS) for maximum security.',
  'home.howItWorks.step4.title': 'Recipient opens the secret',
  'home.howItWorks.step4.description':
    'With the link and key, the secret is decrypted locally by the recipient. Then it is permanently deleted.',

  // Create - Form
  'create.form.title': 'Share a secret',
  'create.form.subtitle': 'Encryption happens locally in your browser',
  'create.form.secretLabel': 'Secret *',
  'create.form.secretPlaceholder':
    'Enter a password, API key or other sensitive information...',
  'create.form.maxViewsLabel': 'Maximum number of views',
  'create.form.expiresLabel': 'Expires in',
  'create.form.expires.1h': '1 hour',
  'create.form.expires.6h': '6 hours',
  'create.form.expires.24h': '24 hours',
  'create.form.expires.3d': '3 days',
  'create.form.expires.7d': '7 days',
  'create.form.extendableLabel': 'Allow recipient to extend',
  'create.form.extendableHint':
    'When enabled, the recipient can extend the expiry time and view limit',
  'create.form.submitButton': 'Create secret',
  'create.form.submitting': 'Creating...',
  'create.form.errorEmpty': 'Please enter a secret',
  'create.form.errorCreate': 'Could not create secret',
  'create.form.errorGeneric': 'An error occurred',

  // Create - Result
  'create.result.title': 'Secret created',
  'create.result.subtitle':
    'Share the link and key via different channels for maximum security',
  'create.result.passphraseLabel': 'Decryption key',
  'create.result.passphraseHint':
    'Share this key via a different channel than the link below',
  'create.result.shareLinkLabel': 'Share link',
  'create.result.expires': 'Expires:',
  'create.result.copyButton': 'Copy',
  'create.result.copied': 'Copied!',
  'create.result.newSecret': 'Share a new secret',

  // View - Form
  'view.form.title': 'Open secret',
  'view.form.subtitle': 'Enter the decryption key to view the secret',
  'view.form.passphraseLabel': 'Decryption key (3 words)',
  'view.form.passphrasePlaceholder': 'word1-word2-word3',
  'view.form.passphraseHint':
    'The key consists of three words separated by hyphens',
  'view.form.submitButton': 'Open secret',
  'view.form.submitting': 'Opening...',
  'view.form.errorEmpty': 'Please enter the decryption key',
  'view.form.errorRetrieve': 'Could not retrieve secret',
  'view.form.errorGeneric': 'An error occurred',

  // View - Result
  'view.result.title': 'Secret retrieved',
  'view.result.viewsRemaining': 'Views remaining:',
  'view.result.expires': 'Expires:',
  'view.result.secretLabel': 'Secret',
  'view.result.copyButton': 'Copy',
  'view.result.copied': 'Copied!',
  'view.result.warningImportant': 'Important:',
  'view.result.warningDelete': 'This secret will be deleted',
  'view.result.warningViewsLeft': 'after {count} more view(s)',
  'view.result.warningLastView': 'permanently after this view',
  'view.result.warningCopy': 'Copy it if you need it later.',
  'view.result.newSecret': '← Share a new secret',

  // View - Extend
  'view.extend.title': 'Extend secret',
  'view.extend.disabled': 'Extension is disabled by the sender',
  'view.extend.addDays': 'Add days',
  'view.extend.addViews': 'Add views',
  'view.extend.submitButton': 'Extend secret',
  'view.extend.submitting': 'Extending...',
  'view.extend.errorEmpty': 'Please specify days or views to add',
  'view.extend.errorForbidden': 'This secret cannot be extended',
  'view.extend.errorLimit': 'Extension exceeds maximum limits',
  'view.extend.errorGeneric': 'Could not extend secret',
  'view.extend.errorNetwork': 'Network error',
  'view.extend.success': 'Secret extended',
} as const;
