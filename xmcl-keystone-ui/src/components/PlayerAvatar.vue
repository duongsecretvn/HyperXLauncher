<template>
  <div
    class="player-avatar-wrapper overflow-hidden"
    :style="{
      'max-width': `${dimension}px`,
      'max-height': `${dimension}px`,
      'min-height': `${dimension}px`,
      'min-width': `${dimension}px`,
      width: `${dimension}px`,
      height: `${dimension}px`,
      'border-radius': radiusValue,
    }"
  >
    <img
      :src="dataUrlSrc"
      :width="dimension"
      :height="dimension"
      class="player-avatar-img block object-cover"
      style="image-rendering: pixelated;"
      :style="{
        width: `${dimension}px`,
        height: `${dimension}px`,
        'border-radius': radiusValue,
      }"
    >
  </div>
</template>

<script lang="ts" setup>
import steve from '@/assets/steve_skin.png'
import { renderMinecraftPlayerTextHead } from '@/util/avatarRenderer'
import { ref, computed, watch } from 'vue'

const props = withDefaults(
  defineProps<{
    src: string
    dimension?: number
    radius?: number | string
  }>(), {
  src: steve,
  dimension: 64,
  radius: 12,
})

const radiusValue = computed(() => {
  if (props.radius === undefined || props.radius === null) return '12px'
  if (typeof props.radius === 'number') return `${props.radius}px`
  return props.radius
})

const steveSrc = renderMinecraftPlayerTextHead(steve)
const dataUrlSrc = ref('')

// Generation counter discards stale async results so a slow steve resolve
// (queued while src='') cannot overwrite a real avatar resolved later.
let generation = 0
watch(() => props.src, (s) => {
  const gen = ++generation
  if (!s) {
    steveSrc?.then((v) => {
      if (gen === generation) dataUrlSrc.value = v
    })
    return
  }
  renderMinecraftPlayerTextHead(s)?.then((v) => {
    if (gen === generation) dataUrlSrc.value = v
  }, () => {
    steveSrc?.then((v) => {
      if (gen === generation && !dataUrlSrc.value) {
        dataUrlSrc.value = v
      }
    })
  })
}, { immediate: true })

</script>

<style scoped>
.player-avatar-wrapper {
  display: flex;
  align-items: center;
  justify-content: center;
  user-select: none;
  -webkit-user-drag: none;
}
.player-avatar-img {
  user-select: none;
  -webkit-user-drag: none;
}
</style>
