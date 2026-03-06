# How to Create Frontend Themes

This document explains how to create new themes for the Critical frontend application, using the newly added `darkred` theme as a reference.

## Theme Architecture

The Critical frontend uses a CSS-based theming system where each theme is defined as a CSS class on the `<html>` element. The themes are defined in `/frontend/app/app.css` and managed through React context in `/frontend/app/contexts/ThemeContext.tsx`.

## Creating a New Theme

### 1. Add CSS Definitions

In `/frontend/app/app.css`, add a new CSS class for your theme following the existing pattern:

```css
html.mynewtheme {
  background-color: #0f0f0f;
  color: #e5e5e5;
  color-scheme: dark;
  
  /* Define all color variables */
  --color-gray-950: #080808;
  --color-gray-900: #111111;
  --color-gray-800: #1c1c1c;
  --color-gray-700: #2a2a2a;
  --color-gray-600: #3d3d3d;
  --color-gray-500: #525252;
  --color-gray-400: #737373;
  --color-gray-300: #a3a3a3;
  --color-gray-200: #d4d4d4;
  --color-gray-100: #e5e5e5;
  --color-gray-50: #f5f5f5;

  /* Roundness variables */
  --radius-component: var(--radius-md);
  --radius-component-lg: var(--radius-lg);
  --radius-component-xl: var(--radius-xl);

  /* Theme-specific colors */
  --color-topbar-bg: #141414;
  --color-topbar-text: #d4d4d4;
  --color-topbar-item-hover: #1f1f1f;
  --color-nav-bg: #0f0f0f;
  --color-nav-border: #242424;
  --color-nav-text: #d4d4d4;
  --color-nav-text-muted: #737373;
  --color-nav-item-hover: #1a1a1a;
  --color-nav-item-hover-text: #e5e5e5;
  --color-nav-item-active: #064e3b; /* Green for default dark theme */
  --color-nav-item-active-text: #6ee7b7; /* Green text for default dark theme */
}
```

### 2. Update ThemeContext.tsx

Update `/frontend/app/contexts/ThemeContext.tsx` to include your new theme:

```typescript
// Add to the Theme type definition
export type Theme = "light" | "dark" | "darkred" | "mynewtheme" | "grayscale" | "barbie" | "orange" | "fusion" | "nostalgic95" | "itheme" | "lime";

// Add to the valid themes array
if (stored && ["light", "dark", "darkred", "mynewtheme", "grayscale", "barbie", "orange", "fusion", "nostalgic95", "itheme", "lime"].includes(stored)) {
  return stored;
}

// Add to the class removal list
root.classList.remove("light", "dark", "darkred", "mynewtheme", "grayscale", "barbie", "orange", "fusion", "nostalgic95", "itheme", "lime");

// Add to dark variant check
if (theme === "grayscale" || theme === "orange" || theme === "darkred" || theme === "mynewtheme") {
  root.classList.add("dark");
}
```

### 3. Update Theme Selector

Update `/frontend/app/components/ThemeCombobox.tsx` to include your new theme in the UI:

```typescript
const themes: ThemeOption[] = [
  // ... existing themes ...
  {
    value: "mynewtheme",
    label: "My New Theme",
    description: "Description of your theme",
    icon: Moon, // or appropriate icon
  },
  // ... rest of themes ...
];
```

## Theme Guidelines

1. **Consistency**: Follow the same color variable naming and structure as existing themes
2. **Dark Variants**: If your theme is dark, make sure to add it to the dark variant check so it gets the `dark` class applied
3. **Color Scheme**: Set `color-scheme` property appropriately (light or dark)
4. **Roundness**: Use the standard roundness CSS variables for consistent UI elements
5. **Testing**: Ensure your theme works across all components and doesn't break existing functionality

## Example: Creating a Red Theme

The `darkred` theme implementation shows exactly how to create a variant of an existing theme with different accent colors:

1. Copy the CSS structure from the base theme (like `html.dark`)
2. Change only the accent colors that differ (like `--color-nav-item-active` and `--color-nav-item-active-text`)
3. Keep all other color variables the same to maintain consistency
4. Update all TypeScript files to include the new theme name

This approach allows for quick theme creation while maintaining the overall design system.