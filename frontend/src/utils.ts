export const authFetch = async <T>(
    url: string,
    options: RequestInit = {}
): Promise<T> => {
    const token = localStorage.getItem('authToken');

    const headers = {
        ...options.headers,
        Authorization: token ? `Bearer ${token}` : '',
    };

    const response = await fetch(url, { ...options, headers });
    if (!response.ok) throw new Error('Network response was not ok: ' + JSON.stringify(response));
    return response.json();
};

export const authFetchWithStatus = async <T>(
    url: string,
    options: RequestInit = {}
): Promise<{ value: T | null; statusCode: number }> => {
    const token = localStorage.getItem('authToken');

    const headers = {
        ...options.headers,
        Authorization: token ? `Bearer ${token}` : '',
    };

    const response = await fetch(url, { ...options, headers });
    const statusCode = response.status;

    if (!response.ok) {
        try {
            const errorValue: T = await response.json();
            return { value: errorValue, statusCode };
        } catch {
            return { value: null, statusCode };
        }
    }

    try {
        const value: T = await response.json();
        return { value, statusCode };
    } catch {
        return { value: null, statusCode };
    }
};

export const authFetchDataOrError = async <T>(
    url: string,
    options: RequestInit = {},
    successSetter: (data: T) => void,
    errorSetter: (error: string) => void
): Promise<void> => {
    try {
        const data: T = await authFetch<T>(url, options);
        successSetter(data);
    } catch (error) {
        errorSetter((error as Error).message);
    }
};

export const logout = () => {
    const token = localStorage.getItem('authToken');
    if (token) {
        localStorage.removeItem('authToken')
    }
}

type RequestHandler = (url: string, data: unknown, headers?: object) => Promise<unknown>;

function createAuthHandler(method: "POST" | "PUT"): RequestHandler {
    return async (url, data, headers = {}) => {
        const token = localStorage.getItem('authToken');
        const options = {
            method,
            headers: {
                "Content-Type": "application/json",
                Authorization: token ? `Bearer ${token}` : '',
                ...headers,
            },
            body: JSON.stringify(data),
        };

        const response = await fetch(url, options);

        if (!response.ok) {
            const error = await response.json();
            throw new Error(error.message || "Request failed");
        }

        return await response.json();
    };
}

export const authPost = createAuthHandler("POST");
export const authPut = createAuthHandler("PUT");

export const authDelete = async (
    url: string
): Promise<object> => {
    const token = localStorage.getItem('authToken');

    const headers = {
        Authorization: token ? `Bearer ${token}` : '',
    };

    const response = await fetch(url, { method: 'DELETE', headers });
    if (!response.ok) throw new Error('Network response was not ok: ' + JSON.stringify(response));
    return response.json();
};

export const authDeleteWithStatus = async (
    url: string
): Promise<{ value: object | null; statusCode: number }> => {
    const token = localStorage.getItem('authToken');

    const headers = {
        Authorization: token ? `Bearer ${token}` : '',
    };

    const response = await fetch(url, { method: 'DELETE', headers });
    const statusCode = response.status;

    if (!response.ok) {
        try {
            const errorValue: object = await response.json();
            return { value: errorValue, statusCode };
        } catch {
            return { value: null, statusCode };
        }
    }

    try {
        const value: object = await response.json();
        return { value, statusCode };
    } catch {
        return { value: null, statusCode };
    }
};

export function pickOne(items: string[]): string {
    if (items.length === 0) {
        throw new Error("The list is empty. Cannot pick an item.");
    }
    const randomIndex = Math.floor(Math.random() * items.length);
    return items[randomIndex];
}

export function getRandomIntNormal(min: number, max: number): number {
    const mean = (max + min) / 2;
    const stdDev = (max - min) / 6; // Standard deviation (spread)

    function getNormalRandom(): number {
        const u1 = Math.random();
        const u2 = Math.random();
        const z0 = Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
        return z0;
    }

    let randomValue = mean + stdDev * getNormalRandom();

    // Ensure the value is within bounds
    randomValue = Math.max(min, Math.min(max, Math.round(randomValue)));
    return randomValue;
}
