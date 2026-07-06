import { readonly, ref } from 'vue'

export type ToastType = 'success' | 'error' | 'info'

export interface ToastMessage {
  id: number
  type: ToastType
  title: string
  description?: string
  duration: number
}

interface ToastOptions {
  title: string
  description?: string
  duration?: number
}

const DEFAULT_DURATION = 3200
const toasts = ref<ToastMessage[]>([])
let nextToastId = 1

function showToast(type: ToastType, options: ToastOptions) {
  const id = nextToastId++
  const toast: ToastMessage = {
    id,
    type,
    title: options.title,
    description: options.description,
    duration: options.duration ?? DEFAULT_DURATION
  }

  toasts.value = [...toasts.value, toast]

  if (toast.duration > 0) {
    window.setTimeout(() => dismissToast(id), toast.duration)
  }

  return id
}

function dismissToast(id: number) {
  toasts.value = toasts.value.filter((toast) => toast.id !== id)
}

function success(options: ToastOptions) {
  return showToast('success', options)
}

function error(options: ToastOptions) {
  return showToast('error', options)
}

function info(options: ToastOptions) {
  return showToast('info', options)
}

export function getErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

export function useToast() {
  return {
    toasts: readonly(toasts),
    success,
    error,
    info,
    dismiss: dismissToast
  }
}
