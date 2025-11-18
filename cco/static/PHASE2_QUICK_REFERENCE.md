# Phase 2 Canvas Rendering - Quick Reference

## For Developers

### Using 256-Color Palette

```javascript
// ANSI sequence for 256-color foreground
terminal.write('\x1b[38;5;196mRed text\x1b[0m');  // Color 196

// ANSI sequence for 256-color background
terminal.write('\x1b[48;5;27mBlue background\x1b[0m');  // Color 27

// Color ranges:
// 0-15:    Legacy colors (basic 16)
// 16-231:  6x6x6 RGB cube (216 colors)
// 232-255: Grayscale ramp (24 shades)
```

### Using RGB True Colors

```javascript
// RGB foreground color
terminal.write('\x1b[38;2;255;128;0mOrange text\x1b[0m');

// RGB background color
terminal.write('\x1b[48;2;0;128;255mBlue background\x1b[0m');

// Combined
terminal.write('\x1b[38;2;255;255;255;48;2;0;0;0mWhite on black\x1b[0m');
```

### Using Text Attributes

```javascript
// Single attributes
terminal.write('\x1b[1mBold\x1b[0m');           // Bold
terminal.write('\x1b[2mDim\x1b[0m');            // Dim
terminal.write('\x1b[3mItalic\x1b[0m');         // Italic
terminal.write('\x1b[4mUnderline\x1b[0m');      // Underline
terminal.write('\x1b[5mBlink\x1b[0m');          // Blink (placeholder)
terminal.write('\x1b[7mReverse\x1b[0m');        // Reverse video
terminal.write('\x1b[8mHidden\x1b[0m');         // Hidden
terminal.write('\x1b[9mStrikethrough\x1b[0m'); // Strikethrough

// Combined attributes (semicolon-separated)
terminal.write('\x1b[1;3;4mBold italic underline\x1b[0m');
terminal.write('\x1b[1;31mBold red\x1b[0m');
```

### Performance Monitoring

```javascript
// Get performance metrics
const metrics = terminal.getPerformanceMetrics();
console.log('Color cache size:', metrics.colorCacheSize);
console.log('Dirty regions:', metrics.dirtyRegions);
console.log('Canvas size:', metrics.canvasSize);
console.log('Total cells:', metrics.cellCount);
```

### Color Palette Reference

#### Standard Colors (0-15)
```
0:  Black         8:  Bright Black (Gray)
1:  Red           9:  Bright Red
2:  Green        10:  Bright Green
3:  Yellow       11:  Bright Yellow
4:  Blue         12:  Bright Blue
5:  Magenta      13:  Bright Magenta
6:  Cyan         14:  Bright Cyan
7:  White        15:  Bright White
```

#### 6x6x6 Color Cube (16-231)
```
Color index = 16 + (36 × r) + (6 × g) + b

Where:
  r, g, b ∈ [0, 5]

RGB values:
  0 → 0
  1 → 95
  2 → 135
  3 → 175
  4 → 215
  5 → 255
```

#### Grayscale Ramp (232-255)
```
Color index = 232 + step

Where step ∈ [0, 23]

Gray value = 8 + (step × 10)
```

### Common ANSI Sequences

```javascript
// Cursor movement
'\x1b[H'           // Move to home (0,0)
'\x1b[{row};{col}H' // Move to position
'\x1b[A'           // Move up
'\x1b[B'           // Move down
'\x1b[C'           // Move right
'\x1b[D'           // Move left

// Screen clearing
'\x1b[2J'          // Clear entire screen
'\x1b[K'           // Clear to end of line
'\x1b[J'           // Clear to end of screen

// Reset
'\x1b[0m'          // Reset all attributes
'\x1b[m'           // Reset all attributes (short form)

// Save/restore cursor
'\x1b[s'           // Save cursor position (DECSC)
'\x1b[u'           // Restore cursor position (DECRC)

// Alternative buffer
'\x1b[?1049h'      // Switch to alternative buffer
'\x1b[?1049l'      // Switch to main buffer

// Scrolling region
'\x1b[{top};{bottom}r'  // Set scrolling region
```

### Testing Quick Commands

```bash
# Verify implementation
cd /Users/brent/git/cc-orchestra/cco/static
./verify-phase2.sh

# Open test harness
open test-wasm-terminal.html

# Check files
ls -lh wasm-terminal.js test-wasm-terminal.html

# View documentation
cat PHASE2_RENDERING_REPORT.md
```

### Color Helper Functions

