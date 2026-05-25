import api from './index'
import type { ProfileData, PaginatedResponse, Material, Review, Project, ProjectApplication } from '@/types'

export const getProfile = () => api.get<ProfileData>('/profile')

export const updateProfile = (data: { nickname?: string }) =>
  api.put('/profile', data)

export const changePassword = (data: { old_password: string; new_password: string }) =>
  api.put('/profile/password', data)

export const getMyMaterials = (params: { page: number; page_size: number }) =>
  api.get<PaginatedResponse<Material>>('/profile/materials', { params })

export const getMyReviews = (params: { page: number; page_size: number }) =>
  api.get<PaginatedResponse<Review>>('/profile/reviews', { params })

export const getMyProjects = (params: { page: number; page_size: number }) =>
  api.get<PaginatedResponse<Project>>('/profile/projects', { params })

export const getMyApplications = (params: { page: number; page_size: number }) =>
  api.get<PaginatedResponse<ProjectApplication>>('/profile/applications', { params })

export const getReceivedApplications = (params: { page: number; page_size: number }) =>
  api.get<PaginatedResponse<ProjectApplication>>('/profile/received-applications', { params })

export const handleProjectApplication = (id: number, status: 'accepted' | 'rejected') =>
  api.put(`/project-applications/${id}/status`, { status })

export const deleteMaterial = (id: number) =>
  api.delete(`/materials/${id}`)

export const deleteReview = (id: number) =>
  api.delete(`/reviews/${id}`)

export const updateOwnProjectStatus = (id: number, status: 'closed' | 'hidden') =>
  api.put(`/projects/${id}/status`, { status })

export const deleteApplication = (id: number) =>
  api.delete(`/profile/project-applications/${id}`)
