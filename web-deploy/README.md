# Phase Simulation - Web Deployment

This directory contains the complete web-ready deployment of the Phase Transition Simulation.

## Files

- `index.html` - Main web interface
- `phase_simulation.wasm` - Compiled simulation (606KB)
- `gl.js` - WebAssembly loader (66KB)

## Deployment

### Option 1: GitHub Pages
1. Upload these files to a GitHub repository
2. Enable GitHub Pages in repository settings
3. Select the branch and folder containing these files
4. Access via: `https://yourusername.github.io/yourrepo/`

### Option 2: Netlify
1. Drag and drop this folder to [Netlify Drop](https://app.netlify.com/drop)
2. Get instant URL

### Option 3: Vercel
1. Install Vercel CLI: `npm i -g vercel`
2. Run: `vercel` in this directory
3. Follow prompts

### Option 4: Any Static Host
Upload all three files to any static file hosting service that supports:
- MIME type `application/wasm` for `.wasm` files
- MIME type `application/javascript` for `.js` files

## Local Testing

```bash
python3 -m http.server 8000
# Then open http://localhost:8000
```

## Browser Requirements

- Modern browser with WebAssembly support
- Chrome 57+, Firefox 52+, Safari 11+, Edge 16+

## Controls

- **↑/↓** - Adjust Temperature
- **←/→** - Adjust Chemical Potential  
- **Space** - Randomize Grid
- **M** - Cycle display modes
- **D** - Toggle density plot
- **S** - Save CSV data

## File Size

Total: ~680KB (compresses to ~200KB with gzip)

