<template>
  <span
    class="app-agent-icon select-none inline-flex items-center justify-center shrink-0"
    :style="{ width: normalizedSize, height: normalizedSize }"
  >
    <!-- Thinking / Motion state: 3 dots in rounded square with aiThinkingPulse animation -->
    <svg
      v-if="thinking"
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 200 200"
      width="100%"
      height="100%"
      class="agent-svg agent-svg--thinking"
    >
      <defs>
        <style>
          @keyframes aiThinkingPulse {
            0%, 100% {
              fill: #6C757D;
              opacity: 0.35;
              transform: scale(0.9);
            }
            50% {
              fill: #000000;
              opacity: 1;
              transform: scale(1.15);
            }
          }
          .dot-1 {
            animation: aiThinkingPulse 1.4s ease-in-out infinite;
            transform-origin: 66px 100px;
          }
          .dot-2 {
            animation: aiThinkingPulse 1.4s ease-in-out infinite 0.25s;
            transform-origin: 100px 100px;
          }
          .dot-3 {
            animation: aiThinkingPulse 1.4s ease-in-out infinite 0.5s;
            transform-origin: 134px 100px;
          }
        </style>
      </defs>

      <!-- Khung hình vuông bo góc tròn (Ruột trắng, nền ngoài trong suốt) -->
      <rect
        x="25"
        y="25"
        width="150"
        height="150"
        rx="38"
        fill="#FFFFFF"
        stroke="#212529"
        stroke-width="5.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      />

      <!-- 3 Chấm suy nghĩ căn giữa hình vuông, đậm nét và chuyển động nhịp nhàng -->
      <circle cx="66" cy="100" r="8.5" class="dot-1" fill="#6C757D" />
      <circle cx="100" cy="100" r="8.5" class="dot-2" fill="#6C757D" />
      <circle cx="134" cy="100" r="8.5" class="dot-3" fill="#6C757D" />
    </svg>

    <!-- Normal Idle state: Robot -->
    <svg
      v-else
      xmlns="http://www.w3.org/2000/svg"
      viewBox="55 55 90 80"
      width="100%"
      height="100%"
      class="agent-svg agent-svg--normal"
    >
      <!-- Ăng-ten robot -->
      <line x1="100" y1="67" x2="100" y2="78" stroke="#212529" stroke-width="4.5" stroke-linecap="round" class="agent-antenna" />
      <circle cx="100" cy="64" r="4.5" fill="#212529" class="agent-antenna-circle" />

      <!-- Thân / Đầu Robot -->
      <rect x="66" y="78" width="68" height="50" rx="10" fill="#212529" class="agent-head" />

      <!-- Tai robot 2 bên -->
      <rect x="59" y="92" width="7" height="20" rx="2.5" fill="#6C757D" />
      <rect x="134" y="92" width="7" height="20" rx="2.5" fill="#6C757D" />

      <!-- Mắt Robot -->
      <circle cx="85" cy="103" r="6" fill="#FFFFFF" />
      <circle cx="115" cy="103" r="6" fill="#FFFFFF" />
    </svg>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    size?: number | string
    thinking?: boolean
  }>(),
  {
    size: 24,
    thinking: false,
  },
)

const normalizedSize = computed(() => {
  if (typeof props.size === 'number') {
    return `${props.size}px`
  }
  return props.size
})
</script>

<style scoped>
.app-agent-icon {
  display: inline-flex;
  vertical-align: middle;
  line-height: 1;
}

.agent-svg {
  display: block;
  width: 100%;
  height: 100%;
}

:global(.dark) .agent-head,
:global(.theme--dark) .agent-head {
  stroke: rgba(255, 255, 255, 0.4);
  stroke-width: 1.8px;
}
:global(.dark) .agent-antenna,
:global(.theme--dark) .agent-antenna {
  stroke: #ffffff;
}
:global(.dark) .agent-antenna-circle,
:global(.theme--dark) .agent-antenna-circle {
  fill: #ffffff;
}
</style>
