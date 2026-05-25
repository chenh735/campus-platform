import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { UserInfo } from '@/types'

export const useAuthStore = defineStore('auth', () => {
  const token = ref(localStorage.getItem('token') || '')
  const role = ref(localStorage.getItem('role') || '')
  const user = ref<UserInfo | null>(null)

  function setAuth(newToken: string, newRole: string, newUser: UserInfo) {
    token.value = newToken
    role.value = newRole
    user.value = newUser
    localStorage.setItem('token', newToken)
    localStorage.setItem('role', newRole)
  }

  function clearAuth() {
    token.value = ''
    role.value = ''
    user.value = null
    localStorage.removeItem('token')
    localStorage.removeItem('role')
  }

  function isLoggedIn() {
    return !!token.value
  }

  function isAdmin() {
    return role.value === 'admin'
  }

  return { token, role, user, setAuth, clearAuth, isLoggedIn, isAdmin }
})
