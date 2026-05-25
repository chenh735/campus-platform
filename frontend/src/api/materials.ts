import api from './index'
import type { Material, PaginatedResponse } from '@/types'

export const getMaterials = (courseId: number, params?: { page?: number; page_size?: number }) =>
  api.get<PaginatedResponse<Material>>(`/courses/${courseId}/materials`, { params })

export const uploadMaterial = (courseId: number, formData: FormData) =>
  api.post(`/courses/${courseId}/materials`, formData, {
    headers: { 'Content-Type': 'multipart/form-data' },
  })

export const downloadMaterial = (id: number) =>
  api.get(`/materials/${id}/download`, { responseType: 'blob' })

export const likeMaterial = (id: number) =>
  api.post(`/materials/${id}/like`)

export const unlikeMaterial = (id: number) =>
  api.delete(`/materials/${id}/like`)
