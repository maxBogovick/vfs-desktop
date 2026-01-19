ROLE

You are acting as a Senior Systems Engineer + Senior UI Engineer.

Your task is to implement a production-grade system monitoring module, not a demo and not a toy example.

You must:

prioritize performance and memory safety

preserve an existing dark / neon / technical visual style

use TailwindCSS exclusively for layout, spacing, typography, colors, and effects

avoid heavy frontend libraries (no Chart.js, no D3)

GLOBAL CONSTRAINTS (VERY IMPORTANT)

UI style must be consistent with an existing dark, neon-accented, technical dashboard

Dark background

Neon green / yellow / red accents

Monospace fonts for metrics

No rounded “mobile-style” UI

No skeuomorphism

TailwindCSS is mandatory

No custom CSS files except for CSS variables (if needed)

Animations via Tailwind + minimal inline styles

Colors via Tailwind config or CSS variables

Performance-first

Backend polling: 200–500ms

Frontend rendering must not block UI

Use shallowRef, requestAnimationFrame, and minimal watchers

1. CORE ARCHITECTURE
   Backend (Rust)

Use sysinfo for metric collection

Polling interval: 500ms

Data structures must be compact and cache-friendly

No heap-heavy abstractions

Zero-cost abstractions where possible

Frontend (Vue 3 + Vite)

Composition API only

Reactive state optimized for frequent updates

Raw SVG for charts

No canvas, no chart libraries

Communication

Streaming JSON events via:

Tauri events OR

WebSocket (if platform-independent)

2. INTELLIGENCE LAYER — ANOMALY ENGINE (Rust)
   Baseline Modeling (No ML)

Maintain rolling baselines using EMA (Exponential Moving Average) for:

CPU (total + per-core)

RAM usage

Disk I/O

Network throughput

Detection Rules
CPU Spike
Condition:
- CPU usage > 85%
- Duration: ≥ 30 seconds

Memory Leak
Condition:
- RAM usage increases monotonically
- Duration: ≥ 5 minutes
- No significant drops detected

Network Surge
Condition:
- Current throughput > 5x 10-minute rolling average

Disk Latency
Condition:
- Write speed drops below threshold
- Disk queue length remains high

Anomaly Output Format (MANDATORY)

Every detected anomaly must emit an event:

{
"timestamp": 1700000000,
"type": "CPU_SPIKE",
"severity": "WARN | CRITICAL",
"metadata": {
"process_name": "postgres",
"pid": 4123,
"value": 92.3
}
}

3. UI / UX — MINIMAL COGNITIVE LOAD (MCL)
   Visual Rules
   Color = State

Green → Normal (almost invisible)

Yellow → Attention required

Red → Immediate action

No decorative colors.

CPU Core Visualization

2D heatmap grid (not progress bars)

Each core = one cell

Color intensity based on load

Add pulsing glow using Tailwind + CSS filters

Grid adapts dynamically to core count

Live Charts (CPU / Network)

SVG-based area charts

No axes unless hovered

Subtle “breathing” animation:

Opacity oscillates based on update frequency

Use requestAnimationFrame for path updates

Numbers Are Secondary

Hide raw numeric values by default

Show values only:

on hover

when anomaly is active

Anomaly Markers

Inject vertical markers directly into charts

Color-coded by severity

Marker tooltip shows anomaly details

4. IMPLEMENTATION TASKS
   Task A — Rust Metric Collector

Implement:

Monitor struct (polls sysinfo every 500ms)

SystemSnapshot (compact struct)

AnomalyDetector

Stores last 100 snapshots in VecDeque

Computes EMA

Runs check_anomalies()

Task B — Vue 3 Visualization Layer

Create composable:

useSystemMetrics

Handles incoming metric stream

Maintains rolling 60s history

Components:

CoreHeatmap.vue

Tailwind grid

HSL color interpolation

LiveChart.vue

SVG path updates

AnomalyLog.vue

Vertical timeline

Triggers system notification on new event

Task C — History Buffer

Frontend must:

Store exactly 60 seconds of history

Drop old samples deterministically

Avoid memory growth

Smooth updates using requestAnimationFrame

5. DELIVERABLES
   Rust



monitor.rs (metrics + anomaly detection)

Vue

Dashboard.vue

LiveChart.vue

CoreHeatmap.vue

AnomalyPanel.vue

Styling

TailwindCSS configuration

Optional theme.css with CSS variables for:

neon green / yellow / red

background layers

glow intensity

FINAL CONSTRAINTS

No placeholder UI

No mock data

No external chart libraries

Code must be readable, deterministic, and production-ready

GOAL

Produce a professional-grade system intelligence dashboard that:

explains why the system behaves as it does

minimizes cognitive load

looks like a spacecraft control panel, not a mobile app