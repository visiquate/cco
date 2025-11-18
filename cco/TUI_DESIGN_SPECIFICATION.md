# TUI Dashboard Design Specification

## Visual Design Language

### Color Palette

```rust
// Dark Theme (Default)
const DARK_THEME: Theme = Theme {
    background: Color::Rgb(15, 23, 42),      // #0f172a - slate-900
    surface: Color::Rgb(30, 41, 59),         // #1e293b - slate-800
    border: Color::Rgb(51, 65, 85),          // #334155 - slate-700
    text_primary: Color::Rgb(226, 232, 240),  // #e2e8f0 - slate-200
    text_secondary: Color::Rgb(148, 163, 184), // #94a3b8 - slate-400

    // Status colors
    active: Color::Green,
    idle: Color::Rgb(148, 163, 184),
    error: Color::Red,
    warning: Color::Yellow,

    // Model tier colors
    opus_color: Color::Rgb(139, 92, 246),    // #8b5cf6 - violet-500
    sonnet_color: Color::Rgb(59, 130, 246),  // #3b82f6 - blue-500
    haiku_color: Color::Rgb(34, 197, 94),    // #22c55e - green-500

    // Accent colors
    highlight: Color::Rgb(99, 102, 241),     // #6366f1 - indigo-500
    selection: Color::Rgb(55, 65, 81),       // #374151 - gray-700
};
```

### Layout Zones

```
┌──────────────────────────────────────────────────────────┐
│                      HEADER (3 rows)                     │
├──────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────────────────────────────────────────┐     │
│  │            STATUS PANEL (5 rows)                │     │
│  └─────────────────────────────────────────────────┘     │
│                                                           │
│  ┌─────────────────────────────────────────────────┐     │
│  │         SUMMARY PANEL (3 rows)                  │     │
│  └─────────────────────────────────────────────────┘     │
│                                                           │
│  ┌─────────────────────────────────────────────────┐     │
│  │      MODEL BREAKDOWN TABLE (8 rows)             │     │
│  └─────────────────────────────────────────────────┘     │
│                                                           │
│  ┌─────────────────────────────────────────────────┐     │
│  │      RECENT CALLS TABLE (remaining space)       │     │
│  └─────────────────────────────────────────────────┘     │
│                                                           │
├──────────────────────────────────────────────────────────┤
│                    FOOTER (1 row)                        │
└──────────────────────────────────────────────────────────┘
```

## Component Specifications

### 1. Header Component

```rust
pub struct HeaderWidget {
    title: String,
    version: String,
    connection_status: ConnectionStatus,
    refresh_rate: Duration,
}

// Render example:
// ╔══════════════════════════════════════════════════════════╗
// ║ Claude API Cost Monitor v2025.11.1  [●Connected] ↻500ms  ║
// ╚══════════════════════════════════════════════════════════╝
```

### 2. Status Panel

```rust
pub struct StatusWidget {
    start_time: DateTime<Local>,
    elapsed: Duration,
    status: SessionStatus,
    current_rate: f64,  // calls per minute
}

// Render example:
// ┌─ Live Monitor ──────────────────────────────────────────┐
// │ Started: Nov 17, 2:30 PM PST                           │
// │ Elapsed: 2h 15m 32s                                    │
// │ Status: ● Active (6.2 calls/min)                       │
// └─────────────────────────────────────────────────────────┘
```

### 3. Summary Panel

```rust
pub struct SummaryWidget {
    total_cost: f64,
    total_calls: u64,
    avg_cost: f64,
    current_rate: f64,
}

// Render with color coding:
// ┌─ Cost Summary ──────────────────────────────────────────┐
// │ Total: $12.473  Calls: 847  Avg: $0.015  Rate: 6.2/min │
// └─────────────────────────────────────────────────────────┘
```

### 4. Model Breakdown Table

```rust
pub struct ModelBreakdownWidget {
    tiers: Vec<TierMetrics>,
    sort_by: SortColumn,
    show_cache_tokens: bool,
}

// Formatted table with bars:
// ┌─ By Model Tier ─────────────────────────────────────────┐
// │ Model   Cost    %    Calls  Input   Output  Cache W/R  │
// │ ──────────────────────────────────────────────────────  │
// │ Sonnet  $8.32  67%  ████████████████  234   1.2M  450K │
// │ Opus    $3.15  25%  ██████            45    200K  50K  │
// │ Haiku   $1.00   8%  ██                568   3.5M  800K │
// └─────────────────────────────────────────────────────────┘
```

### 5. Recent Calls Table

