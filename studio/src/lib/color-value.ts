const HEX3 = /^#([0-9a-fA-F]{3})$/
const HEX6 = /^#([0-9a-fA-F]{6})$/
const HEX8 = /^#([0-9a-fA-F]{8})$/
const RGB = /^rgba?\(\s*([\d.]+)\s*,\s*([\d.]+)\s*,\s*([\d.]+)(?:\s*,\s*[\d.]+\s*)?\)$/i
const HSL = /^hsla?\(/i

function expandHex3(hex: string): string {
  return `#${hex[1]}${hex[1]}${hex[2]}${hex[2]}${hex[3]}${hex[3]}`
}

function channelToHex(value: number): string {
  const clamped = Math.max(0, Math.min(255, Math.round(value)))
  return clamped.toString(16).padStart(2, '0')
}

/** True when the whole value is a single CSS color (not a gradient or other expression). */
export function isSolidColorValue(value: string): boolean {
  const trimmed = value.trim()
  if (!trimmed) return false
  if (trimmed.includes('gradient')) return false
  return (
    HEX3.test(trimmed) ||
    HEX6.test(trimmed) ||
    HEX8.test(trimmed) ||
    RGB.test(trimmed) ||
    HSL.test(trimmed)
  )
}

/** Hex string suitable for `<input type="color">` (`#rrggbb`). */
export function colorInputValue(value: string): string {
  return parseToHex6(value) ?? '#000000'
}

export function parseToHex6(value: string): string | null {
  const trimmed = value.trim()
  if (!trimmed) return null

  if (HEX3.test(trimmed)) {
    return expandHex3(trimmed).toLowerCase()
  }

  if (HEX6.test(trimmed)) {
    return trimmed.toLowerCase()
  }

  if (HEX8.test(trimmed)) {
    return `#${trimmed.slice(1, 7).toLowerCase()}`
  }

  const rgbMatch = trimmed.match(RGB)
  if (rgbMatch) {
    const r = Number(rgbMatch[1])
    const g = Number(rgbMatch[2])
    const b = Number(rgbMatch[3])
    if ([r, g, b].some((n) => Number.isNaN(n))) return null
    return `#${channelToHex(r)}${channelToHex(g)}${channelToHex(b)}`
  }

  return null
}

/** Theme keys that hold solid colors; gradient tokens stay text-only. */
export function isThemeColorField(key: string): boolean {
  return !key.endsWith('_gradient')
}

export function showThemeColorPicker(key: string, value: string): boolean {
  if (!isThemeColorField(key)) return false
  const trimmed = value.trim()
  if (!trimmed) return true
  return isSolidColorValue(trimmed)
}
