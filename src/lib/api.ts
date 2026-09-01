// ─────────────────────────────────────────────────────────────
// FasterEdge 开源项目
// Github: https://github.com/FasterEdge
// Gitee:  https://gitee.com/FasterEdge
// ─────────────────────────────────────────────────────────────
import { invoke } from '@tauri-apps/api/core'

export type Platform = 'windows' | 'linux' | 'android' | 'openharmony'

export interface Instance {
  id: string
  name: string
  url: string
  platform: Platform
  password: string
  online?: boolean
  status?: string
}

export interface Heartbeat {
  version: string
  state: string
  info: string
  timestamp: string
  logs: string[]
  process_pid: number
  process_path: string
  restart_count: number
  file_type: string
  last_exit_time?: string
  last_exit_code?: number
  last_exit_by_signal?: boolean
  last_exit_error?: string
  program_args?: string
  extra_env_raw?: string
}

export const PLATFORM_LABELS: Record<Platform, string> = {
  windows: 'Windows',
  linux: 'Linux (manylinux)',
  android: 'Android',
  openharmony: 'OpenHarmony',
}

export const PLATFORM_COLORS: Record<Platform, string> = {
  windows: '#4aa3ff',
  linux: '#e6b450',
  android: '#6ee7a0',
  openharmony: '#b78aff',
}

const DEFAULT_PORT = 11883

export function normalizeUrl(raw: string): string {
  let url = raw.trim().replace(/\/+$/, '')
  if (!/^https?:\/\//i.test(url)) url = `http://${url}`
  // 无端口号时补全默认端口 11883
  const parsed = new URL(url)
  if (!parsed.port && !parsed.username) parsed.port = String(DEFAULT_PORT)
  return parsed.toString().replace(/\/$/, '')
}

export function endpointUrl(base: string, path: string): string {
  return `${normalizeUrl(base)}${path}`
}

export class DontCrackApi {
  constructor(private readonly instance: Instance) {}

  private async request(path: string, timeoutMs = 8000): Promise<string> {
    return invoke<string>('dc_request', {
      url: endpointUrl(this.instance.url, path),
      method: 'GET',
      password: this.instance.password,
      timeoutMs,
    })
  }

  /** 启动进程，同时重置重启次数 */
  async startup(): Promise<string> {
    return this.request('/startup')
  }

  /** 心跳：进程状态与累积日志 */
  async heartbeat(): Promise<Heartbeat> {
    const text = await this.request('/heartbeat')
    return JSON.parse(text) as Heartbeat
  }

  /** 终止进程 */
  async shutdown(): Promise<string> {
    return this.request('/shutdown')
  }

  /** 探活（轻量）：成功即在线 */
  async ping(): Promise<boolean> {
    try {
      await this.heartbeat()
      return true
    } catch {
      return false
    }
  }
}