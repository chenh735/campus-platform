export interface UserInfo {
  id: number
  email: string
  nickname: string
  role: string
  experience: number
  status: string
}

export interface Course {
  id: number
  code: string
  name: string
  teacher: string
  credit: number | null
  category: string
  description: string
  rating_avg: number | null
  review_count: number
  material_count: number
}

export interface CourseDetail {
  id: number
  code: string
  name: string
  teacher: string
  credit: number | null
  category: string
  semester: string
  description: string
  status: string
  rating_avg: number | null
}

export interface Review {
  id: number
  course_id: number
  user_id: number
  nickname: string
  course_code: string | null
  course_name: string | null
  rating: number
  difficulty: string | null
  workload: string | null
  content: string
  like_count: number
  status: string
  created_at: string
}

export interface Material {
  id: number
  course_id: number
  user_id: number
  nickname: string
  course_code: string | null
  course_name: string | null
  title: string
  description: string | null
  tag: string | null
  original_name: string
  file_size: number
  file_type: string | null
  resource_type: 'file' | 'link'
  link_url: string | null
  download_count: number
  like_count: number
  status: string
  created_at: string
}

export interface Project {
  id: number
  user_id: number
  nickname: string
  title: string
  type: string
  tech_stack: string | null
  description: string
  requirements: string | null
  contact: string | null
  required_members: number
  current_members: number
  deadline: string | null
  status: string
  created_at: string
}

export interface ProjectApplication {
  id: number
  project_id: number
  project_title: string
  user_id: number
  applicant_nickname: string
  introduction: string
  contact: string
  status: string
  created_at: string
}

export interface ProfileData {
  user: UserInfo
  level: number
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  page_size: number
}

export interface DashboardStats {
  user_count: number
  course_count: number
  material_count: number
  project_count: number
  pending_audit_count: number
  today_active_count: number
  daily_active_counts: DailyActiveCount[]
}

export interface DailyActiveCount {
  date: string
  count: number
}

export interface AuditItem {
  id: number
  title: string
  item_type: string
  author: string
  related: string
  resource_type: 'file' | 'link'
  link_url: string | null
  created_at: string
  status: string
}

export interface AdminLog {
  id: number
  admin_id: number
  action: string
  target_type: string
  target_id: number | null
  detail: string | null
  created_at: string
}
