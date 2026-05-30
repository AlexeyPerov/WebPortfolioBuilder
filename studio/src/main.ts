import { mount } from 'svelte'
import { applyBuiltinTheme, DEFAULT_BUILTIN_THEME } from './lib/styles/themeTokens'
import './lib/styles/tokens.css'
import './app.css'
import App from './App.svelte'

applyBuiltinTheme(DEFAULT_BUILTIN_THEME, document.documentElement)

const app = mount(App, {
  target: document.getElementById('app')!,
})

export default app
