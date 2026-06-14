export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
    readonly payload: unknown,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export interface ApiClient {
  get<TResponse>(path: string): Promise<TResponse>
  post<TRequest, TResponse>(path: string, payload: TRequest): Promise<TResponse>
}

const joinPath = (basePath: string, path: string) => {
  const normalizedBase = basePath.endsWith('/') ? basePath.slice(0, -1) : basePath
  const normalizedPath = path.startsWith('/') ? path : `/${path}`
  return `${normalizedBase}${normalizedPath}`
}

export const createApiClient = (basePath = '/api'): ApiClient => {
  const request = async <TResponse>(
    path: string,
    init?: RequestInit,
  ): Promise<TResponse> => {
    const response = await fetch(joinPath(basePath, path), {
      headers: {
        'Content-Type': 'application/json',
        ...(init?.headers ?? {}),
      },
      ...init,
    })

    if (!response.ok) {
      const payload = await response.text()
      throw new ApiError(`API request failed for ${path}`, response.status, payload)
    }

    return (await response.json()) as TResponse
  }

  return {
    get: <TResponse>(path: string) => request<TResponse>(path),
    post: <TRequest, TResponse>(path: string, payload: TRequest) =>
      request<TResponse>(path, {
        method: 'POST',
        body: JSON.stringify(payload),
      }),
  }
}

export const apiClient = createApiClient('/api')

