/**
 * Visual Regression Baseline Tests
 * 
 * Purpose: Capture screenshots of key UI components BEFORE Tailwind migration
 * to ensure no visual breakage during the conversion.
 * 
 * Run: npx playwright test visual-regression.spec.js
 * Update Baseline: npx playwright test visual-regression.spec.js --update-snapshots
 */

const { test, expect } = require('@playwright/test');

// Use consistent viewport for snapshots
test.use({
    viewport: { width: 1920, height: 1080 },
    deviceScaleFactor: 1,
});

test.describe('Visual Regression Baseline - Pre-Tailwind', () => {

    test.beforeEach(async ({ page }) => {
        // Login first (if authentication required)
        await page.goto('/');
        // TODO: Add login steps if auth is enabled
        // For now, assume we can access the dashboard directly
    });

    test('Dashboard - Full Page', async ({ page }) => {
        await page.goto('/');
        await page.waitForLoadState('networkidle');

        // Wait for dynamic content to load
        await page.waitForSelector('[data-testid="dashboard"], .dashboard, main', { timeout: 10000 });

        await expect(page).toHaveScreenshot('dashboard-full.png', {
            fullPage: true,
            animations: 'disabled',
        });
    });

    test('Header and Navigation', async ({ page }) => {
        await page.goto('/');
        await page.waitForLoadState('networkidle');

        const header = page.locator('header, [role="banner"], nav').first();
        await expect(header).toBeVisible();

        await expect(header).toHaveScreenshot('header.png', {
            animations: 'disabled',
        });
    });

    test('Sidebar Navigation', async ({ page }) => {
        await page.goto('/');
        await page.waitForLoadState('networkidle');

        const sidebar = page.locator('aside, [role="navigation"], .sidebar').first();
        if (await sidebar.isVisible()) {
            await expect(sidebar).toHaveScreenshot('sidebar.png', {
                animations: 'disabled',
            });
        } else {
            console.log('Sidebar not visible on this page');
        }
    });

    test('Contacts List View', async ({ page }) => {
        await page.goto('/app/crm/entity/contact');
        await page.waitForLoadState('networkidle');

        // Wait for table OR empty state to load
        await page.waitForSelector('table, .table, [role="grid"], .empty-state', { timeout: 10000 });

        await expect(page).toHaveScreenshot('contacts-list.png', {
            fullPage: true,
            animations: 'disabled',
        });
    });

    test('Properties List View', async ({ page }) => {
        await page.goto('/app/crm/entity/property');
        await page.waitForLoadState('networkidle');

        await page.waitForSelector('table, .table, [role="grid"], .empty-state', { timeout: 10000 });

        await expect(page).toHaveScreenshot('properties-list.png', {
            fullPage: true,
            animations: 'disabled',
        });
    });

    test('Deals List View', async ({ page }) => {
        await page.goto('/app/crm/entity/deal');
        await page.waitForLoadState('networkidle');

        await page.waitForSelector('table, .table, [role="grid"], .empty-state', { timeout: 10000 });

        await expect(page).toHaveScreenshot('deals-list.png', {
            fullPage: true,
            animations: 'disabled',
        });
    });

    test('Create Form - Contact', async ({ page }) => {
        await page.goto('/app/crm/entity/contact');
        await page.waitForLoadState('networkidle');

        // Click "Create" or "New Contact" button
        const createButton = page.locator('button:has-text("Create"), button:has-text("New"), a:has-text("Create")').first();
        if (await createButton.isVisible()) {
            await createButton.click();
            await page.waitForSelector('form, [role="form"]', { timeout: 5000 });

            await expect(page).toHaveScreenshot('contact-create-form.png', {
                animations: 'disabled',
            });
        }
    });

    test('Table Component', async ({ page }) => {
        await page.goto('/app/crm/entity/contact');
        await page.waitForLoadState('networkidle');

        const tableOrEmpty = page.locator('table, .table, [role="grid"], .empty-state').first();
        await expect(tableOrEmpty).toBeVisible();

        await expect(tableOrEmpty).toHaveScreenshot('table-component.png', {
            animations: 'disabled',
        });
    });

    test('Button Variants', async ({ page }) => {
        await page.goto('/');
        await page.waitForLoadState('networkidle');

        // Capture all buttons on the page
        const buttonsContainer = page.locator('body');
        await expect(buttonsContainer).toHaveScreenshot('buttons.png', {
            animations: 'disabled',
            mask: [page.locator('[data-dynamic]')], // Mask any dynamic content
        });
    });

    test('Form Inputs', async ({ page }) => {
        await page.goto('/contacts');

        const createButton = page.locator('button:has-text("Create"), button:has-text("New")').first();
        if (await createButton.isVisible()) {
            await createButton.click();
            await page.waitForSelector('input, select, textarea', { timeout: 5000 });

            await expect(page).toHaveScreenshot('form-inputs.png', {
                animations: 'disabled',
            });
        }
    });

    test('Dark Mode (if supported)', async ({ page }) => {
        await page.goto('/');
        await page.waitForLoadState('networkidle');

        // Try to toggle dark mode
        const darkModeButton = page.locator('[aria-label*="dark"], [aria-label*="theme"], button:has-text("Dark")').first();

        if (await darkModeButton.isVisible()) {
            await darkModeButton.click();
            await page.waitForTimeout(500); // Wait for theme transition

            await expect(page).toHaveScreenshot('dashboard-dark-mode.png', {
                fullPage: true,
                animations: 'disabled',
            });
        }
    });

    test('Responsive - Mobile View', async ({ page }) => {
        await page.setViewportSize({ width: 375, height: 667 }); // iPhone SE
        await page.goto('/');
        await page.waitForLoadState('networkidle');

        await expect(page).toHaveScreenshot('dashboard-mobile.png', {
            fullPage: true,
            animations: 'disabled',
        });
    });

    test('Responsive - Tablet View', async ({ page }) => {
        await page.setViewportSize({ width: 768, height: 1024 }); // iPad
        await page.goto('/');
        await page.waitForLoadState('networkidle');

        await expect(page).toHaveScreenshot('dashboard-tablet.png', {
            fullPage: true,
            animations: 'disabled',
        });
    });
});
