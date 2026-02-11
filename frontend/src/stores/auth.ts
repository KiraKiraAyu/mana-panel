import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { api } from '@/api'

interface User {
  id: string
  username: string
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('token'))
  const user = ref<User | null>(null)

  const isAuthenticated = computed(() => !!token.value)

  const login = async (username: string, password: string) => {
    const response = await api.post('/auth/login', { username, password })
    token.value = response.data.token
    localStorage.setItem('token', response.data.token)
    await fetchUser()
  }

  const fetchUser = async () => {
    if (!token.value) return
    try {
      const response = await api.get('/auth/me')
      user.value = response.data
    } catch {
      logout()
    }
  }

  const logout = () => {
    token.value = null
    user.value = null
    localStorage.removeItem('token')
  }

  return {
    token,
    user,
    isAuthenticated,
    login,
    fetchUser,
    logout,
  }
})
