import { test, expect } from '@playwright/test';

// Extract pathname from full URL
function getPathname(url: string): string {
  return new URL(url).pathname;
}

test.describe('Retrieve Secret', () => {
  test('should retrieve secret with valid passphrase', async ({ page }) => {
    // First create a secret
    await page.goto('/create');
    const originalSecret = 'test-secret-for-retrieval-12345';
    await page.getByTestId('secret-input').fill(originalSecret);
    await page.getByTestId('submit-button').click();

    // Get passphrase and share URL
    await expect(page.getByTestId('passphrase-display')).toBeVisible();
    const passphrase = (await page.getByTestId('passphrase-display').textContent())!.trim();
    const shareUrl = await page.getByTestId('share-url-display').inputValue();

    // Navigate to share URL
    await page.goto(getPathname(shareUrl));

    // Enter passphrase
    await page.getByTestId('passphrase-input').fill(passphrase);
    await page.getByTestId('retrieve-button').click();

    // Verify secret is displayed
    await expect(page.getByTestId('secret-content')).toBeVisible();
    const displayedSecret = await page.getByTestId('secret-content').textContent();
    expect(displayedSecret).toBe(originalSecret);
  });

  test('should show error with wrong passphrase', async ({ page }) => {
    // Create a secret
    await page.goto('/create');
    await page.getByTestId('secret-input').fill('secret-wrong-pass-test');
    await page.getByTestId('submit-button').click();

    await expect(page.getByTestId('share-url-display')).toBeVisible();
    const shareUrl = await page.getByTestId('share-url-display').inputValue();

    // Navigate and enter wrong passphrase
    await page.goto(getPathname(shareUrl));
    await page.getByTestId('passphrase-input').fill('wrong-pass-phrase');
    await page.getByTestId('retrieve-button').click();

    // Should show error
    await expect(page.getByTestId('error-message')).toBeVisible();
  });

  test('should complete full sender-to-recipient flow in separate contexts', async ({ browser, baseURL }) => {
    // Sender context
    const senderContext = await browser.newContext({ baseURL });
    const senderPage = await senderContext.newPage();

    await senderPage.goto('/create');
    const secretMessage = 'Message from sender to recipient';
    await senderPage.getByTestId('secret-input').fill(secretMessage);
    await senderPage.getByTestId('max-views-input').fill('2');
    await senderPage.getByTestId('submit-button').click();

    await expect(senderPage.getByTestId('passphrase-display')).toBeVisible();
    const passphrase = (await senderPage.getByTestId('passphrase-display').textContent())!.trim();
    const shareUrl = await senderPage.getByTestId('share-url-display').inputValue();

    await senderContext.close();

    // Recipient context (simulates different user/browser)
    const recipientContext = await browser.newContext({ baseURL });
    const recipientPage = await recipientContext.newPage();

    await recipientPage.goto(getPathname(shareUrl));
    await recipientPage.getByTestId('passphrase-input').fill(passphrase);
    await recipientPage.getByTestId('retrieve-button').click();

    await expect(recipientPage.getByTestId('secret-content')).toBeVisible();
    const retrievedSecret = await recipientPage.getByTestId('secret-content').textContent();
    expect(retrievedSecret).toBe(secretMessage);

    // Verify views remaining
    await expect(recipientPage.getByTestId('views-remaining')).toContainText('1');

    await recipientContext.close();
  });
});
