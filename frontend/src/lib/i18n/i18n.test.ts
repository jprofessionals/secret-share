import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest';
import { t, getLocale, setLocale, getDateLocale, initI18n } from './index.svelte';

// Mock localStorage since jsdom/node may not provide it properly
const localStorageStore: Record<string, string> = {};
const localStorageMock = {
  getItem: vi.fn((key: string) => localStorageStore[key] ?? null),
  setItem: vi.fn((key: string, value: string) => {
    localStorageStore[key] = value;
  }),
  removeItem: vi.fn((key: string) => {
    delete localStorageStore[key];
  }),
  clear: vi.fn(() => {
    for (const key of Object.keys(localStorageStore)) {
      delete localStorageStore[key];
    }
  }),
  get length() {
    return Object.keys(localStorageStore).length;
  },
  key: vi.fn((index: number) => Object.keys(localStorageStore)[index] ?? null),
};

Object.defineProperty(globalThis, 'localStorage', {
  value: localStorageMock,
  writable: true,
});

describe('i18n', () => {
  beforeEach(() => {
    setLocale('nb');
    localStorageMock.clear();
    vi.clearAllMocks();
  });

  describe('t()', () => {
    it('returns Norwegian translation by default', () => {
      expect(t('layout.nav.home')).toBe('Hjem');
    });

    it('returns English translation when locale is en', () => {
      setLocale('en');
      expect(t('layout.nav.home')).toBe('Home');
    });

    it('returns key if translation not found', () => {
      expect(t('nonexistent.key' as any)).toBe('nonexistent.key');
    });

    it('interpolates parameters', () => {
      setLocale('en');
      expect(t('view.result.warningViewsLeft', { count: 3 })).toBe(
        'after 3 more view(s)'
      );
    });

    it('leaves unmatched placeholders intact', () => {
      setLocale('en');
      expect(t('view.result.warningViewsLeft', {})).toBe(
        'after {count} more view(s)'
      );
    });
  });

  describe('getLocale() / setLocale()', () => {
    it('defaults to nb', () => {
      expect(getLocale()).toBe('nb');
    });

    it('switches locale', () => {
      setLocale('en');
      expect(getLocale()).toBe('en');
    });

    it('persists to localStorage', () => {
      setLocale('en');
      expect(localStorageMock.setItem).toHaveBeenCalledWith('locale', 'en');
    });

    it('updates document.documentElement.lang', () => {
      setLocale('en');
      expect(document.documentElement.lang).toBe('en');
    });
  });

  describe('getDateLocale()', () => {
    it('returns nb-NO for Norwegian', () => {
      setLocale('nb');
      expect(getDateLocale()).toBe('nb-NO');
    });

    it('returns en-GB for English', () => {
      setLocale('en');
      expect(getDateLocale()).toBe('en-GB');
    });
  });

  describe('initI18n()', () => {
    it('reads locale from localStorage', () => {
      localStorageStore['locale'] = 'en';
      initI18n();
      expect(getLocale()).toBe('en');
    });

    it('ignores invalid localStorage values', () => {
      localStorageStore['locale'] = 'fr';
      vi.spyOn(navigator, 'language', 'get').mockReturnValue('nb-NO');
      initI18n();
      expect(getLocale()).toBe('nb');
    });

    it('detects English browser language', () => {
      vi.spyOn(navigator, 'language', 'get').mockReturnValue('en-US');
      initI18n();
      expect(getLocale()).toBe('en');
    });

    it('defaults to nb for non-English browser', () => {
      vi.spyOn(navigator, 'language', 'get').mockReturnValue('de-DE');
      initI18n();
      expect(getLocale()).toBe('nb');
    });

    it('sets document lang attribute', () => {
      localStorageStore['locale'] = 'en';
      initI18n();
      expect(document.documentElement.lang).toBe('en');
    });
  });
});
