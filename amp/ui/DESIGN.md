# AMP Console UI Design System

## Overview
Professional cyberpunk/industrial themed interface for the Agentic Memory Protocol Console.

## Design Principles

### 1. Industrial Cyberpunk Aesthetic
- Dark, moody backgrounds
- Red accent color (#ef4444) for primary actions
- Sharp corners (minimal border radius)
- Grid textures for depth
- Glass panel effects

### 2. Professional Iconography
- **No emojis** - Uses react-icons library
- Material Design icons for consistency
- Icon-only navigation with tooltips
- Proper sizing and spacing

### 3. Layout Structure
```
┌─────────────────────────────────────────┐
│           Header (56px)                 │
├────┬────────────────────────────────────┤
│    │                                    │
│ S  │         Main Content               │
│ i  │                                    │
│ d  │                                    │
│ e  │                                    │
│ b  │                                    │
│ a  │                                    │
│ r  │                                    │
│    │                                    │
│ 16 │                                    │
│ px │                                    │
├────┴────────────────────────────────────┤
│           Footer (32px)                 │
└─────────────────────────────────────────┘
```

## Color Palette

### Primary Colors
```css
--primary: #ef4444          /* Industrial Red */
--primary-glow: #b91c1c     /* Dark Red Glow */
--primary-dark: #7f1d1d     /* Rust Accent */
```

### Background Colors
```css
--background-dark: #09090b  /* Obsidian - Main BG */
--panel-dark: #18181b       /* Charcoal - Panels */
--code-bg: #0c0a09         /* Code Background */
```

### Border Colors
```css
--border-dark: #27272a      /* Metallic Grey */
--border-subtle: #ffffff0d  /* White 5% */
```

### Text Colors
```css
--text-primary: #ffffff     /* White */
--text-secondary: #cbd5e1   /* Slate 300 */
--text-muted: #94a3b8      /* Slate 400 */
--text-dim: #64748b        /* Slate 500 */
```

### Status Colors
```css
--success: #22c55e         /* Green */
--warning: #f59e0b         /* Amber */
--error: #ef4444           /* Red */
--info: #3b82f6            /* Blue */
```

## Typography

### Font Families
```css
--font-display: 'Inter', sans-serif;
--font-mono: 'JetBrains Mono', monospace;
```

### Font Sizes
```css
--text-xs: 10px    /* Labels, metadata */
--text-sm: 12px    /* Body text */
--text-base: 14px  /* Default */
--text-lg: 16px    /* Headings */
--text-xl: 20px    /* Large headings */
--text-2xl: 24px   /* Page titles */
```

## Components

### Sidebar
- **Width**: 16px (64px)
- **Background**: panel-dark with gradient overlay
- **Icons**: 20px size
- **Active State**: Red glow with border
- **Hover**: Red tint on background

### Header
- **Height**: 56px (14 * 4)
- **Background**: panel-dark/80 with backdrop blur
- **Border**: Bottom border-dark
- **Shadow**: Large shadow for depth

### Buttons
```css
/* Primary Button */
.btn-primary {
  background: var(--primary);
  color: white;
  box-shadow: 0 0 10px rgba(239, 68, 68, 0.3);
}

/* Secondary Button */
.btn-secondary {
  background: transparent;
  border: 1px solid var(--border-dark);
  color: var(--text-secondary);
}

/* Icon Button */
.btn-icon {
  padding: 8px;
  border-radius: 4px;
  color: var(--text-muted);
}
```

### Cards
```css
.card {
  background: linear-gradient(145deg, #1c1917 0%, #0c0a09 100%);
  border: 1px solid var(--border-dark);
  border-left: 4px solid var(--primary);
  padding: 20px;
}
```

### Inputs
```css
.input {
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid var(--border-dark);
  color: var(--text-secondary);
  padding: 6px 16px;
}

.input:focus {
  border-color: var(--primary);
  ring: 1px solid var(--primary);
}
```

## Effects

### Shadows
```css
--shadow-neon-red: 0 0 10px rgba(239, 68, 68, 0.3), 
                   0 0 20px rgba(239, 68, 68, 0.1);
--shadow-inner-red: inset 0 0 20px rgba(239, 68, 68, 0.05);
```

### Animations
```css
/* Pulse */
@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

/* Glow */
.glow-red {
  text-shadow: 0 0 10px rgba(239, 68, 68, 0.5);
}
```

### Transitions
```css
--transition-fast: 150ms ease;
--transition-base: 200ms ease;
--transition-slow: 300ms ease;
```

## Patterns

### Grid Texture
```css
.grid-texture {
  background-image: 
    linear-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(255, 255, 255, 0.03) 1px, transparent 1px);
  background-size: 24px 24px;
}
```

### Glass Panel
```css
.glass-panel {
  backdrop-filter: blur(8px);
  background: rgba(24, 24, 27, 0.9);
  border: 1px solid rgba(220, 38, 38, 0.4);
}
```

### Noise Texture
```css
.noise-texture {
  background-image: url('data:image/svg+xml,...');
  opacity: 0.4;
  mix-blend-mode: overlay;
}
```

## Spacing Scale

```css
--space-1: 4px
--space-2: 8px
--space-3: 12px
--space-4: 16px
--space-5: 20px
--space-6: 24px
--space-8: 32px
--space-10: 40px
--space-12: 48px
--space-16: 64px
```

## Border Radius

```css
--radius-none: 0px
--radius-sm: 2px
--radius-default: 4px
--radius-md: 6px
--radius-lg: 8px
--radius-full: 9999px
```

## Z-Index Scale

```css
--z-base: 0
--z-sidebar: 10
--z-header: 20
--z-dropdown: 30
--z-overlay: 40
--z-modal: 50
--z-tooltip: 60
```

## Responsive Breakpoints

```css
--screen-sm: 640px
--screen-md: 768px
--screen-lg: 1024px
--screen-xl: 1280px
--screen-2xl: 1536px
```

## Usage Examples

### File List Item
```tsx
<div className="px-6 py-3 grid grid-cols-12 gap-4 items-center 
                hover:bg-primary/5 hover:border-l-2 hover:border-primary 
                transition-all group cursor-pointer">
  <div className="col-span-6 flex items-center">
    <HiCode className="text-red-500 mr-3" />
    <span className="text-sm font-medium text-slate-300 
                     group-hover:text-white">
      App.tsx
    </span>
  </div>
</div>
```

### Status Badge
```tsx
<span className="px-2 py-0.5 border border-primary bg-primary 
                 text-black font-bold text-[10px] uppercase 
                 shadow-[0_0_10px_rgba(239,68,68,0.4)] animate-pulse">
  Alert
</span>
```

### Metric Card
```tsx
<div className="bg-gradient-to-br from-[#1c1917] to-[#0c0a09] 
                border border-stone-800 p-5 border-l-4 
                border-l-primary shadow-lg">
  <span className="text-stone-500 text-xs uppercase">
    Active Sessions
  </span>
  <div className="text-3xl font-bold text-stone-100">
    1,284
  </div>
</div>
```

## Accessibility

### Contrast Ratios
- Primary text on dark background: 15:1 (AAA)
- Secondary text on dark background: 7:1 (AA)
- Primary red on dark background: 4.5:1 (AA)

### Focus States
All interactive elements have visible focus states:
```css
.focus-visible {
  outline: 2px solid var(--primary);
  outline-offset: 2px;
}
```

### Keyboard Navigation
- Tab order follows visual hierarchy
- All buttons accessible via keyboard
- Tooltips appear on focus

## Performance

### Optimization Techniques
1. **CSS-in-JS avoided** - Using Tailwind for better performance
2. **Minimal animations** - Only where it adds value
3. **Lazy loading** - Components loaded on demand
4. **Memoization** - React.memo for expensive components

### Bundle Size
- React Icons: Tree-shakeable (only imports used icons)
- Tailwind: Purged unused classes in production
- Total bundle: ~150KB gzipped

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- No IE11 support

## Future Enhancements

1. **Dark/Light Mode Toggle** - Add theme switcher
2. **Custom Themes** - Allow user color customization
3. **Animations** - More micro-interactions
4. **Responsive** - Mobile/tablet layouts
5. **Accessibility** - ARIA labels, screen reader support
