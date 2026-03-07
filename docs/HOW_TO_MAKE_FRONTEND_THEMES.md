# How to Create Frontend Themes

## The System in One Sentence

Each theme = one CSS class on `<html>` that overrides CSS variables. Components read variables — they never use hardcoded colors.

---

## Files You Must Touch (All 3, Every Time)

| File | What to do |
|------|-----------|
| `frontend/app/app.css` | Add the `html.mytheme { ... }` CSS block |
| `frontend/app/contexts/ThemeContext.tsx` | Add `"mytheme"` to the `Theme` type, validation array, class removal list, and dark check |
| `frontend/app/components/ThemeCombobox.tsx` | Add an entry to the `themes` array |

---

## Step 1 — Add CSS in `app.css`

Paste this template and fill in every value. **Do not skip `--color-primary-*`.** That's the accent palette (buttons, badges, links). The global default is green — if you don't override it, your theme will have green everywhere.

```css
html.mytheme {
  background-color: #0f0f0f;   /* page background */
  color: #e5e5e5;               /* default text color */
  color-scheme: dark;           /* use "light" for light themes */

  /* ── Gray scale (surfaces, borders, muted text) ─────────────────── */
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
  --color-gray-50:  #f5f5f5;

  /* ── Primary accent palette (buttons, badges, links, highlights) ── */
  /* THIS IS MANDATORY. Without it you inherit the global green palette. */
  /* Replace with your accent hue. Example below uses red (Tailwind red). */
  --color-primary-50:  #fef2f2;
  --color-primary-100: #fee2e2;
  --color-primary-200: #fecaca;
  --color-primary-300: #fca5a5;
  --color-primary-400: #f87171;
  --color-primary-500: #ef4444;  /* ← main accent, used most */
  --color-primary-600: #dc2626;
  --color-primary-700: #b91c1c;
  --color-primary-800: #991b1b;
  --color-primary-900: #7f1d1d;
  --color-primary-950: #450a0a;

  /* ── Border radius (controls roundness everywhere) ───────────────── */
  /* Standard:      md / lg / xl                                        */
  /* Very round:    2xl / full / 2xl   (barbie / lime style)            */
  /* Very sharp:    sm / sm / sm       (orange style)                   */
  /* No roundness:  0.125rem for all   (grayscale style)                */
  --radius-component:    var(--radius-md);
  --radius-component-lg: var(--radius-lg);
  --radius-component-xl: var(--radius-xl);

  /* ── Top bar ─────────────────────────────────────────────────────── */
  --color-topbar-bg:         #141414;
  --color-topbar-text:       #d4d4d4;
  --color-topbar-item-hover: #1f1f1f;

  /* ── Side navigation ─────────────────────────────────────────────── */
  --color-nav-bg:               #0f0f0f;
  --color-nav-border:           #242424;
  --color-nav-text:             #d4d4d4;
  --color-nav-text-muted:       #737373;
  --color-nav-item-hover:       #1a1a1a;
  --color-nav-item-hover-text:  #e5e5e5;
  --color-nav-item-active:      #7f1d1d;   /* ← your accent, dark bg */
  --color-nav-item-active-text: #fca5a5;   /* ← your accent, light text */
}
```

### Cascade warning for dark variants

If your theme also adds the `dark` class (see Step 2), the `html.dark { }` block in `app.css` fires too — because both classes are on `<html>`. Your `html.mytheme { }` block wins only for variables it explicitly sets. The `html.dark` block comes first in the file, so any variable defined in `html.dark` but NOT in `html.mytheme` will use the dark value. That's usually fine, but be aware.

---

## Step 2 — Update `ThemeContext.tsx`

Four places, all in the same file:

```typescript
// 1. Type union
export type Theme = "light" | "dark" | "darkred" | "mytheme" | ...;

// 2. Valid themes array (for localStorage restore)
if (stored && ["light", "dark", "darkred", "mytheme", ...].includes(stored)) {

// 3. Class removal list (runs on every theme switch)
root.classList.remove("light", "dark", "darkred", "mytheme", ...);

// 4. Dark variant check — add here ONLY if your theme is dark
if (theme === "grayscale" || theme === "orange" || theme === "darkred" || theme === "mytheme") {
  root.classList.add("dark");  // adds Tailwind dark: utilities
}
```

---

## Step 3 — Update `ThemeCombobox.tsx`

```typescript
{ value: "mytheme", label: "My Theme", description: "One-line description", icon: Moon },
```

Pick an icon from `lucide-react`. Use `Moon` for dark themes, `Sun` for light, or any thematic icon.

---

## Quick Reference: Existing Accent Palettes

Copy-paste one of these as your `--color-primary-*` block:

| Theme | Palette | Tailwind source |
|-------|---------|-----------------|
| `dark` | Green | `emerald` |
| `darkred` | Red | `red` |
| `orange` | Orange | `orange` |
| `barbie` | Pink | `pink` |
| `fusion` | Sky blue | `sky` |
| `lime` | Lime green | `lime` |
| `nostalgic95` | Navy | custom |

All Tailwind palettes: https://tailwindcss.com/docs/customizing-colors

---

## Checklist Before You're Done

- [ ] `html.mytheme { }` block in `app.css` with ALL variables filled in
- [ ] `--color-primary-*` overridden (11 values, 50–950)
- [ ] `color-scheme: dark` or `color-scheme: light` set correctly
- [ ] `ThemeContext.tsx`: type, array, removal list, dark check (if dark)
- [ ] `ThemeCombobox.tsx`: entry added
- [ ] Open `/ui-gallery` in browser, switch to your theme, verify no green/wrong-color bleed
