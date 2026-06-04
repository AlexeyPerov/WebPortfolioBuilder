export type BuiltinThemeId = 'dark' | 'light'

export const BUILTIN_THEME_IDS: BuiltinThemeId[] = ['dark', 'light']

export const DEFAULT_BUILTIN_THEME: BuiltinThemeId = 'dark'

const LEGACY_THEME_IDS: Record<string, BuiltinThemeId> = {
  'dark-amber': 'dark',
  'light-blue': 'light',
}

const BUILTIN_LABELS: Record<BuiltinThemeId, string> = {
  dark: 'Dark',
  light: 'Light',
}

const BUILTIN_ACCENT_HEX: Record<BuiltinThemeId, string> = {
  dark: '#2376ff',
  light: '#2376ff',
}

export interface ThemeSyntaxPalette {
  keyword: string
  string: string
  comment: string
  number: string
  type: string
  heading: string
  link: string
  markup: string
  punctuation: string
}

const SYNTAX_PALETTE_CSS_VARS = [
  'keyword',
  'string',
  'comment',
  'number',
  'type',
  'heading',
  'link',
  'markup',
  'punctuation',
] as const satisfies ReadonlyArray<keyof ThemeSyntaxPalette>

const SYNTAX_PALETTE_FIXED: Record<
  'dark' | 'light',
  Omit<ThemeSyntaxPalette, 'keyword' | 'type' | 'link'>
> = {
  dark: {
    string: '#98c379',
    comment: '#5c6370',
    number: '#d19a66',
    heading: '#e06c75',
    markup: '#56b6c2',
    punctuation: '#abb2bf',
  },
  light: {
    string: '#50a14f',
    comment: '#a0a1a7',
    number: '#986801',
    heading: '#e45649',
    markup: '#0184bc',
    punctuation: '#383a42',
  },
}

function parseHex(hex: string): { r: number; g: number; b: number } {
  const normalized = hex.replace('#', '')
  return {
    r: Number.parseInt(normalized.slice(0, 2), 16),
    g: Number.parseInt(normalized.slice(2, 4), 16),
    b: Number.parseInt(normalized.slice(4, 6), 16),
  }
}

function toHex(r: number, g: number, b: number): string {
  const clamp = (value: number) =>
    Math.round(Math.min(255, Math.max(0, value)))
      .toString(16)
      .padStart(2, '0')
  return `#${clamp(r)}${clamp(g)}${clamp(b)}`
}

function mixHex(colorA: string, colorB: string, ratio: number): string {
  const a = parseHex(colorA)
  const b = parseHex(colorB)
  return toHex(
    a.r + (b.r - a.r) * ratio,
    a.g + (b.g - a.g) * ratio,
    a.b + (b.b - a.b) * ratio,
  )
}

export function migrateBuiltinThemeId(value: string): BuiltinThemeId | null {
  if (isBuiltinThemeId(value)) return value
  return LEGACY_THEME_IDS[value] ?? null
}

export function getBuiltinThemeLabel(id: BuiltinThemeId): string {
  return BUILTIN_LABELS[id]
}

export function getBuiltinThemeMode(id: BuiltinThemeId): 'dark' | 'light' {
  return id
}

export function getBuiltinAccentHex(id: BuiltinThemeId): string {
  return BUILTIN_ACCENT_HEX[id]
}

export function isBuiltinThemeId(value: string): value is BuiltinThemeId {
  return (BUILTIN_THEME_IDS as readonly string[]).includes(value)
}

export function getThemeSyntaxPalette(id: BuiltinThemeId): ThemeSyntaxPalette {
  const mode = getBuiltinThemeMode(id)
  const accent = getBuiltinAccentHex(id)
  const fixed = SYNTAX_PALETTE_FIXED[mode]

  if (mode === 'dark') {
    return {
      ...fixed,
      keyword: accent,
      type: mixHex(accent, '#ffffff', 0.28),
      link: mixHex(accent, '#ffffff', 0.14),
    }
  }

  return {
    ...fixed,
    keyword: accent,
    type: mixHex(accent, '#ffffff', 0.38),
    link: accent,
  }
}

export function applyBuiltinTheme(id: BuiltinThemeId, root: HTMLElement): void {
  const mode = getBuiltinThemeMode(id)
  root.dataset.theme = mode

  const accent = getBuiltinAccentHex(id)
  root.style.setProperty('--accent-color', accent)
  root.style.setProperty('--color-accent', accent)

  const palette = getThemeSyntaxPalette(id)
  for (const key of SYNTAX_PALETTE_CSS_VARS) {
    root.style.setProperty(`--syntax-${key}`, palette[key])
  }
}
