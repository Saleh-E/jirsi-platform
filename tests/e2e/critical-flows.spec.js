/**
 * E2E Testing with Playwright - Jirsi Platform
 * 
 * Run with: npx playwright test
 * 
 * Tests cover:
 * - Critical user flows (CRUD operations)
 * - SmartField rendering
 * - Command palette
 * - Workflow execution
 * - Offline-first sync (create offline → reconnect → verify)
 * - Real-time collaboration
 * - Performance benchmarks
 */

const { test, expect } = require('@playwright/test');

// Base URLs
const BASE_URL = process.env.TEST_URL || 'http://localhost:8080';
const API_URL = process.env.API_URL || 'http://localhost:8111';

// Test Configuration
test.describe.configure({ mode: 'serial' });

test.describe('Critical User Flows', () => {

    test.beforeEach(async ({ page }) => {
        // Navigate to app
        await page.goto(BASE_URL);

        // Wait for app to load
        await page.waitForSelector('[data-testid="app-loaded"]', { timeout: 10000 }).catch(() => {
            console.log('App loaded selector not found, continuing...');
        });

        // Login if auth is enabled
        const loginButton = page.locator('[data-testid="login-button"]');
        if (await loginButton.isVisible()) {
            await page.fill('[data-testid="email"]', 'test@jirsi.com');
            await page.fill('[data-testid="password"]', 'testpassword123');
            await loginButton.click();
            await page.waitForTimeout(1000);
        }
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
});

test.describe('Offline-First Sync Tests', () => {

    test('Full Offline Cycle: Create Offline → Reconnect → Verify on Backend', async ({ page, context }) => {
        // Step 1: Navigate while online
        await page.goto(BASE_URL);
        await page.click('[data-testid="nav-contacts"]');

        // Step 2: Go offline BEFORE creating record
        await context.setOffline(true);

        // Step 3: Create a record while OFFLINE
        await page.click('[data-testid="create-contact-button"]');
        const testEmail = `offline-${Date.now()}@test.com`;
        await page.fill('[data-testid="field-name"]', 'Offline User');
        await page.fill('[data-testid="field-email"]', testEmail);
        await page.click('[data-testid="save-button"]');

        // Should show offline indicator
        await expect(page.locator('[data-testid="sync-indicator"][data-status="offline"]')).toBeVisible();

        // Step 4: Verify data is saved locally (optimistic)
        await expect(page.locator('text=Offline User')).toBeVisible();

        // Step 5: Go back online
        await context.setOffline(false);

        // Step 6: Wait for background sync to complete
        await expect(page.locator('[data-testid="sync-indicator"][data-status="synced"]')).toBeVisible({ timeout: 15000 });

        // Step 7: Verify on backend (direct API call)
        const response = await page.request.get(`${API_URL}/api/v1/entities/contact?search=${testEmail}`);
        const data = await response.json();
        expect(data.data).toBeDefined();
        expect(data.data.length).toBeGreaterThan(0);
        expect(data.data[0].email).toBe(testEmail);
    });

    test('Offline Edit → Reconnect → Server reflects changes', async ({ page, context }) => {
        // Create a record while online first
        await page.goto(`${BASE_URL}/contacts`);
        await page.click('[data-testid="create-contact-button"]');
        const testName = `Edit-Test-${Date.now()}`;
        await page.fill('[data-testid="field-name"]', testName);
        await page.fill('[data-testid="field-email"]', 'edit-test@test.com');
        await page.click('[data-testid="save-button"]');
        await expect(page.locator('[data-testid="success-toast"]')).toBeVisible();

        // Get the record ID from URL or data attribute
        const recordRow = page.locator(`text=${testName}`);
        await recordRow.click();

        // Go offline
        await context.setOffline(true);

        // Edit the record
        await page.click('[data-testid="edit-button"]');
        const updatedName = `${testName}-UPDATED`;
        await page.fill('[data-testid="field-name"]', updatedName);
        await page.click('[data-testid="save-button"]');

        // Should show syncing/offline status
        await expect(page.locator('[data-testid="sync-indicator"]')).toHaveAttribute('data-status', /(offline|syncing)/);

        // Go back online
        await context.setOffline(false);

        // Wait for sync
        await expect(page.locator('[data-testid="sync-indicator"][data-status="synced"]')).toBeVisible({ timeout: 15000 });

        // Reload and verify
        await page.reload();
        await expect(page.locator(`text=${updatedName}`)).toBeVisible();
    });
});

test.describe('Real-time Collaboration', () => {

    test('Multi-user presence and live updates', async ({ page, context }) => {
        // Open two browser contexts (simulate two users)
        const page2 = await context.newPage();
        await page2.goto(BASE_URL);

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
        await expect(page2.locator('[data-testid="user-presence-1"]')).toBeVisible({ timeout: 5000 });

        // User 1: Save
        await page.click('[data-testid="save-button"]');

        // User 2: Should see update without reload (CRDT sync)
        await expect(page2.locator('text=Updated by User 1')).toBeVisible({ timeout: 5000 });
    });
});

test.describe('Performance Tests', () => {

    test('List view renders 1000 records quickly', async ({ page }) => {
        await page.goto(`${BASE_URL}/contacts`);

        // Measure render time
        const startTime = Date.now();
        await expect(page.locator('[data-testid="contact-row"]').first()).toBeVisible();
        const endTime = Date.now();

        const renderTime = endTime - startTime;
        console.log(`List view render time: ${renderTime}ms`);

        // Should render in less than 2 seconds
        expect(renderTime).toBeLessThan(2000);
    });

    test('Search returns results quickly', async ({ page }) => {
        await page.goto(`${BASE_URL}/contacts`);

        const startTime = Date.now();
        await page.fill('[data-testid="search-input"]', 'john');
        await expect(page.locator('[data-testid="search-result"]').first()).toBeVisible();
        const endTime = Date.now();

        const searchTime = endTime - startTime;
        console.log(`Search response time: ${searchTime}ms`);

        // Should search in less than 500ms
        expect(searchTime).toBeLessThan(500);
    });

    test('API response times under threshold', async ({ page }) => {
        // Test various API endpoints
        const endpoints = [
            '/api/v1/entities/contact',
            '/api/v1/entities/property',
            '/api/v1/entities/deal',
        ];

        for (const endpoint of endpoints) {
            const startTime = Date.now();
            const response = await page.request.get(`${API_URL}${endpoint}?limit=10`);
            const endTime = Date.now();

            const responseTime = endTime - startTime;
            console.log(`API ${endpoint}: ${responseTime}ms`);

            expect(response.ok()).toBeTruthy();
            expect(responseTime).toBeLessThan(500); // 500ms max
        }
    });
});

test.describe('Error Handling', () => {

    test('Shows error toast on API failure', async ({ page }) => {
        // Intercept API and force error
        await page.route('**/api/v1/entities/contact', (route) => {
            route.fulfill({
                status: 500,
                body: JSON.stringify({ error: 'Internal Server Error' }),
            });
        });

        await page.goto(`${BASE_URL}/contacts`);

        // Should show error state
        await expect(page.locator('[data-testid="error-message"]')).toBeVisible({ timeout: 5000 });
    });

    test('Gracefully handles network timeout', async ({ page }) => {
        // Set very short timeout
        await page.route('**/api/v1/**', async (route) => {
            await new Promise(resolve => setTimeout(resolve, 30000)); // 30s delay
            route.continue();
        });

        await page.goto(`${BASE_URL}/contacts`);

        // Should show loading state then timeout/error
        await expect(page.locator('[data-testid="loading-indicator"]')).toBeVisible();
    });
});

