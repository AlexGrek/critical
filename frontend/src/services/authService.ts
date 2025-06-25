
const API_BASE_URL = '/api/v1/auth'; // Replace with your backend API URL

interface LoginResponse {
    message: string;
    token: string; // Your app's JWT or session token
    user: {
        id: string;
        email: string;
        name: string;
        // ... other user details
    };
}

export const googleLogin = async (idToken: string): Promise<LoginResponse> => {
    try {
        const response = await fetch(`${API_BASE_URL}/google`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({ idToken }),
        });

        if (!response.ok) {
            // If the response status is not 2xx, throw an error
            const errorData = await response.json(); // Attempt to parse error message from backend
            throw new Error(errorData.message || `HTTP error! status: ${response.status}`);
        }

        const data: LoginResponse = await response.json();
        return data;
    } catch (error) {
        console.error('Error sending ID token to backend:', JSON.stringify(error));
        throw new Error(`${error} - Failed to authenticate with backend.`);
    }
};

export const logout = async (): Promise<void> => {
    try {
        // Implement your backend logout endpoint if needed
        // await fetch(`${API_BASE_URL}/logout`, { method: 'POST' });
        localStorage.removeItem('authToken'); // Clear token from frontend
    } catch (error) {
        console.error('Logout failed:', error);
        throw error;
    }
};