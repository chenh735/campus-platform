import api from './index'
import type { UserInfo } from '@/types'

export const sendCode = (email: string) => api.post('/auth/send-code', { email })

export const register = (data: { email: string; code: string; nickname: string; password: string }) =>
  api.post('/auth/register', data)

export const login = (data: { email: string; password: string }) =>
  api.post('/auth/login', data)

export const getMe = () => api.get<UserInfo>('/auth/me')
