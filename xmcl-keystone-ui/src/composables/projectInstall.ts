import { getCursforgeFileModLoaders, getCursforgeModLoadersFromString, getModLoaderTypesForFile } from '@/util/curseforge'
import { injection } from '@/util/inject'
import { ProjectEntry, ProjectFile } from '@/util/search'
import { getSWRV } from '@/util/swrvGet'
import { Ref } from 'vue'
import { getCurseforgeProjectFilesModel, getCurseforgeProjectModel } from './curseforge'
import { getCurseforgeDependenciesModel } from './curseforgeDependencies'
import { kCurseforgeInstaller } from './curseforgeInstaller'
import { getModrinthDependenciesModel } from './modrinthDependencies'
import { kModrinthInstaller } from './modrinthInstaller'
import { getModrinthProjectModel } from './modrinthProject'
import { getModrinthVersionModel } from './modrinthVersions'
import { kSWRVConfig } from './swrvConfig'
import { useNotifier } from './notifier'
import { RuntimeVersions } from '@xmcl/instance'

/**
 * Provide default install for the project
 */
export function useProjectInstall(runtime: Ref<RuntimeVersions>,
  loader: Ref<string | undefined>,
  curseforgeInstaller = injection(kCurseforgeInstaller),
  modrinthInstaller = injection(kModrinthInstaller),
  installLocal: (item: ProjectFile) => void,
) {
  const config = injection(kSWRVConfig)
  const { notify } = useNotifier()
  const onInstallProject = async (item: ProjectEntry) => {
    try {
      const activeLoader = loader.value || (
        runtime.value.forge ? 'forge'
        : runtime.value.fabricLoader ? 'fabric'
        : runtime.value.neoForged ? 'neoforge'
        : runtime.value.quiltLoader ? 'quilt'
        : undefined
      )

      const modrinthProjectId = item.modrinth?.project_id
      const curseforgeId = item.curseforge?.id

      if (modrinthProjectId) {
        const proj = await getSWRV(getModrinthProjectModel(ref(modrinthProjectId)), config)
        const gameVersions = [runtime.value.minecraft]
        const versions = await getSWRV(getModrinthVersionModel(ref(modrinthProjectId), undefined, ref(activeLoader), ref(gameVersions)), config)
        const version = (activeLoader && versions ? versions.find(v => v.loaders.includes(activeLoader)) : undefined) || versions?.[0]
        if (version) {
          const deps = await getSWRV(getModrinthDependenciesModel(ref(version), ref(activeLoader), config), config)
          const artifactUrl = (version.files?.find(file => file.primary) ?? version.files?.[0])?.url
          await modrinthInstaller.installWithDependencies(version.id, version.loaders, proj?.icon_url, item.installed, deps || [], artifactUrl)
          return
        }
      }

      if (curseforgeId) {
        const proj = await getSWRV(getCurseforgeProjectModel(ref(curseforgeId)), config)
        const _loaders = getCursforgeModLoadersFromString(activeLoader)
        let files = await getSWRV(getCurseforgeProjectFilesModel(ref(curseforgeId), ref(runtime.value.minecraft), ref(_loaders[0])), config)
        if (!files || !files.data || files.data.length === 0) {
          files = await getSWRV(getCurseforgeProjectFilesModel(ref(curseforgeId), ref(runtime.value.minecraft), ref(undefined)), config)
        }
        const file = files?.data?.[0]
        if (file) {
          const loaderType = getModLoaderTypesForFile(file)
          const deps = await getSWRV(getCurseforgeDependenciesModel(ref(file), ref(runtime.value.minecraft), ref([...loaderType][0]), config), config)
          await curseforgeInstaller.installWithDependencies(file.id, getCursforgeFileModLoaders(file), proj?.logo?.url, item.installed, deps || [], file.downloadUrl)
          return
        }
      }

      if (item.files && item.files.length > 0) {
        const file = item.files[0]
        if (file) {
          installLocal(file)
          return
        }
      }

      notify({
        level: 'warning',
        title: `No compatible version found for Minecraft ${runtime.value.minecraft}${activeLoader ? ` (${activeLoader})` : ''}`,
      })
    } catch (e: any) {
      notify({
        level: 'error',
        title: e?.message || 'Failed to install project',
      })
    }
  }

  return onInstallProject
}
