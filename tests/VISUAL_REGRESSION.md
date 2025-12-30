# Visual Regression Testing Guide

## Purpose
Before migrating to Tailwind CSS, we need to capture baseline screenshots of the current UI to ensure no visual breakage during the conversion.

## Setup

### 1. Install Dependencies
```powershell
cd tests/e2e
npm install
npx playwright install chromium
```

### 2. Start the Application
```powershell
# Terminal 1: Start backend
docker start saas-postgres
cd /mnt/e/s_programmer/Saas\ System
export DATABASE_URL="postgres://postgres@172.29.208.1:15432/saas"
cargo run --bin server

# Terminal 2: Start frontend
cd crates/frontend-web
trunk serve --port 8080
```

## Running Tests

### Capture Baseline (Pre-Tailwind)
```powershell
# From project root
npx playwright test visual-regression.spec.js --update-snapshots
```

This creates baseline screenshots in `tests/e2e/visual-regression.spec.js-snapshots/`

### Verify After Tailwind Migration
```powershell
# After completing Tailwind conversion
npx playwright test visual-regression.spec.js
```

If differences are detected, Playwright will fail the test and show diffs.

### Review Differences
```powershell
npx playwright show-report
```

## What's Tested

| Test | Purpose |
|------|---------|
| Dashboard Full Page | Overall layout and spacing |
| Header Navigation | Top bar, logo, user menu |
| Sidebar Navigation | Navigation drawer |
| Contacts/Properties/Deals Lists | Table components, filtering |
| Create Forms | Form inputs, validation |
| Button Variants | Primary, secondary, danger buttons |
| Form Inputs | Text fields, selects, textareas |
| Dark Mode | Theme switching (if supported) |
| Responsive (Mobile/Tablet) | Breakpoint behavior |

## Updating Baseline

If intentional visual changes are made:
```powershell
npx playwright test visual-regression.spec.js --update-snapshots --grep="specific test name"
```

## CI Integration

Add to GitHub Actions:
```yaml
- name: Run Visual Regression Tests
  run: npx playwright test visual-regression.spec.js
  
- name: Upload Test Report
  if: failure()
  uses: actions/upload-artifact@v3
  with:
    name: playwright-report
    path: playwright-report/
```

## Troubleshooting

### Tests Timeout
- Increase `timeout` in test config
- Check if backend is running
- Verify `baseURL` in `playwright.config.js`

### Flaky Snapshots
- Use `animations: 'disabled'`
- Mask dynamic content with `mask: [page.locator('[data-dynamic]')]`  
- Wait for `networkidle` before screenshot

### Different Fonts/Rendering
- Ensure same OS for baseline and comparison
- Use Docker for consistent environment
