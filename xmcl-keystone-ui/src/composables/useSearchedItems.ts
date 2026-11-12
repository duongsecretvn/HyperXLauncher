import { ExploreProjectModern } from '@/components/StoreExploreCardModern.vue'
import { CurseforgeBuiltinClassId, useCurseforge } from '@/composables/curseforge'
import { useDateString } from '@/composables/date'
import { getFeedTheBeastProjectModel, useFeedTheBeast } from '@/composables/ftb'
import { useMarketSort } from '@/composables/marketSort'
import { useModrinth } from '@/composables/modrinth'
import { useService } from '@/composables/service'
import { getCursforgeModLoadersFromString } from '@/util/curseforge'
import { injection } from '@/util/inject'
import { getExpectedSize } from '@/util/size'
import { getStoreSources, mergeStoreResults, StoreSortMetrics } from '@/util/storeSort'
import { getSWRV } from '@/util/swrvGet'
import { ProjectMappingServiceKey } from '@xmcl/runtime-api'
import { Ref, computed, ref, watch } from 'vue'
import { kSWRVConfig } from './swrvConfig'

interface UseSearchedItemsOptions {
  query: Ref<string>
  gameVersion: Ref<string>
  modLoaders: Ref<string[]>
  sort: Ref<string>
  page: Ref<number>
  omitSources: Ref<string[]>
  modrinthCategories: Ref<string[]>
  curseforgeCategory: Ref<number | undefined>
  projectTypes?: Ref<string[]>
  projectType?: Ref<string>
  pageSize: number
  tCategory: (key: string) => string
}

