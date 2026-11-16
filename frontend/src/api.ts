import type { AssignmentIndex, AssignmentInput, AssignmentResult, Items, LoginInput, SessionState, Wiki } from './types'

const errorMessages: Record<string, string> = {
  unauthenticated: 'Your session has expired. Sign in again.',
  invalid_request_origin: 'Your session token is invalid. Refresh and try again.',
  forbidden: 'You do not have permission for this action.',
  not_found: 'This resource is unavailable.',
  invalid_input: 'Check the required fields and associations.',
  idempotency_conflict: 'This submission ID was already used for different content.',
  conflict_changed: 'The conflict changed. Reload before deciding.',
  storage_busy: 'Not saved. Storage is busy; your text is retained for retry.',
}

export class ApiError extends Error {
  constructor(readonly code: string, readonly status: number, message?: string) {
    super(message ?? errorMessages[code] ?? 'Request failed. Please try again.')
  }
}

export function errorMessage(error: unknown): string {
  return error instanceof ApiError ? error.message : 'Request failed. Please try again.'
}

export class ApiClient {
  private csrfToken = ''

  private async request<T>(path: string, body?: unknown): Promise<T> {
    if (body !== undefined && !this.csrfToken) await this.session()
    let response: Response
    try {
      response = await fetch(`/api${path}`, {
        method: body === undefined ? 'GET' : 'POST',
        credentials: 'same-origin',
        cache: 'no-store',
        headers: body === undefined ? {} : { 'Content-Type': 'application/json', 'X-CSRF-Token': this.csrfToken },
        ...(body === undefined ? {} : { body: JSON.stringify(body) }),
      })
    } catch {
      throw new ApiError('network_error', 0, 'Connection failed. No save was confirmed; retry when connected.')
    }
    if (!response.ok) {
      const error = await response.json().catch(() => null) as { code?: unknown } | null
      const code = typeof error?.code === 'string' && Object.hasOwn(errorMessages, error.code) ? error.code : 'request_failed'
      throw new ApiError(code, response.status, path === '/login' && response.status === 401 ? 'Sign in failed.' : undefined)
    }
    try {
      return await response.json() as T
    } catch {
      throw new ApiError('invalid_response', response.status)
    }
  }

  private acceptSession(state: SessionState): SessionState {
    this.csrfToken = state.csrf_token
    return state
  }

  async session(): Promise<SessionState> {
    return this.acceptSession(await this.request<SessionState>('/session'))
  }

  async login(input: LoginInput): Promise<SessionState> {
    return this.acceptSession(await this.request<SessionState>('/login', input))
  }

  async logout(): Promise<SessionState> {
    return this.acceptSession(await this.request<SessionState>('/logout', {}))
  }

  wikis(): Promise<Items<Wiki>> {
    return this.request('/wikis')
  }

  assignments(): Promise<AssignmentIndex> {
    return this.request('/admin/assignments')
  }

  setAssignment(input: AssignmentInput): Promise<AssignmentResult> {
    return this.request('/admin/assignments', input)
  }
}
