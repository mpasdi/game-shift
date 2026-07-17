import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { router } from './router'
import './styles/main.css'

if (import.meta.env.PROD) {
  document.addEventListener('contextmenu', (event) => {
    const target = event.target

    if (
      target instanceof Element &&
      target.closest('input, textarea, [contenteditable]:not([contenteditable=false])')
    ) {
      return
    }

    event.preventDefault()
  })
}

createApp(App).use(createPinia()).use(router).mount('#app')
