# Testing Guide & Implementation Plan

Comprehensive testing strategy for the SmartField system.

---

## Testing Strategy Overview

### Test Pyramid

```
        /\
       /E2E\         ← Few, critical user flows
      /------\
     /  INT   \      ← Component integration
    /----------\
   /    UNIT    \    ← Many, fast, isolated
  /--------------\
```

**Target Coverage**: >80% for core components

---

## Phase 1: Unit Tests

### Test Infrastructure Setup

**File**: `crates/frontend-web/tests/test_utils.rs`

```rust
//! Test utilities for SmartField components

use leptos::*;
use serde_json::json;
use uuid::Uuid;
use core_models::field::{FieldDef, FieldType, FieldContext};

/// Create a test FieldDef with minimal required fields
pub fn create_test_field(name: &str, field_type: FieldType) -> FieldDef {
    FieldDef {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        entity_type_id: Uuid::new_v4(),
        name: name.to_string(),
        label: name.to_string(),
        field_type,
        is_required: false,
        is_unique: false,
        show_in_list: true,
        show_in_card: true,
        is_searchable: true,
        is_filterable: true,
        is_sortable: true,
        is_readonly: false,
        default_value: None,
        placeholder: Some("Test placeholder".to_string()),
        help_text: None,
        validation: None,
        options: None,
        ui_hints: None,
        context_hints: None,
        sort_order: 0,
        group: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

/// Create test select options
pub fn create_test_options() -> Vec<SelectChoice> {
    use core_models::field::SelectChoice;
    
    vec![
        SelectChoice {
            value: "option1".to_string(),
            label: "Option 1".to_string(),
            color: Some("#3B82F6".to_string()),
            icon: None,
            is_default: false,
            sort_order: 0,
        },
        SelectChoice {
            value: "option2".to_string(),
            label: "Option 2".to_string(),
            color: Some("#10B981".to_string()),
            icon: None,
            is_default: true,
            sort_order: 1,
        },
    ]
}

/// Mock API response
pub async fn mock_api_success<T: serde::de::DeserializeOwned>(data: T) -> Result<T, String> {
    Ok(data)
}

pub async fn mock_api_error() -> Result<(), String> {
    Err("Mock API error".to_string())
}
```

### SmartField Unit Tests

**File**: `crates/frontend-web/tests/smart_field_tests.rs`

```rust
#[cfg(test)]
mod smart_field_tests {
    use super::*;
    use crate::components::smart_field::SmartField;
    use crate::tests::test_utils::*;

    #[test]
    fn test_text_field_renders() {
        let field = create_test_field("name", FieldType::Text);
        let (value, _) = create_signal(json!("test value"));
        
        let rendered = view! {
            <SmartField
                field=field
                value=Signal::from(value)
                context=FieldContext::EditForm
            />
        };
        
        // Assertions would go here
        // Note: Full DOM assertions require a test framework like wasm-bindgen-test
        assert!(true); // Placeholder
    }
    
    #[test]
    fn test_dropdown_field_with_options() {
        let field = create_test_field(
            "status",
            FieldType::Dropdown {
                options: create_test_options(),
                allow_create: true,
            }
        );
        
        assert_eq!(field.name, "status");
        if let FieldType::Dropdown { options, allow_create } = field.field_type {
            assert_eq!(options.len(), 2);
            assert!(allow_create);
        } else {
            panic!("Expected Dropdown field type");
        }
    }
    
    #[test]
    fn test_required_field_validation() {
        let mut field = create_test_field("email", FieldType::Email);
        field.is_required = true;
        
        assert!(field.is_required);
        // Add validation logic tests here
    }
    
    #[test]
    fn test_context_aware_rendering() {
        let field = create_test_field("name", FieldType::Text);
        
        // Test different contexts render different components
        // This would require DOM inspection in a browser test
        assert!(true); // Placeholder
    }
}
```

### AsyncSelect Unit Tests

**File**: `crates/frontend-web/tests/async_select_tests.rs`

