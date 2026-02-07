import nb from './translations/nb';
import en from './translations/en';

export type Locale = 'nb' | 'en';
type TranslationKey = keyof typeof nb;

const translations: Record<Locale, Record<string, string>> = { nb, en };

let locale = $state<Locale>('nb');

export function getLocale(): Locale {
  return locale;
}

export function setLocale(newLocale: Locale): void {
  locale = newLocale;
  try {
    localStorage.setItem('locale', newLocale);
  } catch {
    // localStorage unavailable
  }
  if (typeof document !== 'undefined') {
    document.documentElement.lang = newLocale;
  }
}

export function getDateLocale(): string {
  return locale === 'nb' ? 'nb-NO' : 'en-GB';
}

export function initI18n(): void {
  try {
    const stored = localStorage.getItem('locale');
    if (stored === 'nb' || stored === 'en') {
      locale = stored;
    } else if (typeof navigator !== 'undefined') {
      const browserLang = navigator.language.slice(0, 2);
      locale = browserLang === 'en' ? 'en' : 'nb';
    }
  } catch {
    // localStorage unavailable
  }
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale;
  }
}

export function t(key: TranslationKey, params?: Record<string, string | number>): string {
  const str = translations[locale]?.[key] ?? translations['nb'][key] ?? key;
  if (!params) return str;
  return str.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? `{${k}}`));
}
