# AMP Console UI

Professional cyberpunk/industrial themed React UI for the Agentic Memory Protocol Console.

## Design Philosophy

- **Industrial Cyberpunk Aesthetic**: Dark backgrounds (#09090b, #18181b) with red accent (#ef4444)
- **No Emojis**: Uses react-icons for professional iconography
- **Left Sidebar Navigation**: Clean, icon-based navigation with tooltips
- **Sharp Corners**: Industrial feel with minimal border radius
- **Grid Textures**: Subtle background patterns for depth
- **Glass Panels**: Backdrop blur effects for modern UI

## Tech Stack

- **React 18** - UI framework
- **TypeScript** - Type safety
- **Tailwind CSS** - Utility-first styling
- **React Icons** - Professional icon library
- **Vite** - Build tool
- **Tauri** - Desktop app framework

## Project Structure

```
src/
├── components/
│   ├── Sidebar.tsx       # Left navigation sidebar
│   ├── Header.tsx        # Top header with tabs
│   ├── FileExplorer.tsx  # File browser view
│   ├── KnowledgeGraph.tsx # Graph visualization
│   └── Analytics.tsx     # Analytics dashboard
├── App.tsx               # Main app component
├── main.tsx             # Entry point
└── index.css            # Global styles
```

## Views

### 1. File Explorer
- Left sidebar with file tree
- Main content area with file list
- Breadcrumb navigation
- Search functionality
- Grid/List view toggle

### 2. Knowledge Graph
- Interactive node visualization
- Connection lines with animations
- Control panel for graph settings
- Stats panel (nodes, edges, depth)
- Terminal output log

### 3. Analytics
- System metrics cards
- Request latency chart
- Error distribution
- System events log
- Time range selector

## Color Palette

```css
primary: #ef4444          /* Industrial Red */
primary-glow: #b91c1c     /* Dark Red */
background-dark: #09090b  /* Obsidian */
panel-dark: #18181b       /* Charcoal */
border-dark: #27272a      /* Metallic Grey */
code-bg: #0c0a09         /* Code Background */
```

## Development

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Run Tauri desktop app
npm run tauri:dev

# Build Tauri app
npm run tauri:build
```

## Key Features

- **Responsive Design**: Works on desktop and large screens
- **Custom Scrollbars**: Thin, industrial-styled scrollbars
- **Hover Effects**: Smooth transitions and red accent highlights
- **Status Indicators**: Animated pulse effects for live status
- **Professional Typography**: Inter for UI, JetBrains Mono for code
- **Accessibility**: Proper contrast ratios and focus states

## Customization

The design system is built with Tailwind CSS. To customize:

1. Edit `tailwind.config.js` for colors and theme
2. Modify `index.css` for global styles
3. Update component styles inline with Tailwind classes

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+

## License

Part of the AMP project.
