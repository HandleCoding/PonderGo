import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import HawkeyeWindow from './HawkeyeWindow.svelte'

const params = new URLSearchParams(window.location.search)
const component = params.get('window') === 'hawkeye' ? HawkeyeWindow : App

const app = mount(component, {
  target: document.getElementById('app')!,
})

export default app
