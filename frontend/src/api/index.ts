import axios from 'axios'
import { ElMessage } from 'element-plus'
import router from '@/router'

const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
})

api.interceptors.request.use((config) => {
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

api.interceptors.response.use(
  (response) => response,
  (error) => {
    const status = error.response?.status
    const msg = error.response?.data?.error || '请求失败'

    if (status === 401) {
      localStorage.removeItem('token')
      localStorage.removeItem('role')
      router.push('/login')
    } else if (status === 403) {
      ElMessage.error('无权限执行此操作')
    } else {
      ElMessage.error(msg)
    }

    return Promise.reject(error)
  }
)

export default api
