import api from './index'
import type { PaginatedResponse, Review } from '@/types'

export interface CourseLikeStatus {
  review_ids: number[]
  material_ids: number[]
}

export const getReviews = (courseId: number, params?: { page?: number; page_size?: number }) =>
  api.get<PaginatedResponse<Review>>(`/courses/${courseId}/reviews`, { params })

export const getCourseLikeStatus = (courseId: number) =>
  api.get<CourseLikeStatus>(`/courses/${courseId}/like-status`)

export const createReview = (courseId: number, data: any) =>
  api.post(`/courses/${courseId}/reviews`, data)

export const updateReview = (id: number, data: any) =>
  api.put(`/reviews/${id}`, data)

export const deleteReview = (id: number) =>
  api.delete(`/reviews/${id}`)

export const likeReview = (id: number) =>
  api.post(`/reviews/${id}/like`)

export const unlikeReview = (id: number) =>
  api.delete(`/reviews/${id}/like`)
