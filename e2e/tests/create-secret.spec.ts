import { test, expect } from '@playwright/test';

test.describe('Create Secret', () => {
  test('should create secret and display passphrase and share URL', async ({ page }) => {
    await page.goto('/create');

    // Fill in the secret
    await page.getByTestId('secret-input').fill('my-super-secret-api-key');

    // Set options
    await page.getByTestId('max-views-input').fill('3');
    await page.getByTestId('expires-select').selectOption('24');

    // Submit
    await page.getByTestId('submit-button').click();

    // Verify result is displayed
    await expect(page.getByTestId('passphrase-display')).toBeVisible();
    await expect(page.getByTestId('share-url-display')).toBeVisible();

    // Verify passphrase format (3 words)
    const passphrase = await page.getByTestId('passphrase-display').textContent();
    expect(passphrase).toBeTruthy();
    const wordCount = passphrase!.trim().split('-').length;
    expect(wordCount).toBe(3);

    // Verify share URL contains secret ID
    const shareUrl = await page.getByTestId('share-url-display').inputValue();
    expect(shareUrl).toContain('/secret/');
  });

  test('should show error when submitting empty secret', async ({ page }) => {
    await page.goto('/create');

    // Submit without entering secret
    await page.getByTestId('submit-button').click();

    // Should show error
    await expect(page.getByTestId('error-message')).toBeVisible();
  });

  test('should copy passphrase to clipboard', async ({ page, context }) => {
    // Grant clipboard permissions
    await context.grantPermissions(['clipboard-read', 'clipboard-write']);

    await page.goto('/create');
    await page.getByTestId('secret-input').fill('clipboard-test-secret');
    await page.getByTestId('submit-button').click();

    // Wait for result
    await expect(page.getByTestId('passphrase-display')).toBeVisible();

    // Click copy button
    await page.getByTestId('copy-passphrase-button').click();

    // Verify clipboard content
    const clipboardContent = await page.evaluate(() => navigator.clipboard.readText());
    const displayedPassphrase = await page.getByTestId('passphrase-display').textContent();
    expect(clipboardContent).toBe(displayedPassphrase!.trim());
  });
});
