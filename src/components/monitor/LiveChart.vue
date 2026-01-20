<script setup lang="ts">
import { computed, ref, getCurrentInstance } from 'vue'

interface Props {
  data: number[]
  capacity?: number
  color?: 'green' | 'yellow' | 'red' | 'blue'
  label?: string
  unit?: string
  value?: string
  subValue?: string
  height?: number
  showGrid?: boolean
  min?: number
  max?: number
  formatTick?: (val: number) => string
}

const props = withDefaults(defineProps<Props>(), {
  data: () => [],
  color: 'green',
  height: 100,
  showGrid: true,
  unit: ''
})

const SVG = {
  width: 300,
  height: 100,
  padding: 6
}

const containerRef = ref<HTMLElement | null>(null)
const hoveredIndex = ref<number | null>(null)
const chartId = `chart-${getCurrentInstance()?.uid}`

const colorHex = computed(() => {
  switch (props.color) {
    case 'yellow': return '#facc15'
    case 'red': return '#ef4444'
    case 'blue': return '#60a5fa'
    default: return '#4ade80'
  }
})

const colorClasses = computed(() => {
  switch (props.color) {
    case 'yellow': return { stroke: 'stroke-yellow-400', text: 'text-yellow-400' }
    case 'red': return { stroke: 'stroke-red-500', text: 'text-red-500' }
    case 'blue': return { stroke: 'stroke-blue-400', text: 'text-blue-400' }
    default: return { stroke: 'stroke-green-500', text: 'text-green-500' }
  }
})

const baseMin = computed(() => {
  if (props.min !== undefined) return props.min
  if (!props.data.length) return 0
  return Math.min(...props.data)
})

const baseMax = computed(() => {
  if (props.max !== undefined) return props.max
  if (!props.data.length) return 100
  return Math.max(...props.data)
})

const derivedMin = computed(() => {
  const range = baseMax.value - baseMin.value || 1
  return baseMin.value - range * 0.1
})

const derivedMax = computed(() => {
  const range = baseMax.value - baseMin.value || 1
  return baseMax.value + range * 0.1
})

const ticks = computed(() => {
  const min = derivedMin.value
  const max = derivedMax.value
  const range = max - min || 1

  const values = [max, min + range * 0.5, min]

  return values.map((v, i) => {
    const normalized = (v - min) / range
    const y = SVG.height - SVG.padding - normalized * (SVG.height - SVG.padding * 2)

    let textY = y + 3
    if (i === 0) textY = y + 8
    if (i === 2) textY = y - 2

    return {
      y,
      textY,
      label: props.formatTick ? props.formatTick(v) : `${v.toFixed(1)}${props.unit}`
    }
  })
})

const chartCalc = computed(() => {
  const count = props.data.length
  const capacity = props.capacity ?? count
  const startIndex = Math.max(0, capacity - count)
  const stepX = SVG.width / Math.max(1, capacity - 1)
  const range = derivedMax.value - derivedMin.value || 1

  return { count, startIndex, stepX, range }
})

const paths = computed(() => {
  if (!props.data.length) return { line: '', area: '' }

  const { startIndex, stepX, range } = chartCalc.value
  const usableHeight = SVG.height - SVG.padding * 2

  const points = props.data.map((val, i) => {
    const x = (startIndex + i) * stepX
    const normalized = (val - derivedMin.value) / range
    const y = SVG.height - SVG.padding - normalized * usableHeight
    return `${x.toFixed(1)},${y.toFixed(1)}`
  })

  const line = `M ${points.join(' L ')}`
  const lastX = points.at(-1)?.split(',')[0] ?? SVG.width
  const firstX = points[0]?.split(',')[0] ?? 0
  const area = `${line} L ${lastX},${SVG.height} L ${firstX},${SVG.height} Z`

  return { line, area }
})

const cursor = computed(() => {
  if (hoveredIndex.value === null) return null
  const i = hoveredIndex.value
  const val = props.data[i]
  const { startIndex, stepX, range } = chartCalc.value
  const usableHeight = SVG.height - SVG.padding * 2

  const x = (startIndex + i) * stepX
  const normalized = (val - derivedMin.value) / range
  const y = SVG.height - SVG.padding - normalized * usableHeight

  return { x, y, val }
})

