import { defineConfig } from 'vitest/config'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { svelteTesting } from '@testing-library/svelte/vite'

export default defineConfig({
  plugins: [
    svelte(),
    svelteTesting()
  ],
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['src/**/*.{test,spec}.{js,ts}'],
    setupFiles: ['src/setupTests.ts'],
  },
  resolve: {
    conditions: ['browser'],
    alias: {
      '$lib': '/src/lib',
      '$app/navigation': '/src/mocks/navigation.ts',
      '$app/state': '/src/mocks/state.ts',
    }
  }
})
