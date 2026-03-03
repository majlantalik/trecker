import api from './axios'
import type { ProfileDto, UpdateProfileRequest, ChangePasswordRequest } from '@/types'

export const profileApi = {
  getProfile: () => api.get<ProfileDto>('/profile').then(r => r.data),
  updateProfile: (req: UpdateProfileRequest) => api.patch<ProfileDto>('/profile', req).then(r => r.data),
  changePassword: (req: ChangePasswordRequest) => api.post('/profile/password', req),
}
