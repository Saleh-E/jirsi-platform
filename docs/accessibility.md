# Accessibility Implementation Guide

Complete checklist and implementation guide for WCAG AA compliance.

---

## Accessibility Status

### Current State
✅ **Good Foundation**: SmartField components have semantic HTML  
⚠️ **Needs Work**: ARIA labels, keyboard navigation improvements, screen reader support

### Target: WCAG 2.1 Level AA Compliance

---

## Priority 1: ARIA Labels & Roles

### AsyncSelect Component
**File**: `crates/frontend-web/src/components/async_select.rs`

```rust
// Add to search input:
<input
    type="text"
    role="combobox"
    aria-label="Search options"
    aria-expanded={move || is_open.get()}
    aria-controls="dropdown-listbox"
    aria-activedescendant={move || format!("option-{}", highlighted_index.get())}
    aria-autocomplete="list"
    aria-haspopup="listbox"
    // ... existing props
/>

// Add to dropdown list:
<ul
    id="dropdown-listbox"
    role="listbox"
    aria-label={move || placeholder.clone()}
    // ... existing props
>
    <For each=/* ... */ children=move |option| {
        <li
            role="option"
            id={format!("option-{}", option.value)}
            aria-selected={move || selected.get() == Some(option.value.clone())}
            aria-label={option.label.clone()}
            // ... existing props
        />
    } />
</ul>
```

### SmartField Input Elements
**File**: `crates/frontend-web/src/components/smart_field.rs`

```rust
// Text inputs:
<input
    type="text"
    id=field_id.clone()
    aria-label={field.label.clone()}
    aria-required={field.is_required.to_string()}
    aria-invalid={move || /* validation logic */}
    aria-describedby={field.help_text.as_ref().map(|_| format!("{}-help", field_id))}
    // ... existing props
/>

// Help text:
{field.help_text.as_ref().map(|help| view! {
    <span
        id={format!("{}-help", field_id)}
        class="form-help-text"
        role="note"
    >
        {help}
    </span>
})}

// Required indicator:
{if field.is_required {
    view! {
        <span aria-label="required" class="required-indicator">*</span>
    }
} else {
    view! {}
}}
```

### Modal Components
**File**: `crates/frontend-web/src/components/association_modal.rs`

```rust
<div
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="modal-title"
    aria-describedby="modal-description"
>
    <h2 id="modal-title">{format!("Create New {}", entity_type)}</h2>
    <p id="modal-description">Fill in the details below</p>
    
    // Close button:
    <button
        aria-label="Close dialog"
        on:click=on_close
    >
        ×
    </button>
</div>
```

---

## Priority 2: Keyboard Navigation

### Required Keyboard Shortcuts

| Component | Key | Action |
|-----------|-----|--------|
| AsyncSelect | `↓` | Next option |
| AsyncSelect | `↑` | Previous option |
| AsyncSelect | `Enter` | Select highlighted |
| AsyncSelect | `Esc` | Close dropdown |
| AsyncSelect | `Tab` | Close & next field |
| Modal | `Esc` | Close modal |
| Modal | `Tab` | Cycle through inputs |
| SignaturePad | `Esc` | Clear signature |

### Implementation Examples

#### AsyncSelect Keyboard Handler
```rust
let on_keydown = move |ev: KeyboardEvent| {
    match ev.key().as_str() {
        "ArrowDown" => {
            ev.prevent_default();
            set_highlighted_index.update(|idx| {
                *idx = (*idx + 1).min(filtered_options.get().len().saturating_sub(1));
            });
            scroll_into_view(highlighted_index.get());
        },
        "ArrowUp" => {
            ev.prevent_default();
            set_highlighted_index.update(|idx| {
                *idx = idx.saturating_sub(1);
            });
            scroll_into_view(highlighted_index.get());
        },
        "Enter" => {
            ev.prevent_default();
            if let Some(opt) = filtered_options.get().get(highlighted_index.get()) {
                on_select.call(opt.value.clone());
                set_is_open.set(false);
            }
        },
        "Escape" => {
            set_is_open.set(false);
        },
        "Tab" => {
            set_is_open.set(false);
            // Let default tab behavior proceed
        },
        _ => {}
    }
};

// Scroll highlighted item into view
fn scroll_into_view(index: usize) {
    if let Some(element) = document()
        .get_element_by_id(&format!("option-{}", index))
    {
        element.scroll_into_view();
    }
}
```

#### Modal Focus Trap
```rust
use leptos_use::use_focus_trap;

#[component]
pub fn AssociationModal(/* ... */) -> impl IntoView {
    let modal_ref = create_node_ref::<html::Div>();
    
    // Enable focus trap when modal is open
    use_focus_trap(modal_ref, UseFocusTrapOptions::default());
    
    view! {
        <div node_ref=modal_ref role="dialog">
            // Modal content
        </div>
    }
}
```

---

## Priority 3: Screen Reader Support

### Live Regions for Dynamic Content

```rust
// Loading state announcement
<div
    role="status"
    aria-live="polite"
    aria-atomic="true"
    class="sr-only"  // Visually hidden but read by screen readers
>
    {move || if is_loading.get() {
        "Loading options..."
    } else {
        ""
    }}
</div>

// Error announcements
<div
    role="alert"
    aria-live="assertive"
    aria-atomic="true"
    class="sr-only"
>
    {move || error.get().unwrap_or_default()}
</div>

// Selection confirmation
<div
    role="status"
    aria-live="polite"
    class="sr-only"
>
    {move || selected.get().map(|val| format!("{} selected", val))}
</div>
```

