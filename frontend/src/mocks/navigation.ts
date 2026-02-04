// Mock for $app/navigation
export function goto(url: string) {
  return Promise.resolve();
}

export function invalidate(url: string) {
  return Promise.resolve();
}

export function invalidateAll() {
  return Promise.resolve();
}

export function preloadData(url: string) {
  return Promise.resolve();
}

export function preloadCode(...urls: string[]) {
  return Promise.resolve();
}

export function beforeNavigate(callback: Function) {}

export function afterNavigate(callback: Function) {}

export function onNavigate(callback: Function) {}