```rust
#[cfg(test)]
mod async_select_tests {
    use super::*;
    use crate::components::async_select::*;

    #[test]
    fn test_virtual_scroll_threshold() {
        assert_eq!(VIRTUAL_SCROLL_THRESHOLD, 50);
        assert_eq!(ITEM_HEIGHT, 44.0);
        assert_eq!(BUFFER_ITEMS, 3);
    }
    
    #[test]
    fn test_visible_range_calculation() {
        let options = (0..100).map(|i| SelectOption {
            value: format!("opt{}", i),
            label: format!("Option {}", i),
            description: None,
        }).collect::<Vec<_>>();
        
        // Test start of list
        let (start, end) = calculate_visible_range(0.0, options.len());
        assert_eq!(start, 0);
        assert!(end > 0 && end <= options.len());
        
        // Test middle of list
        let (start, end) = calculate_visible_range(500.0, options.len());
        assert!(start > 0);
        assert!(end > start);
    }
    
    #[test]
    fn test_search_filtering() {
        let options = vec![
            SelectOption {
                value: "js".to_string(),
                label: "JavaScript".to_string(),
                description: Some("Programming language".to_string()),
            },
            SelectOption {
                value: "rust".to_string(),
                label: "Rust".to_string(),
                description: Some("Systems language".to_string()),
            },
        ];
        
        let filtered = filter_options(&options, "rust");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].value, "rust");
        
        let filtered = filter_options(&options, "language");
        assert_eq!(filtered.len(), 2); // Matches description
    }
    
    fn calculate_visible_range(scroll_y: f64, total: usize) -> (usize, usize) {
        const ITEM_HEIGHT: f64 = 44.0;
        const VISIBLE_ITEMS: usize = 10;
        const BUFFER_ITEMS: usize = 3;
        
        let start = (scroll_y / ITEM_HEIGHT).floor() as usize;
        let start_with_buffer = start.saturating_sub(BUFFER_ITEMS);
        let end = (start + VISIBLE_ITEMS + BUFFER_ITEMS).min(total);
        
        (start_with_buffer, end)
    }
    
    fn filter_options(options: &[SelectOption], query: &str) -> Vec<SelectOption> {
        let query_lower = query.to_lowercase();
        options.iter()
            .filter(|opt| {
                opt.label.to_lowercase().contains(&query_lower) ||
                opt.value.to_lowercase().contains(&query_lower) ||
                opt.description.as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .cloned()
            .collect()
    }
}
```

---

## Phase 2: Integration Tests

### Component Integration Tests

**File**: `crates/frontend-web/tests/integration_tests.rs`

```rust
#[cfg(test)]
mod integration_tests {
    use wasm_bindgen_test::*;
    
    wasm_bindgen_test_configure!(run_in_browser);
    
    #[wasm_bindgen_test]
    async fn test_association_modal_workflow() {
        // Test full association creation workflow:
        // 1. Open modal
        // 2. Fill form
        // 3. Submit
        // 4. Verify callback
        // 5. Verify modal closes
        
        // Implementation would use leptos testing utilities
        assert!(true); // Placeholder
    }
    
    #[wasm_bindgen_test]
    async fn test_signature_pad_drawing() {
        // Test signature pad:
        // 1. Simulate mouse down
        // 2. Simulate mouse move
        // 3. Simulate mouse up
        // 4. Verify canvas has drawing
        // 5. Test clear button  
        // 6. Verify save returns base64
        
        assert!(true); // Placeholder
    }
    
    #[wasm_bindgen_test]
    async fn test_form_validation() {
        // Test form validation workflow:
        // 1. Create form with required fields
        // 2. Submit with empty values
        // 3. Verify validation errors shown
        // 4. Fill valid values
        // 5. Verify submission succeeds
        
        assert!(true); // Placeholder
    }
}
```

---

## Phase 3: End-to-End Tests

### Playwright Setup

**File**: `tests/e2e/playwright.config.ts`

```typescript
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:8081',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  webServer: {
    command: 'trunk serve',
    url: 'http://localhost:8081',
    reuseExistingServer: !process.env.CI,
  },
});
```

### E2E Test Examples

**File**: `tests/e2e/smartfield.spec.ts`