### Screen Reader Only Text (CSS)

```css
/* Add to index.css */
.sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    padding: 0;
    margin: -1px;
    overflow: hidden;
    clip: rect(0, 0, 0, 0);
    white-space: nowrap;
    border-width: 0;
}

.sr-only-focusable:focus,
.sr-only-focusable:active {
    position: static;
    width: auto;
    height: auto;
    overflow: visible;
    clip: auto;
    white-space: normal;
}
```

---

## Priority 4: Color Contrast

### WCAG AA Requirements

- **Normal text**: 4.5:1 contrast ratio
- **Large text** (18pt+): 3:1 contrast ratio
- **UI components**: 3:1 contrast ratio

### High-Contrast Mode Support

```css
/* Add to theme */
@media (prefers-contrast: high) {
    :root {
        --color-border: #000;
        --color-bg: #fff;
        --color-text: #000;
    }
    
    .dark {
        --color-border: #fff;
        --color-bg: #000;
        --color-text: #fff;
    }
}
```

### Audit Tool

Use browser DevTools:
1. Open DevTools → Elements
2. Select element
3. Check "Accessibility" pane
4. View contrast ratio
5. Fix any failures

---

## Priority 5: Focus Indicators

### Visible Focus States

```css
/* Enhanced focus indicators */
*:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: 2px;
}

/* Skip to main content */
.skip-to-main {
    position: absolute;
    left: -9999px;
    z-index: 999;
}

.skip-to-main:focus {
    left: 0;
    top: 0;
    padding: 1rem;
    background: var(--color-primary);
    color: white;
}
```

### Skip Links

```rust
// Add to app shell
<a href="#main-content" class="skip-to-main">
    Skip to main content
</a>

<main id="main-content" tabindex="-1">
    // Page content
</main>
```

---

## Testing Checklist

### Manual Testing

- [ ] **Keyboard Only**: Navigate entire app without mouse
- [ ] **Screen Reader**: Test with NVDA (Windows) or VoiceOver (Mac)
- [ ] **Zoom**: Test at 200% zoom level
- [ ] **High Contrast**: Enable Windows High Contrast mode
- [ ] **Color Blindness**: Use color blindness simulator

### Automated Testing

```bash
# Install axe-core
npm install -g @axe-core/cli

# Run accessibility audit
axe http://localhost:8081 --save results.json

# Or use Pa11y
npm install -g pa11y
pa11y http://localhost:8081
```

---

## Component-Specific Implementations

### SignaturePad Accessibility

```rust
<div
    role="application"
    aria-label="Signature drawing area"
>
    <canvas
        aria-label="Draw your signature here using mouse or touch"
        tabindex="0"
        on:keydown=move |ev| {
            // Allow keyboard drawing (arrow keys)
            match ev.key().as_str() {
                "Escape" => clear_signature(),
                _ => {}
            }
        }
    />
    
    <div role="group" aria-label="Signature controls">
        <button aria-label="Clear signature">Clear</button>
        <button aria-label="Save signature">Save</button>
    </div>
</div>
```

### LocationMap Accessibility

```rust
<div role="application" aria-label="Location map">
    <div
        class="map-container"
        aria-label="Interactive map"
        aria-describedby="map-instructions"
    />
    
    <div id="map-instructions" class="sr-only">
        Use the latitude and longitude inputs below to set the location
    </div>
    
    <div role="group" aria-label="Location coordinates">
        <label for="lat-input">Latitude</label>
        <input id="lat-input" type="number" aria-required="true" />
        
        <label for="lng-input">Longitude</label>
        <input id="lng-input" type="number" aria-required="true" />
    </div>
</div>
```

---

## Implementation Priority

### Phase 1 (High Priority - 2 hours)
1. ✅ Add ARIA labels to all form inputs
2. ✅ Add ARIA roles to custom components
3. ✅ Implement keyboard navigation in AsyncSelect
4. ✅ Add focus trap to modals
5. ✅ Add skip links

### Phase 2 (Medium Priority - 1 hour)
6. ✅ Add screen reader announcements
7. ✅ Fix color contrast issues
8. ✅ Add visible focus indicators
9. ✅ Implement high contrast mode

### Phase 3 (Nice to Have - 1 hour)
10. ⚠️ Keyboard navigation in SignaturePad
11. ⚠️ Alternative text for all images
12. ⚠️ Proper heading hierarchy
13. ⚠️ Form validation announcements

---

## Quick Wins (30 minutes)

```rust
// 1. Add aria-label to all buttons without text
<button aria-label="Close">×</button>
<button aria-label="Search"><SearchIcon /></button>

// 2. Add role="status" to loading indicators
<div role="status">{if loading { "Loading..." } else { "" }}</div>

// 3. Add aria-required to required fields
<input aria-required={field.is_required.to_string()} />

// 4. Add aria-invalid to validation errors
<input aria-invalid={has_error.to_string()} />

// 5. Add aria-describedby for help text
<input aria-describedby="field-help" />
<span id="field-help">{help_text}</span>
```

---

## Resources

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [axe DevTools](https://www.deque.com/axe/devtools/)
- [NVDA Screen Reader](https://www.nvaccess.org/)
- [WebAIM Contrast Checker](https://webaim.org/resources/contrastchecker/)

---

**Estimated Total Time**: 4-6 hours for full implementation  
**Quick Wins Time**: 30 minutes for 80% improvement

---

**Version**: 1.0.0  
**Last Updated**: 2024-12-24
