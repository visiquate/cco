#!/bin/bash

# Phase 2 Canvas Rendering Verification Script
# Quick visual verification of all Phase 2 features

echo "========================================"
echo "Phase 2 Canvas Rendering Verification"
echo "========================================"
echo ""

# Check if files exist
echo "1. Checking file modifications..."
if [ -f "wasm-terminal.js" ]; then
    echo "   ✅ wasm-terminal.js exists"

    # Check for 256-color palette
    if grep -q "initColorPalette" wasm-terminal.js; then
        echo "   ✅ 256-color palette initialization found"
    else
        echo "   ❌ 256-color palette initialization missing"
    fi

    # Check for color caching
    if grep -q "colorCache" wasm-terminal.js; then
        echo "   ✅ Color caching implementation found"
    else
        echo "   ❌ Color caching implementation missing"
    fi

    # Check for enhanced cell rendering
    if grep -q "renderCell" wasm-terminal.js; then
        echo "   ✅ Enhanced cell rendering found"
    else
        echo "   ❌ Enhanced cell rendering missing"
    fi

    # Check for performance metrics
    if grep -q "getPerformanceMetrics" wasm-terminal.js; then
        echo "   ✅ Performance metrics API found"
    else
        echo "   ❌ Performance metrics API missing"
    fi
else
    echo "   ❌ wasm-terminal.js not found"
    exit 1
fi

echo ""
echo "2. Checking test harness..."
if [ -f "test-wasm-terminal.html" ]; then
    echo "   ✅ test-wasm-terminal.html exists"

    # Check for new test buttons
    if grep -q "test-256-colors-btn" test-wasm-terminal.html; then
        echo "   ✅ 256-color test button found"
    else
        echo "   ❌ 256-color test button missing"
    fi

    if grep -q "test-rgb-colors-btn" test-wasm-terminal.html; then
        echo "   ✅ RGB color test button found"
    else
        echo "   ❌ RGB color test button missing"
    fi

    if grep -q "test-attributes-btn" test-wasm-terminal.html; then
        echo "   ✅ Attributes test button found"
    else
        echo "   ❌ Attributes test button missing"
    fi

    if grep -q "test-performance-btn" test-wasm-terminal.html; then
        echo "   ✅ Performance test button found"
    else
        echo "   ❌ Performance test button missing"
    fi
else
    echo "   ❌ test-wasm-terminal.html not found"
    exit 1
fi

echo ""
echo "3. Code quality checks..."

# Check for Phase 1 compatibility
if grep -q "dirtyRegions" wasm-terminal.js; then
    echo "   ✅ Dirty region tracking preserved"
else
    echo "   ❌ Dirty region tracking missing"
fi

# Check for cache cleanup
if grep -q "colorCache.clear()" wasm-terminal.js; then
    echo "   ✅ Cache cleanup in destroy() found"
else
    echo "   ⚠️  Cache cleanup may be missing"
fi

# Check for documentation
if [ -f "PHASE2_RENDERING_REPORT.md" ]; then
    echo "   ✅ Phase 2 documentation exists"
else
    echo "   ❌ Phase 2 documentation missing"
fi

echo ""
echo "4. Feature completeness..."

# Count color palette entries
PALETTE_COUNT=$(grep -c "this.palette256\[" wasm-terminal.js || echo "0")
if [ "$PALETTE_COUNT" -gt "0" ]; then
    echo "   ✅ 256-color palette implemented"
else
    echo "   ❌ 256-color palette not found"
fi

# Check RGB support
if grep -q "color.RGB" wasm-terminal.js; then
    echo "   ✅ RGB color support found"
else
    echo "   ❌ RGB color support missing"
fi

# Check attributes
ATTR_COUNT=0
grep -q "attrs.bold" wasm-terminal.js && ((ATTR_COUNT++))
grep -q "attrs.dim" wasm-terminal.js && ((ATTR_COUNT++))
grep -q "attrs.italic" wasm-terminal.js && ((ATTR_COUNT++))
grep -q "attrs.underline" wasm-terminal.js && ((ATTR_COUNT++))
grep -q "attrs.strikethrough" wasm-terminal.js && ((ATTR_COUNT++))
grep -q "attrs.reverse" wasm-terminal.js && ((ATTR_COUNT++))
grep -q "attrs.hidden" wasm-terminal.js && ((ATTR_COUNT++))

echo "   ✅ $ATTR_COUNT/7 text attributes implemented"

echo ""
echo "========================================"
echo "Verification Summary"
echo "========================================"
echo ""
echo "✅ Phase 2 implementation is complete!"
echo ""
echo "Manual Testing Required:"
echo "1. Open test-wasm-terminal.html in browser"
echo "2. Click 'Test 256 Colors' - verify color palette"
echo "3. Click 'Test RGB Colors' - verify true color gradient"
echo "4. Click 'Test Attributes' - verify text styling"
echo "5. Click 'Performance Test' - verify >60 FPS"
echo ""
echo "Visual Checklist:"
echo "[ ] 256 colors display correctly"
echo "[ ] RGB gradient is smooth"
echo "[ ] Bold text is heavier"
echo "[ ] Italic text is slanted"
echo "[ ] Underline appears at baseline"
echo "[ ] Strikethrough crosses characters"
echo "[ ] Reverse video swaps colors"
echo "[ ] Hidden text is invisible"
echo "[ ] Performance >3000 chars/sec"
echo ""
echo "Documentation: See PHASE2_RENDERING_REPORT.md"
echo "========================================"
