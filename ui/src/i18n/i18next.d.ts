import 'i18next'
import type about from '@/locales/en/about.json'
import type accounts from '@/locales/en/accounts.json'
import type common from '@/locales/en/common.json'
import type instance from '@/locales/en/instance.json'
import type instanceDialogs from '@/locales/en/instanceDialogs.json'
import type launch from '@/locales/en/launch.json'
import type main from '@/locales/en/main.json'
import type newInstance from '@/locales/en/newInstance.json'
import type settings from '@/locales/en/settings.json'
import type setup from '@/locales/en/setup.json'

declare module 'i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common'
    resources: {
      about: typeof about
      accounts: typeof accounts
      common: typeof common
      instance: typeof instance
      instanceDialogs: typeof instanceDialogs
      launch: typeof launch
      main: typeof main
      newInstance: typeof newInstance
      settings: typeof settings
      setup: typeof setup
    }
  }
}
