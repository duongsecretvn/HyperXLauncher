<template>
  <div
    class="loading-indicator-wrapper items-center justify-center"
    :class="type === 'progressBar' ? 'flex w-full' : 'inline-flex'"
    :style="{
      '--loading-color': computedColor,
      '--track-color': computedTrackColor,
      ...(type === 'progressBar' ? { maxWidth: `${actualBarWidth}px` } : {}),
    }"
  >
    <!-- 1. SPINNER -->
    <div
      v-if="type === 'spinner'"
      role="progressbar"
      aria-label="Đang tải..."
      class="spinner-box"
      :style="{ width: `${actualSpinnerSize}px`, height: `${actualSpinnerSize}px` }"
    >
      <svg viewBox="0 0 50 50" class="spinner-svg">
        <circle
          class="spinner-track"
          cx="25"
          cy="25"
          r="20"
          fill="none"
          stroke-width="4"
          :style="{ stroke: computedTrackColor }"
        />
        <circle
          class="spinner-fill"
          cx="25"
          cy="25"
          r="20"
          fill="none"
          stroke-width="4"
          :style="{ stroke: computedColor }"
        />
      </svg>
    </div>

    <!-- 2. PROGRESS RING (KHÔNG SỐ) -->
    <div
      v-else-if="type === 'progressRing'"
      role="progressbar"
      aria-label="Tiến trình vòng tròn"
      aria-valuemin="0"
      aria-valuemax="100"
      class="ring-box"
      :style="{ width: `${actualRingSize}px`, height: `${actualRingSize}px` }"
    >
      <svg viewBox="0 0 100 100" class="ring-svg">
        <circle
          class="ring-track"
          cx="50"
          cy="50"
          r="42"
          fill="none"
          stroke-width="8"
          :style="{ stroke: computedTrackColor }"
        />
        <circle
          class="ring-fill"
          cx="50"
          cy="50"
          r="42"
          fill="none"
          stroke-width="8"
          :style="{ stroke: computedColor }"
        />
      </svg>
    </div>

    <!-- 3. LINEAR PROGRESS BAR (KHÔNG SỐ) -->
    <div
      v-else-if="type === 'progressBar'"
      role="progressbar"
      aria-label="Tiến trình thanh ngang"
      aria-valuemin="0"
      aria-valuemax="100"
      class="linear-track"
      :style="{
        width: '100%',
        maxWidth: `${actualBarWidth}px`,
        height: `${actualBarHeight}px`,
        backgroundColor: computedTrackColor,
      }"
    >
      <div
        class="linear-fill"
        :style="{
          backgroundColor: computedColor,
        }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { LoadingIconType } from '@/composables/loadingIcon'

const props = withDefaults(
  defineProps<{
    type?: LoadingIconType
    color?: string
    size?: number
    barWidth?: number
    barHeight?: number
    trackColor?: string
  }>(),
  {
    type: 'spinner',
    color: '#ffffff',
    size: undefined,
    barWidth: 320,
    barHeight: 8,
    trackColor: '',
  },
)

const actualSpinnerSize = computed(() => props.size ?? 48)
const actualRingSize = computed(() => props.size ?? 64)
const actualBarWidth = computed(() => props.barWidth ?? 320)
const actualBarHeight = computed(() => props.barHeight ?? 8)

const computedColor = computed(() => props.color || '#ffffff')
const computedTrackColor = computed(() => {
  if (props.trackColor) return props.trackColor
  return '#334155'
})
</script>

<style scoped>
/* 1. SPINNER */
.spinner-box {
  display: inline-flex;
}
.spinner-svg {
  width: 100%;
  height: 100%;
  animation: spinner-rotate 1.5s linear infinite;
}
.spinner-track {
  stroke: var(--track-color, #334155);
}
.spinner-fill {
  stroke: var(--loading-color, #ffffff);
  stroke-linecap: round;
  stroke-dasharray: 1, 150;
  stroke-dashoffset: 0;
  animation: spinner-dash 1.5s ease-in-out infinite;
}
@keyframes spinner-rotate {
  100% {
    transform: rotate(360deg);
  }
}
@keyframes spinner-dash {
  0% {
    stroke-dasharray: 1, 150;
    stroke-dashoffset: 0;
  }
  50% {
    stroke-dasharray: 90, 150;
    stroke-dashoffset: -35px;
  }
  100% {
    stroke-dasharray: 90, 150;
    stroke-dashoffset: -124px;
  }
}

/* 2. PROGRESS RING (KHÔNG SỐ) */
.ring-box {
  display: inline-flex;
}
.ring-svg {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg); /* Bắt đầu chạy từ đỉnh 12h */
}
.ring-track {
  stroke: var(--track-color, #334155);
}
.ring-fill {
  stroke: var(--loading-color, #ffffff);
  stroke-linecap: round;
  /* Chu vi 2 * PI * 42 = 263.89px */
  stroke-dasharray: 263.89;
  stroke-dashoffset: 263.89;
  animation: fill-ring 2.5s cubic-bezier(0.4, 0, 0.2, 1) infinite;
}
@keyframes fill-ring {
  0% {
    stroke-dashoffset: 263.89; /* 0% */
  }
  80%, 100% {
    stroke-dashoffset: 0;      /* 100% */
  }
}

/* 3. LINEAR PROGRESS BAR (KHÔNG SỐ) */
.linear-track {
  width: 100%;
  max-width: 320px;
  height: 8px;
  background-color: var(--track-color, #334155); /* Progress track */
  border-radius: 9999px;
  overflow: hidden;
}
.linear-fill {
  height: 100%;
  width: 0%;
  background-color: var(--loading-color, #ffffff); /* Thanh tiến trình */
  border-radius: 9999px;
  animation: fill-bar 2.5s cubic-bezier(0.4, 0, 0.2, 1) infinite;
}
@keyframes fill-bar {
  0% {
    width: 0%;
  }
  80%, 100% {
    width: 100%;
  }
}
</style>