```rust
pub struct RecentCallsWidget {
    calls: VecDeque<ApiCall>,
    selected_index: usize,
    scroll_offset: usize,
}

// Scrollable with selection:
// ┌─ Recent API Calls ────────────────────[↑↓ Navigate]────┐
// │  Time      Model    Cost      Tokens  Source           │
// │ ──────────────────────────────────────────────────────  │
// │► 4:45:32  Sonnet   $0.045    1.5K/450 architect.js     │
// │  4:45:28  Haiku    $0.002    800/120  python-pro.js    │
// │  4:45:15  Haiku    $0.001    500/80   docs-writer.js   │
// │  4:44:58  Opus     $0.125    2.1K/600 chief.js         │
// │  4:44:45  Sonnet   $0.038    1.2K/380 security.js      │
// └────────────────────────────────────────── 5 of 25 ─────┘
```

## Interactive Elements

### Keyboard Navigation

```rust
pub enum KeyAction {
    // Navigation
    Up,         // ↑ - Navigate up in lists
    Down,       // ↓ - Navigate down in lists
    PageUp,     // PgUp - Jump 10 items up
    PageDown,   // PgDn - Jump 10 items down
    Home,       // Home - Go to first item
    End,        // End - Go to last item

    // Actions
    Refresh,    // r - Force refresh
    Reset,      // R - Reset session metrics
    Export,     // e - Export data
    Quit,       // q - Quit application

    // Views
    Details,    // d - Show call details
    History,    // h - Show historical view
    Settings,   // s - Open settings

    // Display toggles
    ToggleCache,  // c - Toggle cache token display
    ToggleTheme,  // t - Toggle dark/light theme
    TogglePause,  // space - Pause/resume updates
}
```

### Mouse Support (Optional)

```rust
pub enum MouseAction {
    Click(u16, u16),      // Click on coordinates
    Scroll(i16),          // Scroll wheel
    Drag(u16, u16),       // Drag for selection
}

// Click targets:
// - Table headers for sorting
// - Rows for selection
// - Buttons in footer
```

## Animation and Transitions

### Live Updates

```rust
pub struct AnimationState {
    // Smooth number transitions
    cost_animator: SmoothValue<f64>,
    count_animator: SmoothValue<u64>,

    // Status indicators
    activity_pulse: PulseAnimation,

    // New item highlights
    highlight_duration: Duration,
    new_items: HashSet<Uuid>,
}

// Example animations:
// - Cost increments smoothly: $12.47 -> $12.48 (over 200ms)
// - New calls flash briefly (green background for 500ms)
// - Active status pulses (opacity 100% -> 70% -> 100%)
// - Graph bars grow smoothly when values change
```

### Progress Indicators

```
// Loading state
┌─────────────────────────────┐
│  ⠋ Connecting to CCO proxy  │
└─────────────────────────────┘

// Active state (animated spinner)
[⠋] [⠙] [⠹] [⠸] [⠼] [⠴] [⠦] [⠧] [⠇] [⠏]

// Rate indicator (live sparkline)
Rate: ▁▂▃▅▇█▅▃▂▁ 6.2/min
```

## Responsive Layout

### Terminal Size Handling

```rust
pub struct ResponsiveLayout {
    min_width: u16,   // 80 columns minimum
    min_height: u16,  // 24 rows minimum
    breakpoints: Vec<Breakpoint>,
}

pub enum LayoutMode {
    Compact,   // < 100 cols: Hide some columns
    Standard,  // 100-150 cols: Normal view
    Wide,      // > 150 cols: Extra details
}

// Compact mode adjustments:
// - Abbreviate model names (Sonnet -> Son)
// - Hide cache tokens
// - Truncate source files
// - Show relative times (2m ago)

// Wide mode additions:
// - Show full token breakdowns
// - Display latency graphs
// - Add percentile columns
// - Show agent names
```

## Status Indicators

### Visual States

```
// Connection Status
● Connected (green)
◐ Connecting (yellow, animated)
○ Disconnected (gray)
✗ Error (red)

// Activity Status
▶ Active (green, animated)
⏸ Paused (yellow)
■ Stopped (gray)

// Cost Alerts
↗ Rising (yellow) - Rate increasing
→ Stable (green) - Normal rate
↘ Falling (blue) - Rate decreasing
⚠ Alert (red) - Threshold exceeded
```

## Data Visualization

### Cost Distribution Bar

```
Model Cost Distribution
├─ Sonnet  ████████████████████░░░░░ 67% ($8.32)
├─ Opus    ██████░░░░░░░░░░░░░░░░░░ 25% ($3.15)
└─ Haiku   ██░░░░░░░░░░░░░░░░░░░░░░  8% ($1.00)
```

### Rate Sparkline

```
Calls/min: ▁▂▃▃▅▇█▅▃▂▃▅▆▅▃▂ (avg: 6.2)
Cost/min:  ▁▁▂▄█▃▂▁▂▃▅▇▅▃▂▁ (avg: $0.09)
```

### Token Usage Meters

```
Input:  [████████████░░░░░░] 1.2M/2M
Output: [██████░░░░░░░░░░░░] 450K/2M
Cache:  [██░░░░░░░░░░░░░░░░] 150K/2M
```

## Error States

### Connection Error

