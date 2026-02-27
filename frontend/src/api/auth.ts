import api from './axios'
import type { UserDto, LoginRequest, RegisterRequest } from '@/types'

export const authApi = {
  async login(req: LoginRequest): Promise<UserDto> {
    const response = await api.post<UserDto>('/auth/login', req)
    return response.data
  },

  async register(req: RegisterRequest): Promise<UserDto> {
    const response = await api.post<UserDto>('/auth/register', req)
    return response.data
  },

  async logout(): Promise<void> {
    await api.post('/auth/logout')
  },

  async me(): Promise<UserDto> {
    const response = await api.get<UserDto>('/auth/me')
    return response.data
  },

  async refresh(): Promise<void> {
    await api.post('/auth/refresh')
  }
}