```typescript
import { test, expect } from '@playwright/test';

test.describe('SmartField Components', () => {
  test('renders text field correctly', async ({ page }) => {
    await page.goto('/app/playground');
    
    // Wait for field to render
    const textField = page.locator('input[type="text"]').first();
    await expect(textField).toBeVisible();
    
    // Test input
    await textField.fill('Hello World');
    await expect(textField).toHaveValue('Hello World');
  });
  
  test('dropdown with virtual scrolling', async ({ page }) => {
    await page.goto('/app/playground');
    
    // Open dropdown
    await page.click('[data-testid="status-dropdown"]');
    
    // Verify dropdown opens
    await expect(page.locator('[role="listbox"]')).toBeVisible();
    
    // Search
    await page.fill('input[role="combobox"]', 'active');
    
    // Verify filtered results
    const options = page.locator('[role="option"]');
    await expect(options).toHaveCount(1);
    
    // Select option
    await page.click('[role="option"]');
    
    // Verify selection
    await expect(page.locator('[data-testid="selected-value"]'))
      .toHaveText('Active');
  });
  
  test('association inline creation', async ({ page }) => {
    await page.goto('/app/entities/contacts');
    
    // Click create button
    await page.click('button:has-text("New Contact")');
    
    // Open association modal
    await page.click('[data-testid="company-field"]');
    await page.click('button:has-text("+ Create New")');
    
    // Fill modal
    await expect(page.locator('[role="dialog"]')).toBeVisible();
    await page.fill('input[name="name"]', 'Acme Corp');
    await page.click('button:has-text("Create")');
    
    // Verify auto-select
    await expect(page.locator('[data-testid="company-field"]'))
      .toContainText('Acme Corp');
  });
  
  test('signature pad workflow', async ({ page }) => {
    await page.goto('/app/playground');
    
    // Locate signature pad
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();
    
    // Simulate drawing (note: actual drawing simulation complex)
    // await canvas.click({ position: { x: 50, y: 50 } });
    
    // Test clear button
    await page.click('button:has-text("Clear")');
    
    // Test save button
    await page.click('button:has-text("Save")');
    
    // Verify callback (would check signal value)
  });
});

test.describe('Accessibility', () => {
  test('keyboard navigation in dropdown', async ({ page }) => {
    await page.goto('/app/playground');
    
    // Focus dropdown
    await page.focus('[data-testid="status-dropdown"] input');
    
    // Open with Enter
    await page.keyboard.press('Enter');
    
    // Navigate with arrow keys
    await page.keyboard.press('ArrowDown');
    await page.keyboard.press('ArrowDown');
    
    // Select with Enter
    await page.keyboard.press('Enter');
    
    // Verify selection
    // (assertions would check final state)
  });
  
  test('screen reader announcements', async ({ page }) => {
    await page.goto('/app/playground');
    
    // Check for ARIA labels
    const input = page.locator('input[type="text"]').first();
    await expect(input).toHaveAttribute('aria-label');
    
    // Check for live regions
    const status = page.locator('[role="status"]');
    await expect(status).toBeDefined();
  });
});
```

---

## Testing Commands

### Run Unit Tests

```bash
# Rust tests
cargo test --package frontend-web --lib

# With coverage
cargo tarpaulin --out Html --output-dir coverage/
```

### Run Integration Tests

```bash
# Browser tests
wasm-pack test --headless --chrome crates/frontend-web

# Firefox
wasm-pack test --headless --firefox crates/frontend-web
```

### Run E2E Tests

```bash
# Install Playwright
npm install -D @playwright/test

# Run tests
npx playwright test

# Run with UI
npx playwright test --ui

# Run specific test
npx playwright test smartfield.spec.ts
```

---

## Continuous Integration

### GitHub Actions Workflow

**File**: `.github/workflows/test.yml`

```yaml
name: Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run unit tests
        run: cargo test --all --lib
        
      - name: Run coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
          
      - name: Upload coverage
        uses: codecov/codecov-action@v3

  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install dependencies
        run: npm ci
        
      - name: Install Playwright
        run: npx playwright install --with-deps
        
      - name: Run E2E tests
        run: npx playwright test
        
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: playwright-report/
```

---

## Test Coverage Goals

| Component | Target | Current |
|-----------|--------|---------|
| SmartField core | 90% | 0% |
| AsyncSelect | 85% | 0% |
| Association components | 80% | 0% |
| SignaturePad | 75% | 0% |
| LocationMap | 70% | 0% |
| **Overall** | **80%** | **0%** |

---

## Quick Start Checklist

- [ ] Install wasm-pack: `cargo install wasm-pack`
- [ ] Install Playwright: `npm install -D @playwright/test`
- [ ] Create test utils: `tests/test_utils.rs`
- [ ] Write first unit test
- [ ] Write first E2E test
- [ ] Set up CI/CD
- [ ] Achieve 50% coverage
- [ ] Achieve 80% coverage

---

## Resources

- [wasm-bindgen-test Guide](https://rustwasm.github.io/wasm-bindgen/wasm-bindgen-test/)
- [Playwright Documentation](https://playwright.dev/)
- [Leptos Testing](https://leptos-rs.github.io/leptos/testing.html)
- [Cargo Tarpaulin](https://github.com/xd009642/tarpaulin)

---

**Estimated Implementation Time**: 12-16 hours for comprehensive suite  
**MVP Time**: 4-6 hours for critical path coverage

---

**Version**: 1.0.0  
**Last Updated**: 2024-12-24
