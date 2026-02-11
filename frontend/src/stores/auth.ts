import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api'

interface User {
  id: string
  username: string
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(null)
  const user = ref<User | null>(null)
  const isInitialized = ref(false)

  const isAuthenticated = computed(() => !!token.value)

  const login = async (username: string, password: string) => {
    const response = await api.post('/auth/login', { username, password })
    setToken(response.data.token)
    // user.value = response.data.user // If login returns user info
    await fetchUser()
  }

  const setToken = (newToken: string) => {
    token.value = newToken
  }

  const fetchUser = async () => {
    if (!token.value) return
    try {
      const response = await api.get('/auth/me')
      user.value = response.data
    } catch {
      // TODO: Error message toast
    }
  }

  const init = async () => {
    if (isInitialized.value) return
    // On app load, try to refresh immediately to see if we have a session
    try {
      const response = await api.post('/auth/refresh')
      setToken(response.data.token)
      await fetchUser()
    } catch (e) {
      // Ignored
    } finally {
      isInitialized.value = true
    }
  }

  const logout = async () => {
    try {
        await api.post('/auth/logout')
    } catch (e) {
        // TODO: Error message toast
    }
    
    token.value = null
    user.value = null
  }

  return {
    token,
    user,
    isAuthenticated,
    isInitialized,
    login,
    setToken,
    fetchUser,
    logout,
    init,
  }
})
