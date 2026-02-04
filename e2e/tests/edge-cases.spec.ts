import { test, expect } from '@playwright/test';

// Extract pathname from full URL
function getPathname(url: string): string {
  return new URL(url).pathname;
}

test.describe('Edge Cases', () => {
  test('should successfully extend secret with additional views', async ({ page }) => {
    // Create secret with extendable enabled (default)
    await page.goto('/create');
    await page.getByTestId('secret-input').fill('extendable-secret-test');
    await page.getByTestId('max-views-input').fill('2');
    await page.getByTestId('submit-button').click();

    await expect(page.getByTestId('passphrase-display')).toBeVisible();
    const passphrase = (await page.getByTestId('passphrase-display').textContent())!.trim();
    const shareUrl = await page.getByTestId('share-url-display').inputValue();

    // Navigate to retrieve secret
    await page.goto(getPathname(shareUrl));
    await page.getByTestId('passphrase-input').fill(passphrase);
    await page.getByTestId('retrieve-button').click();

    // Verify secret is displayed
    await expect(page.getByTestId('secret-content')).toBeVisible();
    await expect(page.getByTestId('views-remaining')).toContainText('1');

    // Extension form should be visible
    await expect(page.getByTestId('add-views-input')).toBeVisible();
    await expect(page.getByTestId('add-days-input')).toBeVisible();
    await expect(page.getByTestId('extend-button')).toBeVisible();

    // Extend with additional views
    await page.getByTestId('add-views-input').fill('3');
    await page.getByTestId('extend-button').click();

    // Should show success message
    await expect(page.getByTestId('extend-success-message')).toBeVisible();

    // Views remaining should be updated (1 + 3 = 4)
    await expect(page.getByTestId('views-remaining')).toContainText('4');
  });

  test('should show disabled message when extension is not allowed', async ({ page }) => {
    // Create secret with extendable disabled
    await page.goto('/create');
    await page.getByTestId('secret-input').fill('non-extendable-secret');
    await page.getByTestId('max-views-input').fill('5');

    // Uncheck the extendable checkbox
    await page.getByTestId('extendable-checkbox').uncheck();
    await page.getByTestId('submit-button').click();

    await expect(page.getByTestId('passphrase-display')).toBeVisible();
    const passphrase = (await page.getByTestId('passphrase-display').textContent())!.trim();
    const shareUrl = await page.getByTestId('share-url-display').inputValue();

    // Navigate to retrieve secret
    await page.goto(getPathname(shareUrl));
    await page.getByTestId('passphrase-input').fill(passphrase);
    await page.getByTestId('retrieve-button').click();

    // Verify secret is displayed
    await expect(page.getByTestId('secret-content')).toBeVisible();

    // Extension should be disabled - show disabled message
    await expect(page.getByTestId('extend-disabled-message')).toBeVisible();

    // Extension form inputs should NOT be visible
    await expect(page.getByTestId('add-views-input')).not.toBeVisible();
    await expect(page.getByTestId('add-days-input')).not.toBeVisible();
    await expect(page.getByTestId('extend-button')).not.toBeVisible();
  });

  test('should show deleted message after max views reached', async ({ browser, baseURL }) => {
    // Create secret with max_views=1
    const context1 = await browser.newContext({ baseURL });
    const page1 = await context1.newPage();

    await page1.goto('/create');
    await page1.getByTestId('secret-input').fill('one-time-secret');
    await page1.getByTestId('max-views-input').fill('1');
    await page1.getByTestId('submit-button').click();

    await expect(page1.getByTestId('passphrase-display')).toBeVisible();
    const passphrase = (await page1.getByTestId('passphrase-display').textContent())!.trim();
    const shareUrl = await page1.getByTestId('share-url-display').inputValue();
    await context1.close();

    // First retrieval succeeds
    const context2 = await browser.newContext({ baseURL });
    const page2 = await context2.newPage();
    await page2.goto(getPathname(shareUrl));
    await page2.getByTestId('passphrase-input').fill(passphrase);
    await page2.getByTestId('retrieve-button').click();
    await expect(page2.getByTestId('secret-content')).toBeVisible();
    await context2.close();

    // Second retrieval fails
    const context3 = await browser.newContext({ baseURL });
    const page3 = await context3.newPage();
    await page3.goto(getPathname(shareUrl));
    await page3.getByTestId('passphrase-input').fill(passphrase);
    await page3.getByTestId('retrieve-button').click();
    await expect(page3.getByTestId('error-message')).toBeVisible();
    await context3.close();
  });

  test('should delete secret after too many wrong passphrase attempts', async ({ page }) => {
    // Create secret with max_views=2
    await page.goto('/create');
    await page.getByTestId('secret-input').fill('brute-force-test-secret');
    await page.getByTestId('max-views-input').fill('2');
    await page.getByTestId('submit-button').click();

    await expect(page.getByTestId('passphrase-display')).toBeVisible();
    const passphrase = (await page.getByTestId('passphrase-display').textContent())!.trim();
    const shareUrl = await page.getByTestId('share-url-display').inputValue();
    const secretPath = new URL(shareUrl).pathname;

    // Try wrong passphrase 4 times (2 free + 2 that consume views)
    for (let i = 0; i < 4; i++) {
      await page.goto(secretPath);
      await page.getByTestId('passphrase-input').fill('wrong-passphrase');
      await page.getByTestId('retrieve-button').click();

      if (i < 3) {
        // First 3 attempts should show error
        await expect(page.getByTestId('error-message')).toBeVisible();
      } else {
        // 4th attempt (2nd view-consuming attempt) should delete secret
        // and show not found error
        await expect(page.getByTestId('error-message')).toBeVisible();
      }
    }

    // Now try with correct passphrase - should be deleted
    await page.goto(secretPath);
    await page.getByTestId('passphrase-input').fill(passphrase);
    await page.getByTestId('retrieve-button').click();
    await expect(page.getByTestId('error-message')).toBeVisible();
  });

});