```
┌─ Connection Error ───────────────────────────────────────┐
│                                                          │
│  ⚠ Unable to connect to CCO proxy on port 8888         │
│                                                          │
│  Possible causes:                                       │
│  • CCO proxy is not running                            │
│  • Port 8888 is blocked                                │
│  • Log file path is incorrect                          │
│                                                          │
│  [R]etry  [S]ettings  [Q]uit                           │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### Data Error

```
┌─ Data Warning ───────────────────────────────────────────┐
│                                                          │
│  ⚠ Incomplete data detected                             │
│                                                          │
│  Missing token counts in 3 recent API calls.            │
│  Cost calculations may be inaccurate.                   │
│                                                          │
│  [C]ontinue  [V]iew Details  [Q]uit                    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

## Export View

### Quick Export Dialog

```
┌─ Export Data ────────────────────────────────────────────┐
│                                                          │
│  Export Format:                                         │
│  ● CSV  ○ JSON  ○ SQLite                              │
│                                                          │
│  Date Range:                                            │
│  ● Current Session                                      │
│  ○ Today                                               │
│  ○ Last 7 days                                         │
│  ○ Custom: [2024-11-10] to [2024-11-17]               │
│                                                          │
│  Include:                                              │
│  ☑ Summary statistics                                  │
│  ☑ Per-model breakdown                                 │
│  ☑ Individual API calls                                │
│  ☐ Cache token details                                 │
│                                                          │
│  Export to: ~/Desktop/cco-export-2024-11-17.csv        │
│                                                          │
│  [E]xport  [C]ancel                                    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

## Settings View

### Configuration Editor

```
┌─ Settings ───────────────────────────────────────────────┐
│ Display                                                  │
│ ├─ Theme:        [●Dark ○Light ○Auto]                   │
│ ├─ Refresh:      [500ms ▼]                             │
│ ├─ Timezone:     [PST ▼]                               │
│ └─ Decimals:     [3 ▼]                                 │
│                                                          │
│ Data                                                     │
│ ├─ Retention:    [90 days ▼]                           │
│ ├─ Auto-cleanup: [●Yes ○No]                            │
│ └─ Export path:  [~/Desktop/             ]             │
│                                                          │
│ Alerts                                                   │
│ ├─ Daily limit:  [$100.00    ]                         │
│ ├─ Session limit:[$50.00     ]                         │
│ └─ Sound alert:  [○Yes ●No]                            │
│                                                          │
│ [S]ave  [R]eset  [C]ancel                              │
└──────────────────────────────────────────────────────────┘
```

## Performance Considerations

### Rendering Optimization

1. **Differential Updates**: Only redraw changed sections
2. **Virtual Scrolling**: Render only visible rows in tables
3. **Debounced Updates**: Batch updates every 16ms (60 FPS)
4. **Lazy Calculation**: Calculate aggregates on-demand

### Memory Management

1. **Ring Buffer**: Keep only last 25 calls in memory
2. **Compressed Storage**: Use efficient data structures
3. **Garbage Collection**: Clean old highlight states
4. **String Interning**: Reuse common strings

## Accessibility Features

### Screen Reader Support

```rust
// Provide text descriptions for visual elements
impl Accessible for DashboardWidget {
    fn describe(&self) -> String {
        format!(
            "API Cost Monitor. Session active for {}. \
             Total cost: ${:.2}. {} calls made. \
             Current rate: {:.1} calls per minute.",
            self.elapsed, self.total_cost,
            self.total_calls, self.rate
        )
    }
}
```

### High Contrast Mode

```rust
const HIGH_CONTRAST_THEME: Theme = Theme {
    background: Color::Black,
    text_primary: Color::White,
    border: Color::White,
    // Strong color differentiation
    opus_color: Color::Magenta,
    sonnet_color: Color::Cyan,
    haiku_color: Color::Yellow,
};
```

## Implementation Notes

### Component Library: ratatui

```rust
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Gauge, List,
        ListItem, Paragraph, Row, Sparkline, Table, Tabs, Wrap
    },
    Frame, Terminal,
};
```

### Event Loop Structure

```rust
pub async fn run_tui(mut terminal: Terminal<impl Backend>) -> Result<()> {
    let tick_rate = Duration::from_millis(500);
    let mut last_tick = Instant::now();

    loop {
        // Draw UI
        terminal.draw(|f| render_dashboard(f, &app_state))?;

        // Handle events
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => handle_key(key, &mut app_state),
                Event::Mouse(mouse) => handle_mouse(mouse, &mut app_state),
                Event::Resize(width, height) => handle_resize(width, height),
            }
        }

        // Update data
        if last_tick.elapsed() >= tick_rate {
            update_metrics(&mut app_state).await;
            last_tick = Instant::now();
        }

        // Check for quit
        if app_state.should_quit {
            break;
        }
    }

    Ok(())
}
```

---

*Specification Version: 1.0*
*Date: November 17, 2024*
*Status: READY FOR IMPLEMENTATION*