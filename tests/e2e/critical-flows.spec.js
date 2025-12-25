//! E2E Testing with Playwright
//! 
//! Run with: npm run test:e2e

const { test, expect } = require('@playwright/test');

test.describe('Critical User Flows', () => {

    test.beforeEach(async ({ page }) => {
        // Navigate to app
        await page.goto('http://localhost:8080');

        // Login (if needed)
        // await page.fill('[data-testid="email"]', 'test@example.com');
        // await page.fill('[data-testid="password"]', 'password');
        // await page.click('[data-testid="login-button"]');
    });

    test('Create Deal → Edit Field → Verify Timeline', async ({ page }) => {
        // Navigate to Deals
        await page.click('[data-testid="nav-deals"]');

        // Create new deal
        await page.click('[data-testid="create-deal-button"]');
        await page.fill('[data-testid="field-title"]', 'Test Deal');
        await page.fill('[data-testid="field-value"]', '10000');
        await page.click('[data-testid="save-button"]');

        // Wait for save
        await expect(page.locator('[data-testid="success-toast"]')).toBeVisible();

        // Edit field
        await page.click('[data-testid="edit-button"]');
        await page.fill('[data-testid="field-title"]', 'Updated Deal');
        await page.click('[data-testid="save-button"]');

        // Verify timeline shows update
        await page.click('[data-testid="timeline-tab"]');
        await expect(page.locator('text=Updated Deal')).toBeVisible();
    });

    test('SmartField rendering in all contexts', async ({ page }) => {
        // List view
        await page.click('[data-testid="nav-contacts"]');
        await expect(page.locator('[data-testid="smartfield-email"]')).toBeVisible();

        // Detail view
        await page.click('[data-testid="contact-row-1"]');
        await expect(page.locator('[data-testid="smartfield-phone"]')).toBeVisible();
        await expect(page.locator('[data-testid="smartfield-currency"]')).toBeVisible();

        // Edit mode
        await page.click('[data-testid="edit-button"]');
        await expect(page.locator('[data-testid="smartfield-date-picker"]')).toBeVisible();
    });

    test('Command palette functionality', async ({ page }) => {
        // Open command palette (Ctrl+K)
        await page.keyboard.press('Control+K');

        // Type command
        await page.fill('[data-testid="command-input"]', 'create contact');

        // Select first result
        await page.keyboard.press('Enter');

        // Should open create contact form
        await expect(page.locator('[data-testid="create-contact-form"]')).toBeVisible();
    });

    test('Workflow execution flow', async ({ page }) => {
        // Navigate to workflows
        await page.click('[data-testid="nav-workflows"]');

        // Select workflow
        await page.click('[data-testid="workflow-1"]');

        // Execute workflow
        await page.click('[data-testid="execute-button"]');

        // Select records
        await page.click('[data-testid="select-records-button"]');
        await page.click('[data-testid="record-checkbox-1"]');
        await page.click('[data-testid="record-checkbox-2"]');
        await page.click('[data-testid="confirm-selection"]');

        // Start execution
        await page.click('[data-testid="start-execution"]');

        // Wait for completion
        await expect(page.locator('[data-testid="execution-complete"]')).toBeVisible({ timeout: 30000 });

        // Verify results
        await expect(page.locator('[data-testid="success-count"]')).toContainText('2');
    });

    test('Offline-first sync scenario', async ({ page, context }) => {
        // Create a record while online
        await page.click('[data-testid="create-contact-button"]');
        await page.fill('[data-testid="field-name"]', 'John Doe');
        await page.fill('[data-testid="field-email"]', 'john@example.com');
        await page.click('[data-testid="save-button"]');

        // Go offline
        await context.setOffline(true);

        // Edit record (should queue mutation)
        await page.click('[data-testid="contact-row-latest"]');
        await page.click('[data-testid="edit-button"]');
        await page.fill('[data-testid="field-name"]', 'John Doe Updated');
        await page.click('[data-testid="save-button"]');

        // Should show offline indicator
        await expect(page.locator('[data-testid="offline-indicator"]')).toBeVisible();

        // Go back online
        await context.setOffline(false);

        // Wait for sync
        await expect(page.locator('[data-testid="sync-complete"]')).toBeVisible({ timeout: 10000 });

        // Verify change persisted
        await page.reload();
        await expect(page.locator('text=John Doe Updated')).toBeVisible();
    });

    test('Real-time collaboration', async ({ page, context }) => {
        // Open two browser contexts (simulate two users)
        const page2 = await context.newPage();
        await page2.goto('http://localhost:8080');

        // User 1: Navigate to a contact
        await page.click('[data-testid="nav-contacts"]');
        await page.click('[data-testid="contact-row-1"]');

        // User 2: Navigate to same contact
        await page2.click('[data-testid="nav-contacts"]');
        await page2.click('[data-testid="contact-row-1"]');

        // User 1: Edit a field
        await page.click('[data-testid="edit-button"]');
        await page.fill('[data-testid="field-notes"]', 'Updated by User 1');

        // User 2: Should see presence indicator
        await expect(page2.locator('[data-testid="user-presence-1"]')).toBeVisible();

        // User 1: Save
        await page.click('[data-testid="save-button"]');

        // User 2: Should see update without reload
        await expect(page2.locator('text=Updated by User 1')).toBeVisible({ timeout: 5000 });
    });
});

test.describe('Performance Tests', () => {
    test('List view renders 1000 records quickly', async ({ page }) => {
        await page.goto('http://localhost:8080/contacts');

        // Measure render time
        const startTime = Date.now();
        await expect(page.locator('[data-testid="contact-row"]').first()).toBeVisible();
        const endTime = Date.now();

        const renderTime = endTime - startTime;

        // Should render in less than 2 seconds
        expect(renderTime).toBeLessThan(2000);
    });

    test('Search returns results quickly', async ({ page }) => {
        await page.goto('http://localhost:8080/contacts');

        const startTime = Date.now();
        await page.fill('[data-testid="search-input"]', 'john');
        await expect(page.locator('[data-testid="search-result"]').first()).toBeVisible();
        const endTime = Date.now();

        const searchTime = endTime - startTime;

        // Should search in less than 500ms
        expect(searchTime).toBeLessThan(500);
    });
});
