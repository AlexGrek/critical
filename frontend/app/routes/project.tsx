import { Outlet, useLoaderData, NavLink, isRouteErrorResponse, useRouteError, Link, type LoaderFunctionArgs } from "react-router";
import { type LoaderFunction, type MetaFunction } from "react-router";

import { authFetchJson, AuthFetchError } from '~/lib/authFetch'; // Adjust import path as needed
import type { ProjectStateResponse } from "~/lib/models";

// Project type definition
export interface Project {
    id: string;
    name: string;
    description: string;
    status: 'active' | 'inactive' | 'archived';
    createdAt: string;
    updatedAt: string;
    owner: {
        id: string;
        name: string;
        email: string;
    };
    members: Array<{
        id: string;
        name: string;
        email: string;
        role: 'owner' | 'admin' | 'member' | 'viewer';
    }>;
    settings: {
        isPublic: boolean;
        allowExternalCollaborators: boolean;
        defaultTicketPriority: 'low' | 'medium' | 'high' | 'urgent';
    };
    stats: {
        totalTickets: number;
        openTickets: number;
        completedTickets: number;
        totalPipelines: number;
        activePipelines: number;
    };
}

// API response wrapper (if your API returns wrapped responses)
interface ProjectApiResponse {
    success: boolean;
    data: Project;
    message?: string;
}

// Enhanced version with better error handling for React Router loaders
export const fetchProjectWithError = async (projectId: string): Promise<ProjectStateResponse> => {
    if (!projectId) {
        throw new Response('Project ID is required', { status: 400 });
    }

    try {
        const url = `/api/v1/state/describe/project?id=${encodeURIComponent(projectId)}`;
        const project = await authFetchJson<ProjectStateResponse>(url);

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

        console.error(`Failed to fetch project ${projectId}:`, error);
        throw new Response('Failed to load project', { status: 500 });
    }
};

export const clientLoader = async ({ params }: LoaderFunctionArgs ) => {
    const project = await fetchProjectWithError(params.projectId!);
    return { project };
};


export function ErrorBoundary() {
    const error = useRouteError();

    if (isRouteErrorResponse(error)) {
        return (
            <div className="error-page">
                <h1>{error.status} {error.statusText}</h1>
                <p>{error.data}</p>

                {error.status === 404 && (
                    <div>
                        <p>This project doesn't exist or has been deleted.</p>
                        <Link to="/projects">← Back to Projects</Link>
                    </div>
                )}

                {error.status === 403 && (
                    <div>
                        <p>You don't have permission to view this project.</p>
                        <Link to="/dashboard">← Back to Dashboard</Link>
                    </div>
                )}
            </div>
        );
    }

    // Handle unexpected errors
    return (
        <div className="error-page">
            <h1>Unexpected Error</h1>
            <p>Something went wrong. Please try again.</p>
            <button onClick={() => window.location.reload()}>
                Reload Page
            </button>
        </div>
    );
}

export default function ProjectLayout() {
    const { project } = useLoaderData<typeof clientLoader>();

    return (
        <div className="project-layout">
            {/* Top bar - consistent across all project sub-routes */}
            <header className="project-header">
                <div className="project-info">
                    <h1>{project.meta.name_id}</h1>
                    <p>{project.meta.public_name}</p>
                </div>

                {/* Navigation tabs */}
                <nav className="project-nav">
                    <NavLink
                        to={`/project/${project.meta.name_id}`}
                        className={({ isActive }) => isActive ? "active" : ""}
                        end
                    >
                        Overview
                    </NavLink>
                    <NavLink
                        to={`/project/${project.meta.name_id}/tickets`}
                        className={({ isActive }) => isActive ? "active" : ""}
                    >
                        Tickets
                    </NavLink>
                    <NavLink
                        to={`/project/${project.meta.name_id}/settings`}
                        className={({ isActive }) => isActive ? "active" : ""}
                    >
                        Settings
                    </NavLink>
                    <NavLink
                        to={`/project/${project.meta.name_id}/pipelines`}
                        className={({ isActive }) => isActive ? "active" : ""}
                    >
                        Pipelines
                    </NavLink>
                </nav>
            </header>

            {/* This is where the sub-routes render */}
            <main className="project-content">
                <Outlet />
            </main>
        </div>
    );
}