import api from './index'
import type { Project, ProjectApplication } from '@/types'

export const getProjects = (params: Record<string, any>) =>
  api.get('/projects', { params })

export const getProject = (id: number) =>
  api.get<Project>(`/projects/${id}`)

export const createProject = (data: any) =>
  api.post('/projects', data)

export const updateProject = (id: number, data: any) =>
  api.put(`/projects/${id}`, data)

export const deleteProject = (id: number) =>
  api.delete(`/projects/${id}`)

export const applyProject = (id: number, data: any) =>
  api.post(`/projects/${id}/apply`, data)

export const getMyApplications = () =>
  api.get<ProjectApplication[]>('/profile/project-applications')