const currentValue = computed(() => {
  if (hoveredIndex.value !== null) {
    const v = props.data[hoveredIndex.value]
    return props.formatTick ? props.formatTick(v) : `${v.toFixed(1)}${props.unit}`
  }
  if (props.value) return props.value
  if (!props.data.length) return '--'
  const v = props.data.at(-1)!
  return props.formatTick ? props.formatTick(v) : `${v.toFixed(1)}${props.unit}`
})

const handleMouseMove = (e: MouseEvent) => {
  if (!containerRef.value || !props.data.length) return
  const rect = containerRef.value.getBoundingClientRect()
  const scaleX = SVG.width / rect.width
  const svgX = (e.clientX - rect.left) * scaleX

  const { startIndex, stepX, count } = chartCalc.value
  const rawIndex = (svgX / stepX) - startIndex
  const i = Math.floor(rawIndex + 0.5)

  hoveredIndex.value = Math.max(0, Math.min(i, count - 1))
}

const handleMouseLeave = () => {
  hoveredIndex.value = null
}
</script>

<template>
  <div class="flex flex-col w-full h-full bg-[#111] border border-[#333] rounded-sm overflow-hidden select-none"
       @mouseleave="handleMouseLeave">

    <!-- Header / Values -->
    <div class="flex-shrink-0 px-3 pt-2 pb-1 relative z-10 flex justify-between items-start pointer-events-none">
      <div>
        <div v-if="label" class="text-[9px] uppercase tracking-wider text-gray-500 font-mono leading-tight mb-0.5">
          {{ label }}
        </div>
        <div :class="['text-base font-mono font-bold leading-none', colorClasses.text]">
          {{ currentValue }}
        </div>
        <div v-if="subValue && hoveredIndex === null" class="text-[9px] text-gray-500 font-mono mt-0.5 leading-tight">
          {{ subValue }}
        </div>
        <div v-else-if="hoveredIndex !== null" class="text-[9px] text-gray-400 font-mono mt-0.5 leading-tight">
          At cursor
        </div>
      </div>
    </div>

    <div class="flex-1 relative overflow-hidden" ref="containerRef" @mousemove="handleMouseMove">
      <svg class="absolute inset-0 w-full h-full"
           :viewBox="`0 0 ${SVG.width} ${SVG.height}`"
           preserveAspectRatio="none">

        <defs>
          <linearGradient :id="chartId" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" :stop-color="colorHex" stop-opacity="0.25" />
            <stop offset="100%" :stop-color="colorHex" stop-opacity="0" />
          </linearGradient>
        </defs>

        <g v-if="showGrid">
          <template v-for="(t, i) in ticks" :key="i">
            <line x1="0" :y1="t.y" :x2="SVG.width" :y2="t.y"
                  stroke="#333" stroke-width="1"
                  vector-effect="non-scaling-stroke" />
            <text :x="SVG.width - 2" :y="t.textY"
                  text-anchor="end"
                  class="text-[7px] font-mono fill-gray-500">
              {{ t.label }}
            </text>
          </template>
        </g>

        <path :d="paths.area" :fill="`url(#${chartId})`" />
        <path :d="paths.line" fill="none"
              :class="colorClasses.stroke"
              stroke-width="1.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              vector-effect="non-scaling-stroke" />

        <g v-if="cursor">
          <line :x1="cursor.x" y1="0" :x2="cursor.x" :y2="SVG.height"
                :stroke="colorHex"
                stroke-width="1"
                stroke-dasharray="3,3"
                opacity="0.5"
                vector-effect="non-scaling-stroke" />
          <circle :cx="cursor.x" :cy="cursor.y" r="3"
                  :fill="colorHex"
                  stroke="#111"
                  stroke-width="1" />
        </g>
      </svg>
    </div>
  </div>
</template>

<style scoped>
svg {
  display: block;
}
</style>
