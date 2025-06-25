import React from 'react';
import {
  createBrowserRouter,
  Link,
  Outlet,
  useNavigate,
  useLocation,
} from 'react-router-dom';
import { Button, Container, Footer, Panel, CustomProvider } from 'rsuite';
// import { FaReact } from 'react-icons/fa'; // Example: Importing an icon from React Icons

// Ensure your custom LESS/CSS is imported here.
// The RSuite base styles will now be handled through your custom LESS file.
// This import should now correctly resolve after ensuring all steps are followed.
import '../style.less';
import AppMenu from './components/AppMenu';
import TopBar from './components/layout/TopBar';
import Login from './components/user/Login';
import PersonalPage from './components/user/PersonalPage';
import { AuthProvider } from './components/user/AuthProvider';

// Define the root App component for routing
// eslint-disable-next-line react-refresh/only-export-components
const App = () => {
  const [isMenuOpen, setIsMenuOpen] = React.useState(false);
  const [appName, setAppName] = React.useState("");
  const navigate = useNavigate();
  const location = useLocation();

  return (
    <AuthProvider>
      <div className="min-h-screen flex flex-col font-sans">
        <TopBar
          currentAppName={appName}
          isMenuOpen={isMenuOpen}
          setIsMenuOpen={setIsMenuOpen}
        />
        <AppMenu
          isOpen={isMenuOpen}
          currentPath={location.pathname}
          onAppChanged={(app) => {
            setAppName(app.name);
            console.log("App name: ", app.name);
          }}
          onClose={() => setIsMenuOpen(false)}
          navigate={navigate}
        />

        <Container className="flex-1 p-4 md:p-8">
          <Outlet /> {/* This is where nested routes will be rendered */}
        </Container>

        <Footer className="p-4 text-center shadow-inner">
          © {new Date().getFullYear()} My App. All rights reserved.
        </Footer>
      </div>
    </AuthProvider>
  );
};

// Home Page Component
const HomePage = () => (
  <Panel header={<h2 className="text-2xl font-semibold text-gray-800">Welcome to Your App!</h2>} bordered>
    <p>
      This is a basic frontend application built with Vite, React, TypeScript, RSuite, and React Router.
    </p>
    <p>
      RSuite is integrated, and you can see a sample button below. Explore the navigation to see different routes.
    </p>
    <Button appearance="primary" size="lg">
      Click Me (RSuite Button)
    </Button>
  </Panel>
);

// About Page Component
const AboutPage = () => (
  <Panel header={<h2 className="text-2xl font-semibold text-gray-800">About Us</h2>} bordered>
    <p className="text-gray-700">
      This page demonstrates a simple route within the application.
    </p>
  </Panel>
);

// Dashboard Page Component
const DashboardPage = () => (
  <Panel header={<h2 className="text-2xl font-semibold text-gray-800">Dashboard</h2>} bordered>
    <p>
      This is your private dashboard. More features coming soon!
    </p>
  </Panel>
);

// Configure React Router
const router = createBrowserRouter([
  {
    path: '/',
    element: <CustomProvider theme="dark"><App /></CustomProvider>, // App component acts as the layout
    children: [
      {
        index: true, // This makes HomePage the default child route for '/'
        element: <HomePage />,
      },
      {
        path: 'about',
        element: <AboutPage />,
      },
      {
        path: 'dashboard',
        element: <DashboardPage />,
      },
      {
        path: 'personal',
        element: <PersonalPage />,
      },
      {
        path: 'login',
        element: <Login />,
      },
      // a catch-all for 404 Not Found
      {
        path: '*',
        element: (
          <Panel header={<h2 className="text-2xl font-semibold text-gray-800">404: Not Found</h2>} bordered>
            <p className="text-gray-700 mb-4">Oops! The page you're looking for does not exist.</p>
            <Button appearance="link" as={Link} to="/">Go to Home</Button>
          </Panel>
        ),
      },
    ],
  },
]);

export default router;
