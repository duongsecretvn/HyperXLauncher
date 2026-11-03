<template>
  <v-select
    v-model="selected"
    variant="outlined"
    density="comfortable"
    rounded="lg"
    :items="items"
    item-title="text"
    item-value="value"
    return-object
    :label="t('user.authMode')"
    flat
  >
    <template #selection="{ item }">
      <div class="flex items-center gap-2">
        <img
          v-if="getItemIcon(item) && (getItemIcon(item).startsWith('http') || getItemIcon(item).startsWith('data:'))"
          :src="getItemIcon(item)"
          class="flex-shrink-0"
          style="width: 20px; height: 20px; min-width: 20px; min-height: 20px; border-radius: 0 !important; object-fit: contain; image-rendering: pixelated;"
          alt=""
        />
        <v-icon v-else size="18" class="flex-shrink-0">
          {{ getItemIcon(item) || 'vpn_key' }}
        </v-icon>
        <span>{{ getItemText(item) }}</span>
      </div>
    </template>
    <template #item="{ item, props: itemProps }">
      <v-list-item
        v-bind="itemProps"
        :key="item.value"
        :data-testid="`login-authority-item-${authoritySlug(item.value)}`"
        :title="getItemText(item)"
      >
        <template #prepend>
          <img
            v-if="getItemIcon(item) && (getItemIcon(item).startsWith('http') || getItemIcon(item).startsWith('data:'))"
            :src="getItemIcon(item)"
            class="mr-2 flex-shrink-0"
            style="width: 24px; height: 24px; min-width: 24px; min-height: 24px; border-radius: 0 !important; object-fit: contain; image-rendering: pixelated;"
            alt=""
          />
          <v-icon v-else class="mr-2 flex-shrink-0">
            {{ getItemIcon(item) || 'vpn_key' }}
          </v-icon>
        </template>
      </v-list-item>
    </template>
    <template
      v-if="allowAddService"
      #append-item
    >
      <v-divider />
      <v-list-item
        :title="t('userService.add')"
        @click="$emit('add-service')"
      >
        <template #prepend>
          <v-icon class="mr-2 flex-shrink-0">add</v-icon>
        </template>
      </v-list-item>
    </template>
  </v-select>
</template>
<script setup lang="ts">
import { AuthorityItem, useAllowThirdparty } from '@/composables/login'
import { AUTHORITY_DEV, AUTHORITY_MICROSOFT, AUTHORITY_MOJANG } from '@xmcl/runtime-api'

const props = defineProps<{
  modelValue: string
  items: AuthorityItem[]
}>()
const emit = defineEmits<{
  (event: 'update:modelValue', value: string): void
  (event: 'add-service'): void
}>()

const { t } = useI18n()

/**
 * Stable, locale-independent slug for each authority option so e2e specs can
 * anchor on `data-testid="login-authority-item-<slug>"`. Built-in authorities
 * get a friendly name; third-party services fall back to their host.
 */
function authoritySlug(value: string) {
  if (value === AUTHORITY_MICROSOFT) return 'microsoft'
  if (value === AUTHORITY_MOJANG) return 'mojang'
  if (value === AUTHORITY_DEV) return 'offline'
  try {
    return `thirdparty-${new URL(value).host}`
  } catch {
    return `thirdparty-${value}`
  }
}

const allowThirdParty = useAllowThirdparty()
const allowAddService = computed(() => allowThirdParty.value)

const selected = computed<AuthorityItem | undefined>({
  get() { return props.items.find(a => a.value === props.modelValue) },
  set(v) {
    if (v) emit('update:modelValue', v.value)
  },
})

function getItemIcon(item: any): string {
  if (!item) return ''
  return item.raw?.icon ?? item.icon ?? ''
}

function getItemText(item: any): string {
  if (!item) return ''
  return item.raw?.text ?? item.title ?? item.text ?? ''
}
</script>
