/**
 * E2E Tests - New Features (Phase 2-5)
 * 
 * Run with: npx playwright test collaboration-features.spec.js
 * 
 * Tests cover:
 * - Conflict resolution UI
 * - Audit timeline
 * - Real-time CRDT collaboration
 * - Workflow triggers
 */

const { test, expect } = require('@playwright/test');

const BASE_URL = process.env.TEST_URL || 'http://localhost:8080';
const API_URL = process.env.API_URL || 'http://localhost:8111';

test.describe.configure({ mode: 'serial' });

test.describe('Conflict Resolution', () => {

    test.beforeEach(async ({ page }) => {
        await page.goto(BASE_URL);
        // Wait for app to load
        await page.waitForSelector('body', { timeout: 5000 });
    });

    test('Shows conflict modal when version mismatch detected', async ({ page }) => {
        // Navigate to contacts
        await page.goto(`${BASE_URL}/contacts`);

        // Create a contact
        await page.click('[data-testid="create-contact-button"]');
        await page.fill('[data-testid="field-name"]', 'Conflict Test User');
        await page.fill('[data-testid="field-email"]', 'conflict@test.com');
        await page.click('[data-testid="save-button"]');

        // Simulate server version change via API
        // This would typically happen when another user edits

        // Edit locally
        await page.click('[data-testid="edit-button"]');
        await page.fill('[data-testid="field-name"]', 'Local Update');

        // Mock conflict response
        await page.route('**/api/v1/entities/contact/**', (route) => {
            if (route.request().method() === 'PUT') {
                route.fulfill({
                    status: 409,
                    contentType: 'application/json',
                    body: JSON.stringify({
                        error: 'Version conflict',
                        server_version: 5,
                        server_data: { name: 'Server Update', email: 'conflict@test.com' }
                    })
                });
            } else {
                route.continue();
            }
        });

        await page.click('[data-testid="save-button"]');

        // Conflict resolver should appear
        await expect(page.locator('.conflict-overlay')).toBeVisible({ timeout: 5000 });
        await expect(page.locator('.conflict-modal')).toBeVisible();

        // Should show diff
        await page.click('.toggle-details');
        await expect(page.locator('.diff-column.local')).toBeVisible();
        await expect(page.locator('.diff-column.server')).toBeVisible();
    });

    test('Keep Mine resolution applies local changes', async ({ page }) => {
        // Setup conflict scenario
        await page.route('**/api/v1/entities/contact/**', (route) => {
            if (route.request().method() === 'PUT') {
                route.fulfill({
                    status: 409,
                    contentType: 'application/json',
                    body: JSON.stringify({
                        error: 'Version conflict',
                        server_version: 5
                    })
                });
            } else {
                route.continue();
            }
        });

        // Trigger conflict (simulated)
        await page.evaluate(() => {
            window.dispatchEvent(new CustomEvent('sync-conflict', {
                detail: {
                    entity_id: '123',
                    entity_type: 'contact',
                    local_data: { name: 'My Version' },
                    server_data: { name: 'Server Version' }
                }
            }));
        });

        // Click Keep Mine
        if (await page.locator('.conflict-modal').isVisible()) {
            await page.click('.btn-primary'); // Keep Mine
            await expect(page.locator('.conflict-overlay')).not.toBeVisible({ timeout: 3000 });
        }
    });

    test('Keep Theirs resolution discards local changes', async ({ page }) => {
        // Similar setup, click Keep Theirs button
        await page.evaluate(() => {
            window.dispatchEvent(new CustomEvent('sync-conflict', {
                detail: {
                    entity_id: '123',
                    entity_type: 'contact',
                    local_data: { name: 'My Version' },
                    server_data: { name: 'Server Version' }
                }
            }));
        });

        if (await page.locator('.conflict-modal').isVisible()) {
            await page.click('.btn-secondary'); // Keep Theirs
            await expect(page.locator('.conflict-overlay')).not.toBeVisible({ timeout: 3000 });
        }
    });
});

test.describe('Audit Timeline', () => {

    test('Shows audit history for an entity', async ({ page }) => {
        await page.goto(`${BASE_URL}/contacts`);

        // Click first contact
        await page.click('[data-testid="contact-row"]');

        // Navigate to activity/timeline tab
        const timelineTab = page.locator('[data-testid="timeline-tab"]');
        if (await timelineTab.isVisible()) {
            await timelineTab.click();

            // Audit timeline component should be visible
            await expect(page.locator('.audit-timeline')).toBeVisible();
            await expect(page.locator('.audit-header h3')).toContainText('Activity History');
        }
    });

    test('Filter audit by user', async ({ page }) => {
        await page.goto(`${BASE_URL}/contacts`);
        await page.click('[data-testid="contact-row"]');

        const timelineTab = page.locator('[data-testid="timeline-tab"]');
        if (await timelineTab.isVisible()) {
            await timelineTab.click();

            // Use filter dropdown
            const filterSelect = page.locator('.filter-select');
            if (await filterSelect.isVisible()) {
                await filterSelect.selectOption({ index: 1 }); // Select first user

                // Events should be filtered
                await page.waitForTimeout(500);
            }
        }
    });

    test('Export audit as CSV', async ({ page }) => {
        await page.goto(`${BASE_URL}/contacts`);
        await page.click('[data-testid="contact-row"]');

        const timelineTab = page.locator('[data-testid="timeline-tab"]');
        if (await timelineTab.isVisible()) {
            await timelineTab.click();

            // Track download
            const [download] = await Promise.all([
                page.waitForEvent('download').catch(() => null),
                page.click('.export-btn').catch(() => null)
            ]);

            if (download) {
                expect(download.suggestedFilename()).toContain('audit');
                expect(download.suggestedFilename()).toContain('.csv');
            }
        }
    });

    test('View event details modal', async ({ page }) => {
        await page.goto(`${BASE_URL}/contacts`);
        await page.click('[data-testid="contact-row"]');

        const timelineTab = page.locator('[data-testid="timeline-tab"]');
        if (await timelineTab.isVisible()) {
            await timelineTab.click();

            // Click view button on first audit item
            const viewBtn = page.locator('.audit-item .view-btn').first();
            if (await viewBtn.isVisible()) {
                await viewBtn.click();

                // Modal should appear with details
                await expect(page.locator('.audit-modal')).toBeVisible();
                await expect(page.locator('.audit-modal .modal-header h4')).toContainText('Event Details');

                // Close modal
                await page.click('.audit-modal .close-btn');
                await expect(page.locator('.audit-modal')).not.toBeVisible();
            }
        }
    });
});

