import api from './index'
import type { DashboardStats, AuditItem, UserInfo, AdminLog } from '@/types'

export const getDashboard = () => api.get<DashboardStats>('/admin/dashboard')

export const createCourse = (data: any) => api.post('/admin/courses', data)

export const updateCourse = (id: number, data: any) => api.put(`/admin/courses/${id}`, data)

export const deleteCourse = (id: number) => api.delete(`/admin/courses/${id}`)

export const getAuditItems = (params: Record<string, any>) =>
  api.get('/admin/audit-items', { params })

export const auditMaterial = (id: number, status: string) =>
  api.put(`/admin/materials/${id}/status`, { status })

export const deleteMaterialByAdmin = (id: number) => api.delete(`/materials/${id}`)

export const updateReviewStatus = (id: number, status: string) =>
  api.put(`/admin/reviews/${id}/status`, { status })

export const getUsers = (params: Record<string, any>) =>
  api.get('/admin/users', { params })

export const updateUserStatus = (id: number, status: string) =>
  api.put(`/admin/users/${id}/status`, { status })

export const getAdminProjects = (params: Record<string, any>) =>
  api.get('/admin/projects', { params })

export const updateProjectStatus = (id: number, status: string) =>
  api.put(`/admin/projects/${id}/status`, { status })

export const getAdminLogs = (params?: { page: number; page_size: number }) =>
  api.get('/admin/logs', { params })
