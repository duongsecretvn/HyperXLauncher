import { type PresenceService as IPresenceService, PresenceServiceKey } from '@xmcl/runtime-api'
import { Inject, LauncherAppKey } from '~/app'
import { AbstractService, ExposeServiceKey } from '~/service'
import { LauncherApp } from '../app/LauncherApp'

@ExposeServiceKey(PresenceServiceKey)
export class PresenceService extends AbstractService implements IPresenceService {
  constructor(@Inject(LauncherAppKey) app: LauncherApp) {
    super(app)
  }

  async setActivity(_activity: string): Promise<void> {
    // Discord presence permanently disabled and purged
  }
}
