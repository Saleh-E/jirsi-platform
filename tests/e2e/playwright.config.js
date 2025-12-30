// @ts-check
const { defineConfig, devices } = require('@playwright/test');

module.exports = defineConfig({
    testDir: '.',  // Current directory since config is now in tests/e2e
    fullyParallel: true,
    forbidOnly: !!process.env.CI,
    retries: process.env.CI ? 2 : 0,
    workers: process.env.CI ? 1 : undefined,
    reporter: 'html',

    use: {
        baseURL: 'http://localhost:8104',
        trace: 'on-first-retry',
        screenshot: 'only-on-failure',
    },

    projects: [
        {
            name: 'chromium',
            use: { ...devices['Desktop Chrome'] },
        },
    ],

    webServer: {
        // Go up two levels to reach crates/frontend-web
        command: 'cd ../../crates/frontend-web && trunk serve --port 8104 --address 0.0.0.0',
        url: 'http://localhost:8104',
        reuseExistingServer: true, // Reuse the server we started manually
        timeout: 120 * 1000,
    },
});