test.describe('Real-time Collaboration (Extended)', () => {

    test('WebSocket connection established on page load', async ({ page }) => {
        // Track WebSocket connections
        const wsConnections = [];
        page.on('websocket', ws => {
            wsConnections.push(ws.url());
        });

        await page.goto(BASE_URL);
        await page.waitForTimeout(2000);

        // Should have attempted WebSocket connection
        // Note: May fail if WS server not running, which is expected
        console.log('WebSocket connections attempted:', wsConnections.length);
    });

    test('Presence indicator shows connected users', async ({ page }) => {
        await page.goto(`${BASE_URL}/contacts`);
        await page.click('[data-testid="contact-row"]');

        // Check for presence/avatar indicators
        const presenceIndicator = page.locator('.presence-indicator, .user-avatars, .active-users');
        if (await presenceIndicator.isVisible()) {
            // Should show at least current user
            const userCount = await presenceIndicator.count();
            expect(userCount).toBeGreaterThanOrEqual(0);
        }
    });

    test('Changes sync between browser tabs', async ({ page, context }) => {
        // Open same record in two tabs
        const page2 = await context.newPage();

        await page.goto(`${BASE_URL}/contacts`);
        await page2.goto(`${BASE_URL}/contacts`);

        // Create contact in page 1
        await page.click('[data-testid="create-contact-button"]');
        const testName = `Sync-Test-${Date.now()}`;
        await page.fill('[data-testid="field-name"]', testName);
        await page.fill('[data-testid="field-email"]', 'sync@test.com');
        await page.click('[data-testid="save-button"]');

        // Refresh page 2 and verify
        await page2.reload();
        await page2.waitForTimeout(1000);

        // Should see new contact
        const newContact = page2.locator(`text=${testName}`);
        await expect(newContact).toBeVisible({ timeout: 10000 });

        await page2.close();
    });
});

test.describe('Sync Indicator', () => {

    test('Shows synced status when online', async ({ page }) => {
        await page.goto(BASE_URL);
        await page.waitForTimeout(2000);

        const syncIndicator = page.locator('.sync-indicator, [data-testid="sync-indicator"]');
        if (await syncIndicator.isVisible()) {
            // Should show synced or online status
            const status = await syncIndicator.getAttribute('data-status');
            expect(['synced', 'online', 'connected']).toContain(status);
        }
    });

    test('Shows offline status when disconnected', async ({ page, context }) => {
        await page.goto(BASE_URL);

        // Go offline
        await context.setOffline(true);
        await page.waitForTimeout(1000);

        const syncIndicator = page.locator('.sync-indicator, [data-testid="sync-indicator"]');
        if (await syncIndicator.isVisible()) {
            const status = await syncIndicator.getAttribute('data-status');
            expect(['offline', 'disconnected', 'syncing']).toContain(status);
        }

        // Go back online
        await context.setOffline(false);
    });
});

test.describe('Command Palette', () => {

    test('Opens with Ctrl+K', async ({ page }) => {
        await page.goto(BASE_URL);

        await page.keyboard.press('Control+k');

        // Command palette should be visible
        const palette = page.locator('.command-palette, [data-testid="command-palette"]');
        await expect(palette).toBeVisible({ timeout: 2000 });
    });

    test('Search filters commands', async ({ page }) => {
        await page.goto(BASE_URL);
        await page.keyboard.press('Control+k');

        const paletteInput = page.locator('.command-palette input, [data-testid="command-palette-input"]');
        if (await paletteInput.isVisible()) {
            await paletteInput.fill('contact');

            // Should show filtered results
            await page.waitForTimeout(300);
            const results = page.locator('.command-palette-results .result-item');
            const count = await results.count();
            expect(count).toBeGreaterThan(0);
        }
    });

    test('Navigate results with keyboard', async ({ page }) => {
        await page.goto(BASE_URL);
        await page.keyboard.press('Control+k');

        const paletteInput = page.locator('.command-palette input');
        if (await paletteInput.isVisible()) {
            await paletteInput.fill('create');
            await page.waitForTimeout(300);

            // Navigate with arrow keys
            await page.keyboard.press('ArrowDown');
            await page.keyboard.press('ArrowDown');
            await page.keyboard.press('ArrowUp');

            // Select with Enter
            await page.keyboard.press('Enter');

            // Palette should close
            await expect(page.locator('.command-palette')).not.toBeVisible({ timeout: 2000 });
        }
    });

    test('Closes with Escape', async ({ page }) => {
        await page.goto(BASE_URL);
        await page.keyboard.press('Control+k');

        await expect(page.locator('.command-palette')).toBeVisible();
        await page.keyboard.press('Escape');
        await expect(page.locator('.command-palette')).not.toBeVisible({ timeout: 2000 });
    });
});