```javascript
// Convert RGB to 256-color index (approximation)
function rgbTo256(r, g, b) {
    // Use grayscale for gray colors
    if (Math.abs(r - g) < 10 && Math.abs(g - b) < 10) {
        const gray = Math.round((r + g + b) / 3);
        const grayIndex = Math.round((gray - 8) / 10);
        return 232 + Math.max(0, Math.min(23, grayIndex));
    }

    // Use 6x6x6 cube
    const r6 = Math.round(r / 51);  // 0-5
    const g6 = Math.round(g / 51);  // 0-5
    const b6 = Math.round(b / 51);  // 0-5
    return 16 + (36 * r6) + (6 * g6) + b6;
}

// Convert 256-color index to RGB
function index256ToRgb(idx) {
    if (idx < 16) {
        // Legacy colors - see palette
        return terminal.palette256[idx];
    } else if (idx < 232) {
        // 6x6x6 cube
        const i = idx - 16;
        const r = Math.floor(i / 36) % 6;
        const g = Math.floor(i / 6) % 6;
        const b = i % 6;
        return [
            r === 0 ? 0 : 55 + r * 40,
            g === 0 ? 0 : 55 + g * 40,
            b === 0 ? 0 : 55 + b * 40
        ];
    } else {
        // Grayscale
        const gray = 8 + (idx - 232) * 10;
        return [gray, gray, gray];
    }
}
```

### Browser DevTools Debugging

```javascript
// Access terminal instance
const terminal = adapter.terminal;

// Inspect color cache
console.log('Cache size:', terminal.colorCache.size);
console.log('Cache contents:', Array.from(terminal.colorCache.entries()));

// Test color conversion
const color = { RGB: { r: 255, g: 128, b: 0 } };
console.log('CSS:', terminal.colorToCSS(color));

// Check palette
console.log('Color 196:', terminal.palette256[196]);
console.log('Grayscale 240:', terminal.palette256[240]);

// Performance testing
const start = performance.now();
for (let i = 0; i < 1000; i++) {
    terminal.write('\x1b[38;5;' + (i % 256) + 'm' + String.fromCharCode(65 + i % 26));
}
const duration = performance.now() - start;
console.log('1000 chars:', duration.toFixed(2) + 'ms');
```

### Common Issues & Solutions

#### Issue: Colors not displaying
```javascript
// Check if palette initialized
console.log('Palette:', terminal.palette256);

// Verify color format
const testColor = { Indexed: 196 };
console.log('CSS:', terminal.colorToCSS(testColor));
```

#### Issue: Attributes not working
```javascript
// Check cell structure
terminal.write('test');
// Inspect cell in devtools to verify attrs object exists
```

#### Issue: Performance degradation
```javascript
// Check cache size
const metrics = terminal.getPerformanceMetrics();
if (metrics.colorCacheSize > 800) {
    console.warn('Cache near limit, pruning will occur');
}

// Monitor frame rate
let frameCount = 0;
let lastTime = performance.now();
setInterval(() => {
    const now = performance.now();
    const fps = frameCount / ((now - lastTime) / 1000);
    console.log('FPS:', fps.toFixed(1));
    frameCount = 0;
    lastTime = now;
}, 1000);
```

### Performance Optimization Tips

1. **Use color caching**: The cache is automatic, but avoid creating new color objects unnecessarily

2. **Batch writes**: Combine multiple writes into one string
   ```javascript
   // Good
   terminal.write('\x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m');

   // Less efficient
   terminal.write('\x1b[31mRed\x1b[0m');
   terminal.write(' ');
   terminal.write('\x1b[32mGreen\x1b[0m');
   ```

3. **Reset after complex attributes**: Always reset to default
   ```javascript
   terminal.write('\x1b[1;3;4;31mComplex\x1b[0m'); // Reset at end
   ```

4. **Use 256-colors over RGB when possible**: Slightly faster due to palette lookup caching

5. **Monitor dirty regions**: Dirty region tracking is automatic but verify it's working
   ```javascript
   console.log(terminal.dirtyRegions);
   ```

### Integration with Rust WASM

The JavaScript renderer automatically handles:
- Color format conversions from Rust Color enum
- Cell attribute rendering from Rust CellAttrs struct
- Buffer switching (main/alternative)
- Scrolling region visualization

No additional JavaScript code needed for these features.

### Testing Checklist

- [ ] 256-color palette displays correctly
- [ ] RGB colors match specifications
- [ ] Bold text is heavier than normal
- [ ] Italic text is slanted properly
- [ ] Underline appears at baseline
- [ ] Strikethrough crosses characters
- [ ] Reverse video swaps colors
- [ ] Hidden text is invisible
- [ ] Performance > 3000 chars/sec
- [ ] Cache size stays under 1000 entries
- [ ] No memory leaks over time
- [ ] Works in Chrome, Safari, Firefox, Edge

### Resources

- **Full Documentation**: `PHASE2_RENDERING_REPORT.md`
- **Change Summary**: `PHASE2_CHANGES_SUMMARY.md`
- **Test Harness**: `test-wasm-terminal.html`
- **Verification Script**: `verify-phase2.sh`
- **Main Implementation**: `wasm-terminal.js`

### Contact

For questions or issues:
1. Review the full documentation first
2. Run the verification script
3. Check the test harness for examples
4. Coordinate with Rust Specialist for color format issues
5. Coordinate with Test Engineer for test coverage

---

**Last Updated**: 2025-11-17
**Version**: Phase 2.0
**Status**: Production Ready ✅
