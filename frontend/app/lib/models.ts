

// Project type definitions based on Rust structs
export interface VisibilityConfig {
    public_visible: boolean;
    public_can_report: boolean;
    public_can_see_tickets: string[];
}

export interface ProjectLinks {
    github: string;
}

export interface ProjectTicketCategory {
    supported_statuses: string[];
}

export interface Project {
    name_id: string;
    public_name: string;
    owner_uid: string;
    admins_uid: string[];
    visibility: VisibilityConfig;
    links: ProjectLinks;
    ticket_categories: Record<string, ProjectTicketCategory>;
    pipelines_feature_enabled: boolean;
    releases_feature_enabled: boolean;
    short_description: string;
    readme: string;
}

// Helper function to create a new project with defaults
export const createDefaultProject = (overrides: Partial<Project> = {}): Project => ({
    name_id: '',
    public_name: '',
    owner_uid: '',
    admins_uid: [],
    visibility: defaultVisibilityConfig(),
    links: defaultProjectLinks(),
    ticket_categories: {},
    pipelines_feature_enabled: false,
    releases_feature_enabled: false,
    short_description: '',
    readme: '',
    ...overrides,
});

export const defaultProjectTicketCategory = (): ProjectTicketCategory => ({
    supported_statuses: defaultSupportedStatuses(),
});

export const defaultSupportedStatuses = (): string[] => [
    'Open',
    'In Progress',
    'Resolved',
    'Closed',
    'Reopened',
    'To Do',
    'Done',
    'Blocked',
];

// Default values (matching Rust defaults)
export const defaultVisibilityConfig = (): VisibilityConfig => ({
    public_visible: false,
    public_can_report: false,
    public_can_see_tickets: [],
});

export const defaultProjectLinks = (): ProjectLinks => ({
    github: '',
});

// Additional state interfaces
export interface ProjectState {
    total_tickets: number; // isize maps to number in TypeScript
}

export interface ProjectStateResponse {
    meta: Project; // ProjectGitopsSerializable is the serializable version of Project
    state: ProjectState;
}
