import YAML from 'yaml';
// import Papa from 'papaparse';

// Types
interface AuthFetchOptions extends RequestInit {
    headers?: Record<string, string>;
}

type ErrorHandler = (message: string) => void;

// Common fetch functionality
const performAuthFetch = async (url: string, options: AuthFetchOptions = {}): Promise<Response> => {
    var token = null;
    if (typeof window !== 'undefined' && typeof localStorage !== 'undefined') {
        token = localStorage.getItem('authToken');
    } else {
        console.error("LocalStorage is not avalable")
    }

    const headers = {
        ...options.headers,
        Authorization: token ? `Bearer ${token}` : '',
    };

    const response = await fetch(url, { ...options, headers });

    if (!response.ok) {
        throw new Error(`HTTP error ${response.status}`);
    }

    return response;
};

export const authFetchJson = async <T = any>(
    url: string,
    options: AuthFetchOptions = {},
    onError: ErrorHandler = console.error
): Promise<T | null> => {
    try {
        const response = await performAuthFetch(url, options);
        const data: T = await response.json();
        return data;
    } catch (error) {
        const message = error instanceof Error ? error.message : 'Unknown error';
        onError(`Fetch error: ${message}`);
        return null;
    }
};

export const authFetch = async (url: string, options: AuthFetchOptions = {}): Promise<Response> => {
    const token = localStorage.getItem('authToken');
    const headers = {
        ...options.headers,
        Authorization: token ? `Bearer ${token}` : '',
    };

    const response = await fetch(url, { ...options, headers });
    return response;
};

export const logout = (): void => {
    localStorage.removeItem('authToken');
    localStorage.removeItem('company');
};

// Additional utility functions with proper typing
export const authFetchText = async (
    url: string,
    options: AuthFetchOptions = {}
): Promise<string | null> => {
    try {
        const response = await performAuthFetch(url, options);
        return await response.text();
    } catch (error) {
        console.error(`Fetch error: ${error instanceof Error ? error.message : 'Unknown error'}`);
        return null;
    }
};

export const authFetchBlob = async (
    url: string,
    options: AuthFetchOptions = {}
): Promise<Blob | null> => {
    try {
        const response = await performAuthFetch(url, options);
        return await response.blob();
    } catch (error) {
        console.error(`Fetch error: ${error instanceof Error ? error.message : 'Unknown error'}`);
        return null;
    }
};

// Helper for POST requests with JSON body
export const authPostJson = async <T = any>(
    url: string,
    data: any,
    options: AuthFetchOptions = {}
): Promise<T | null> => {
    return authFetchJson<T>(url, {
        ...options,
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            ...options.headers,
        },
        body: JSON.stringify(data),
    });
};

// Helper for PUT requests with JSON body
export const authPutJson = async <T = any>(
    url: string,
    data: any,
    options: AuthFetchOptions = {}
): Promise<T | null> => {
    return authFetchJson<T>(url, {
        ...options,
        method: 'PUT',
        headers: {
            'Content-Type': 'application/json',
            ...options.headers,
        },
        body: JSON.stringify(data),
    });
};

// Helper for DELETE requests
export const authDelete = async (
    url: string,
    options: AuthFetchOptions = {}
): Promise<Response> => {
    return authFetch(url, {
        ...options,
        method: 'DELETE',
    });
};

// Type-safe error class
export class AuthFetchError extends Error {
    constructor(
        message: string,
        public status: number,
        public response?: Response
    ) {
        super(message);
        this.name = 'AuthFetchError';
    }
}

// Enhanced version with better error handling
export const authFetchWithError = async (
    url: string,
    options: AuthFetchOptions = {}
): Promise<Response> => {
    const token = localStorage.getItem('authToken');
    const headers = {
        ...options.headers,
        Authorization: token ? `Bearer ${token}` : '',
    };

    const response = await fetch(url, { ...options, headers });

    if (!response.ok) {
        throw new AuthFetchError(
            `HTTP error ${response.status}: ${response.statusText}`,
            response.status,
            response
        );
    }

    return response;
};
