import api from './index'
import type { Course, CourseDetail } from '@/types'

export const getCourses = (params: Record<string, any>) =>
  api.get('/courses', { params })

export const getCourse = (id: number) =>
  api.get<CourseDetail>(`/courses/${id}`)
