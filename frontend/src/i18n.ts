import { enMessages } from './i18n.en'
import { zhMessages } from './i18n.zh'

export type Locale = 'en' | 'zh'

export const localeNames: Record<Locale, string> = {
  en: 'EN',
  zh: '中',
}

const messages = {
  en: enMessages,
  zh: zhMessages,
} satisfies Record<Locale, Record<string, string>>

export type MessageKey = keyof typeof enMessages
export type Translate = (key: MessageKey) => string

export function getTranslator(locale: Locale): Translate {
  return (key) => messages[locale][key] ?? messages.en[key]
}
