#!/bin/bash

echo "🔍 Dashboard Fixes Final Verification"
echo ""

DASHBOARD_JS="cco/static/dashboard.js"
PASS=0
FAIL=0

# Test 1: updateLastUpdateTime has timestamp parameter
if grep -q "function updateLastUpdateTime(timestamp)" "$DASHBOARD_JS"; then
    echo "✅ updateLastUpdateTime() accepts timestamp parameter"
    ((PASS++))
else
    echo "❌ updateLastUpdateTime() missing timestamp parameter"
    ((FAIL++))
fi

# Test 2: updateLastUpdateTime updates DOM
if grep -q "projectLastUpdate" "$DASHBOARD_JS" && grep -q "machineLastUpdate" "$DASHBOARD_JS"; then
    echo "✅ updateLastUpdateTime() updates both DOM elements"
    ((PASS++))
else
    echo "❌ updateLastUpdateTime() doesn't update DOM"
    ((FAIL++))
fi

# Test 3: Handles relative time display
if grep -q "Just now\|minute.*ago\|hour.*ago" "$DASHBOARD_JS"; then
    echo "✅ updateLastUpdateTime() formats relative time"
    ((PASS++))
else
    echo "❌ Relative time formatting missing"
    ((FAIL++))
fi

# Test 4: Handles array format
if grep -q "Array.isArray(data.activity)" "$DASHBOARD_JS"; then
    echo "✅ handleAnalyticsUpdate() handles array activity format"
    ((PASS++))
else
    echo "❌ Array format handling missing"
    ((FAIL++))
fi

# Test 5: Legacy format fallback
if grep -q "} else {" "$DASHBOARD_JS" && grep -q "addActivity(data.activity)" "$DASHBOARD_JS"; then
    echo "✅ handleAnalyticsUpdate() has legacy fallback"
    ((PASS++))
else
    echo "❌ Legacy fallback missing"
    ((FAIL++))
fi

# Test 6: Updates activity table immediately
if grep -q "updateActivityTable()" "$DASHBOARD_JS"; then
    # Check it appears after activity handling
    if grep -A 10 "Handle activity" "$DASHBOARD_JS" | grep -q "updateActivityTable()"; then
        echo "✅ Activity table updates immediately with new data"
        ((PASS++))
    else
        echo "❌ Activity table update logic issue"
        ((FAIL++))
    fi
else
    echo "❌ updateActivityTable() call missing"
    ((FAIL++))
fi

# Test 7: Field name variations in activity (check for both type formats)
if grep -q "item.type || item.event_type" "$DASHBOARD_JS" || grep -q "item.event_type || item.type" "$DASHBOARD_JS"; then
    echo "✅ Activity table handles field name variations"
    ((PASS++))
else
    echo "❌ Field name fallback missing"
    ((FAIL++))
fi

# Test 8: Cost field handling
if grep -q "const cost = item.cost || 0" "$DASHBOARD_JS"; then
    echo "✅ Activity handles missing cost field"
    ((PASS++))
else
    echo "❌ Cost field handling missing"
    ((FAIL++))
fi

# Test 9: No old hardcoded timestamps in updateProjectStats
if grep -A 15 "function updateProjectStats" "$DASHBOARD_JS" | grep -q "toLocaleTimeString"; then
    echo "❌ Old hardcoded timestamp still in updateProjectStats"
    ((FAIL++))
else
    echo "✅ Hardcoded timestamps removed from updateProjectStats"
    ((PASS++))
fi

# Test 10: No old hardcoded timestamps in updateMachineStats
if grep -A 15 "function updateMachineStats" "$DASHBOARD_JS" | grep -q "toLocaleTimeString"; then
    echo "❌ Old hardcoded timestamp still in updateMachineStats"
    ((FAIL++))
else
    echo "✅ Hardcoded timestamps removed from updateMachineStats"
    ((PASS++))
fi

# Test 11: JavaScript syntax is valid
if node -c "$DASHBOARD_JS" 2>/dev/null; then
    echo "✅ JavaScript syntax is valid"
    ((PASS++))
else
    echo "❌ JavaScript syntax error"
    ((FAIL++))
fi

# Test 12: HTML file exists and has correct elements
if grep -q "id=\"projectLastUpdate\"" "cco/static/dashboard.html"; then
    echo "✅ HTML has projectLastUpdate element"
    ((PASS++))
else
    echo "❌ HTML missing projectLastUpdate element"
    ((FAIL++))
fi

echo ""
echo "📊 Results: $PASS passed, $FAIL failed"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "✨ All verification tests passed!"
    exit 0
else
    echo "⚠️  Some tests failed"
    exit 1
fi
