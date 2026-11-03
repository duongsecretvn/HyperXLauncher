<template>
  <div class="absolute z-0 h-full w-full">
    <transition
      name="fade-transition"
    >
      <video
        v-if="customBackgroundVideo"
        ref="videoRef"
        :key="`custom-video-${customBackgroundVideo}`"
        class="absolute z-0 h-full w-full object-cover"
        :style="{ filter: `blur(${blur}px)`, 'object-fit': backgroundImageFit }"
        :src="customBackgroundVideo"
        autoplay
        loop
        playsinline
        :muted="isMuted"
      />
      <Particles
        v-else-if="backgroundType === BackgroundType.PARTICLE"
        color="#dedede"
        class="absolute z-0 h-full w-full"
        :style="{ filter: `blur(${blur}px)` }"
        :click-mode="particleMode"
        :move-enabled="isBackgroundActive"
      />
      <Halo
        v-else-if="backgroundType === BackgroundType.HALO && isBackgroundActive"
        class="absolute z-0 h-full w-full"
        :style="{ filter: `blur(${blur}px)` }"
      />
      <img
        v-else-if="backgroundImage?.type === 'image' && backgroundType === BackgroundType.IMAGE"
        :key="backgroundImage.url"
        :src="backgroundImage.url"
        class="absolute z-0 h-full w-full"
        :style="{ filter: `blur(${blur}px)`, 'object-fit': backgroundImageFit }"
      >
      <video
        v-else-if="backgroundImage?.type === 'video' && backgroundType === BackgroundType.VIDEO"
        ref="videoRef"
        :key="`video-${backgroundImage.url}`"
        class="absolute z-0 h-full w-full object-cover"
        :style="{ filter: `blur(${blur}px)`, 'object-fit': backgroundImageFit }"
        :src="backgroundImage.url"
        autoplay
        loop
        playsinline
        :muted="isMuted"
      />
    </transition>

    <transition
      name="fade-transition"
    >
      <div
        v-if="((backgroundColorOverlay && !isHome) || backgroundType === BackgroundType.NONE) && !customBackgroundVideo"
        class="z-3 absolute h-full w-full"
        :style="{ 'background': backgroundColor }"
      />
    </transition>
  </div>
</template>
<script lang="ts" setup>
import { useWindowFocus, useDocumentVisibility } from '@vueuse/core'
import Halo from '@/components/Halo.vue'
import Particles from '@/components/Particles.vue'
import { injection } from '@/util/inject'
import { kTheme, BackgroundType } from '@/composables/theme'
import { kInstanceLaunch } from '@/composables/instanceLaunch'
import { kCustomCss } from '@/composables/customCss'
import { inject } from 'vue'

const customCss = inject(kCustomCss)
const customBackgroundVideo = computed(() => customCss?.customBackgroundVideo.value || '')

const { sideBarColor, backgroundColorOverlay, backgroundColor, blur, backgroundImage, backgroundType, particleMode, backgroundImageFit, volume } = injection(kTheme)
const videoRef = ref(null as null | HTMLVideoElement)
const isMuted = computed(() => volume.value <= 0)

const route = useRoute()
const isHome = computed(() => route.path === '/')

const isWindowFocused = useWindowFocus()
const documentVisibility = useDocumentVisibility()
const { gameProcesses } = injection(kInstanceLaunch)

// Background animations (Particles, Halo, Video) should only be active when:
// 1. No Minecraft game process is running
// 2. Window is visible (not minimized/hidden)
// 3. Window is focused (when not focused, pause animations to save 100% GPU/CPU)
const isBackgroundActive = computed(() => {
  if (gameProcesses.value.length > 0) return false
  if (documentVisibility.value === 'hidden') return false
  return isWindowFocused.value
})

watch(volume, (newVolume) => {
  if (videoRef.value) {
    videoRef.value.volume = newVolume
  }
})

watch(isBackgroundActive, (active) => {
  if (videoRef.value) {
    if (active) {
      videoRef.value.play().catch(() => {})
    } else {
      videoRef.value.pause()
    }
  }
})

watch(videoRef, (v) => {
  if (v) {
    v.volume = volume.value
  }
})

onMounted(() => {
  if (videoRef.value) {
    videoRef.value.volume = volume.value
  }
})

watch(backgroundType, (t) => {
  console.log(t)
  console.log(backgroundImage.value)
})


</script>
<style scoped>
.img-container {
  background: radial-gradient(ellipse at top right, transparent, v-bind(sideBarColor) 72%);
  position: absolute;
  min-width: 100%;
  min-height: 100%;
  z-index: 4;
}
</style>