export function useSearchedItems(options: UseSearchedItemsOptions) {
  const {
    query,
    gameVersion,
    modLoaders,
    sort,
    page,
    omitSources,
    modrinthCategories,
    curseforgeCategory,
    projectTypes,
    projectType,
    pageSize,
    tCategory,
  } = options

  const { getDateString } = useDateString()
  const { lookupBatch } = useService(ProjectMappingServiceKey)
  const { modrinthSort, curseforgeSort } = useMarketSort(sort)
  const sources = computed(() => getStoreSources(omitSources.value, modrinthCategories.value, curseforgeCategory.value))
  const config = injection(kSWRVConfig)

  const activeProjectTypes = computed(() => {
    if (projectTypes?.value && projectTypes.value.length > 0) {
      return projectTypes.value
    }
    if (projectType?.value) {
      return [projectType.value]
    }
    return []
  })

  // FTB
  const { refreshing: ftbLoading, data: ftbData } = useFeedTheBeast(reactive({ keyword: query }))

  // FTB
  type StoreProject = ExploreProjectModern & { sortMetrics: StoreSortMetrics }
  const ftbItems = shallowRef<StoreProject[]>([])
  watch([ftbData, page, activeProjectTypes], async ([packs, pageNum, types], _, onCleanup) => {
    let cancelled = false
    onCleanup(() => { cancelled = true })
    ftbItems.value = []
    if (types.length > 0 && !types.some(t => t === 'modpack' || t === 'modpacks')) {
      return
    }
    if (!packs) {
      ftbItems.value = []
      return
    }
    if (!('packs' in packs)) {
      ftbItems.value = []
      return
    }

    // each page show 5 items
    const offset = (pageNum - 1) * 5

    const result = await Promise.all(packs.packs.slice(offset, offset + 5).map(async (p: any) => {
      const data = await getSWRV(getFeedTheBeastProjectModel(ref(p)), config)
      const result: StoreProject = {
        sortMetrics: {
          downloads: data?.installs ?? 0,
          updated: (data?.updated ?? 0) * 1000,
          newest: (data?.released ?? 0) * 1000,
        },
        id: p.toString(),
        type: 'ftb',
        title: data?.name ?? '',
        iconUrl: data?.art.find((v: any) => v.type === 'square')?.url ?? '',
        description: data?.synopsis || '',
        author: data?.authors[0]?.name ?? '',
        downloadCount: getExpectedSize(data?.installs ?? 0, ''),
        updatedAt: getDateString((data?.updated ?? 0) * 1000),
        version: data?.plays.toString() ?? '0',
        gallery: data?.art.map((a: any) => a.url) ?? [],
      }
      return result
    }))

    if (!cancelled) ftbItems.value = result
  }, { immediate: true })

  const modrinthProjectType = computed(() => {
    const types = activeProjectTypes.value
    if (types.length === 0) return ''
    const normalize = (pt: string) => {
      if (pt === 'modpack' || pt === 'modpacks') return 'modpack'
      if (pt === 'mod' || pt === 'mods') return 'mod'
      if (pt === 'resourcepack' || pt === 'resourcepacks') return 'resourcepack'
      if (pt === 'datapack' || pt === 'datapacks') return 'datapack'
      if (pt === 'shader' || pt === 'shaders') return 'shader'
      return pt
    }
    if (types.length === 1) {
      return normalize(types[0])
    }
    return types.map(normalize)
  })

  const curseforgeClassId = computed(() => {
    const types = activeProjectTypes.value
    if (types.length !== 1) return undefined
    const pt = types[0]
    if (pt === 'modpack' || pt === 'modpacks') return CurseforgeBuiltinClassId.modpack
    if (pt === 'mod' || pt === 'mods') return CurseforgeBuiltinClassId.mod
    if (pt === 'resourcepack' || pt === 'resourcepacks') return CurseforgeBuiltinClassId.resourcePack
    if (pt === 'datapack' || pt === 'datapacks') return CurseforgeBuiltinClassId.datapack
    if (pt === 'shader' || pt === 'shaders') return CurseforgeBuiltinClassId.shaderPack
    return undefined
  })

  // Modrinth
  const {
    error: searchError,
    refreshing: isModrinthSearching,
    projects,
    pageCount: modrinthPageCount,
  } = useModrinth(
    query,
    gameVersion,
    '',
    modrinthCategories,
    modLoaders,
    '',
    modrinthSort,
    modrinthProjectType,
    page,
    pageSize,
  )

  // Curseforge
  const { projects: curseforgeProjects, pages: curseforgePageCount, isValidating: isCurseforgeSearching } = useCurseforge(
    curseforgeClassId,
    query,
    page,
    computed(() => getCursforgeModLoadersFromString(modLoaders.value)),
    curseforgeCategory,
    curseforgeSort,
    gameVersion,
    pageSize,
  )

  // Mappings for search results
  const mappings = ref<Record<string, { name: string; description: string }>>({})

  watch([projects, curseforgeProjects], async ([modrinthProjects, cfProjects]) => {
    const modrinthIds = modrinthProjects.map(p => p.project_id)
    const curseforgeIds = cfProjects.map(p => p.id)

    const result = await lookupBatch(modrinthIds, curseforgeIds)
    const newMappings: Record<string, { name: string; description: string }> = {}

    for (const mapping of result) {
      if (mapping.modrinthId) {
        newMappings[`modrinth:${mapping.modrinthId}`] = {
          name: mapping.name,
          description: mapping.description,
        }
      }
      if (mapping.curseforgeId) {
        newMappings[`curseforge:${mapping.curseforgeId}`] = {
          name: mapping.name,
          description: mapping.description,
        }
      }
    }

    mappings.value = newMappings
  }, { immediate: true })

  const items = computed(() => {
    const modrinths = projects.value.map((p) => {
      const mapping = mappings.value[`modrinth:${p.project_id}`]
      const mapped: StoreProject = {
        sortMetrics: {
          downloads: p.downloads,
          updated: Date.parse(p.date_modified),
          newest: Date.parse(p.date_created),
        },
        id: p.project_id,
        type: 'modrinth',
        title: p.title,
        iconUrl: p.icon_url,
        description: p.description,
        author: p.author,
        downloadCount: getExpectedSize(p.downloads, ''),
        updatedAt: getDateString(p.date_modified),
        version: p.versions[p.versions.length - 1],
        gallery: p.gallery,
        localizedTitle: mapping?.name,
        localizedDescription: mapping?.description,
      }
      return mapped
    })
    const curseforges = curseforgeProjects.value.map((p) => {
      const mapping = mappings.value[`curseforge:${p.id}`]
      const mapped: StoreProject = {
        sortMetrics: {
          downloads: p.downloadCount,
          updated: Date.parse(p.dateModified),
          newest: Date.parse(p.dateCreated),
        },
        id: p.id.toString(),
        type: 'curseforge',
        title: p.name,
        iconUrl: p.logo?.thumbnailUrl ?? '',
        description: p.summary,
        author: p.authors[0]?.name ?? '',
        downloadCount: getExpectedSize(p.downloadCount, ''),
        updatedAt: getDateString(p.dateModified),
        version: p.latestFilesIndexes[0]?.gameVersion,
        gallery: p.screenshots.map(s => s?.thumbnailUrl || ''),
        localizedTitle: mapping?.name,
        localizedDescription: mapping?.description,
      }
      return mapped
    })

    const results = { modrinth: modrinths, curseforge: curseforges, ftb: ftbItems.value }
    // A single provider's server-side ordering must survive unchanged across pages.
    if (sources.value.length === 1) return results[sources.value[0]]
    return mergeStoreResults(sources.value.map(source => results[source]), modrinthSort.value)
  })

  const pageCount = computed(() => Math.max(0, ...sources.value.map(source => {
    if (source === 'modrinth') return modrinthPageCount.value
    if (source === 'curseforge') return curseforgePageCount.value
    const data = ftbData.value
    return data && 'packs' in data ? Math.ceil(data.packs.length / 5) : 0
  })))
  const isSearching = computed(() => isModrinthSearching.value || isCurseforgeSearching.value || ftbLoading.value)

  return {
    items,
    isSearching,
    searchError,
    pageCount,
    sources,
  }
}
