<script lang="ts" setup>
import VersionInput, { VersionItem } from './VersionInput.vue'
import { BuiltinImages } from '../constant'
import { computed, markRaw } from 'vue'

const props = withDefaults(defineProps<{
  value?: string
  placeholder?: string
  disabled?: boolean
}>(), {
  value: '',
  placeholder: '',
  disabled: false,
})

const emit = defineEmits<{
  (event: 'input', value: string): void
}>()

const items = computed<VersionItem[]>(() => [
  markRaw({
    name: '1.21.11 (Rust Core)',
    tag: 'Horizon',
    tagColor: 'primary',
    description: 'Horizon Native Rust Rewrite (Parity 100%)',
  }),
  markRaw({
    name: '1.21.11 (Rust + Sodium)',
    tag: 'High FPS',
    tagColor: 'green',
    description: 'Horizon Rust Engine + Sodium & Fabric API Integrated',
  }),
  markRaw({
    name: '1.21.11-rust-all-loaders',
    tag: 'Universal',
    tagColor: 'cyan',
    description: 'Rust Core compatible with Fabric, Forge, NeoForged & Quilt',
  }),
  markRaw({
    name: 'horizon-0.2.97 (1.21.11)',
    tag: 'Release',
    tagColor: 'orange',
    description: 'Official Horizon Rust Minecraft Core v0.2.97',
  }),
])
</script>

<template>
  <VersionInput
    :icon="BuiltinImages.rust"
    title="Rust"
    url="https://github.com/horizon-rust/horizon"
    :is-clearable="true"
    :items="items"
    clear-text="Auto / None"
    empty-text="No Rust Core"
    :placeholder="placeholder || '1.21.11 (Rust Core)'"
    :value="value"
    :disabled="disabled"
    @input="emit('input', $event)"
  />
</template>
