import React from 'react';
import {
    createBrowserRouter,
    RouterProvider,
    useLoaderData,
    useRouteError,
    Link,
    type LoaderFunctionArgs,
} from 'react-router';
import { authFetch, AuthFetchError, authFetchJson } from '~/lib/authFetch';
import { fetchProjectWithError } from './project';

// Define the TypeScript interfaces for the data based on your Rust structs
interface ProjectGitopsSerializable {
    id: string;
    name: string;
    // Add other fields as needed from your ProjectGitopsSerializable
}

interface TicketGitopsSerializable {
    id: string;
    title: string;
    // Add other fields as needed from your TicketGitopsSerializable
}

interface UserPublicDataGitopsSerializable {
    id: string;
    username: string;
    email: string;
    // Add other fields as needed from your UserPublicDataGitopsSerializable
}

interface UserDashboard {
    recentAndOwnedProjects: ProjectGitopsSerializable[];
    recentTickets: TicketGitopsSerializable[];
    me: UserPublicDataGitopsSerializable;
}

export const fetchDashboardWithError = async (): Promise<UserDashboard> => {
    try {
        const url = `/api/v1/personal/dashboard`;
        const project = await authFetchJson<UserDashboard>(url);

        if (!project) {
            throw new Response('Project not found', { status: 404 });
        }

        return project;
    } catch (error) {
        if (error instanceof Response) {
            throw error; // Re-throw Response objects for React Router
        }

        if (error instanceof AuthFetchError) {
            if (error.status === 401) {
                throw new Response('Unauthorized', { status: 401 });
            }
            if (error.status === 403) {
                throw new Response('Access denied', { status: 403 });
            }
            if (error.status === 404) {
                throw new Response('Project not found', { status: 404 });
            }
        }

        console.error(`Failed to fetch dashboard:`, error);
        throw new Response('Failed to load project', { status: 500 });
    }
};


/**
 * clientLoader function for the dashboard route.
 * This function is called by React Router before the DashboardLayout component renders.
 * It fetches the necessary data for the dashboard.
 * @returns {Promise<UserDashboard>} The dashboard data.
 * @throws {Error} If data fetching fails.
 */
export const clientLoader = async ({ params }: LoaderFunctionArgs ) => {
    const dashboard = await fetchDashboardWithError();
    return { dashboard };
};

/**
 * ErrorBoundary component to catch and display errors for the route.
 * This component will render if the loader or the DashboardLayout component throws an error.
 */
const ErrorBoundary: React.FC = () => {
    const error: any = useRouteError(); // Get the error thrown by the route
    console.error("ErrorBoundary caught an error:", error);

    let errorMessage: string;
    let errorStatus: number | undefined;

    if (error instanceof Response) {
        // If the error is a Response object (e.g., from loader throwing a Response)
        errorMessage = error.statusText || 'An unexpected network error occurred.';
        errorStatus = error.status;
    } else if (error instanceof Error) {
        // If the error is a standard JavaScript Error object
        errorMessage = error.message;
    } else {
        // Fallback for unknown error types
        errorMessage = 'An unknown error occurred.';
    }

    return (
        <div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center p-4 font-sans">
            <div className="bg-white p-8 rounded-xl shadow-lg w-full max-w-md text-center">
                <h1 className="text-3xl font-bold text-red-600 mb-4">Oops!</h1>
                <p className="text-gray-700 mb-2 text-lg">Something went wrong.</p>
                {errorStatus && <p className="text-gray-700 mb-4">Status: {errorStatus}</p>}
                <pre className="bg-red-50 p-4 rounded-lg text-sm text-red-800 overflow-auto max-h-48 border border-red-200">
                    {errorMessage}
                </pre>
                <Link to="/" className="mt-6 inline-block bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-lg shadow-md transition duration-300 ease-in-out">
                    Go Home
                </Link>
            </div>
        </div>
    );
};

/**
 * DashboardLayout component. This serves as the main component for the /dashboard route.
 * It uses useLoaderData to access the data fetched by clientLoader.
 */
export default function DashboardLayout() {
    // useLoaderData hook provides the data returned by the clientLoader function
    const { dashboard } = useLoaderData<typeof clientLoader>();

    return (
        <div className="min-h-screen bg-gray-100 flex flex-col font-sans">
            {/* Header */}
            <header className="bg-blue-800 text-white p-4 shadow-md">
                <nav className="flex justify-between items-center max-w-4xl mx-auto">
                    <Link to="/" className="text-2xl font-bold rounded-md hover:bg-blue-700 p-2 transition duration-200">
                        My Dashboard App
                    </Link>
                    <div>
                        <Link to="/" className="text-white hover:text-blue-200 px-3 py-2 rounded-md transition duration-200">Home</Link>
                    </div>
                </nav>
            </header>

            {/* Main Content Area */}
            <main className="flex-grow flex items-center justify-center p-4">
                <div className="bg-white p-8 rounded-xl shadow-lg w-full max-w-2xl">
                    <h1 className="text-3xl font-bold text-gray-800 mb-6 text-center">Dashboard</h1>

                    {/* Display fetched data */}
                    <div className="space-y-4">
                        <h2 className="text-xl font-semibold text-gray-700">Fetched Data (JSON String):</h2>
                        <pre className="bg-gray-50 p-4 rounded-lg text-sm overflow-auto max-h-96 border border-gray-200">
                            {JSON.stringify(dashboard, null, 2)}
                        </pre>

                        <div className="mt-6 p-4 bg-blue-50 rounded-lg border border-blue-200">
                            <h3 className="text-lg font-medium text-blue-800 mb-2">Recent Projects:</h3>
                            <ul className="list-disc list-inside text-gray-700">
                                {dashboard.recentAndOwnedProjects.length > 0 ? (
                                    dashboard.recentAndOwnedProjects.map((project) => (
                                        <li key={project.id}>{project.name} (ID: {project.id})</li>
                                    ))
                                ) : (
                                    <li>No recent projects.</li>
                                )}
                            </ul>
                        </div>

                        <div className="p-4 bg-green-50 rounded-lg border border-green-200">
                            <h3 className="text-lg font-medium text-green-800 mb-2">Recent Tickets:</h3>
                            <ul className="list-disc list-inside text-gray-700">
                                {dashboard.recentTickets.length > 0 ? (
                                    dashboard.recentTickets.map((ticket) => (
                                        <li key={ticket.id}>{ticket.title} (ID: {ticket.id})</li>
                                    ))
                                ) : (
                                    <li>No recent tickets.</li>
                                )}
                            </ul>
                        </div>

                        <div className="p-4 bg-purple-50 rounded-lg border border-purple-200">
                            <h3 className="text-lg font-medium text-purple-800 mb-2">User Info:</h3>
                            <p className="text-gray-700">
                                <strong>Email:</strong> {dashboard.me.email}
                            </p>
                        </div>
                    </div>
                </div>
            </main>

            {/* Footer */}
            <footer className="bg-gray-800 text-white p-4 text-center text-sm">
                &copy; {new Date().getFullYear()} My Dashboard App. All rights reserved.
            </footer>
        </div>
    );
}
