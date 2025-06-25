import React from 'react';
// import { FaRegEye, FaRegEyeSlash } from 'react-icons/fa';
import PlusIcon from '@rsuite/icons/Plus';
import { Button, Divider, Form, IconButton, Panel, Stack, VStack, Drawer, Message, toaster, ButtonGroup } from 'rsuite';
import { useNavigate } from 'react-router-dom';
import { useAuth } from './AuthProvider';
import { logout } from '../../utils';
import { fetchUserWhoami } from './login_utils';
import GoogleLoginButton from './GoogleLoginButton';
import { googleLogin } from '../../services/authService';

interface LoginProps {
    from?: string;
}

const Login: React.FC<LoginProps> = ({ from }) => {
    const navigate = useNavigate();
    const auth = useAuth();
    const [formData, setFormData] = React.useState<Record<string, unknown>>({
        'name': '',
        'password': ''
    })
    const [registrationData, setRegistrationData] = React.useState<Record<string, unknown>>({
        'email': '',
        'password': '',
        'confirmPassword': '',
        'firstName': '',
        'lastName': ''
    });
    const [loading, setLoading] = React.useState(false);
    const [error, setError] = React.useState<string | null>(null);
    const [registrationLoading, setRegistrationLoading] = React.useState(false);
    const [forgot, setForgot] = React.useState(false);
    const [drawerOpen, setDrawerOpen] = React.useState(false);

    const handleLogin = async () => {
        setLoading(true);
        try {
            const formDataValues = new URLSearchParams();
            formDataValues.append('username', formData['name'] as string);
            formDataValues.append('password', formData['password'] as string);

            const response = await fetch('/api/v1/auth/token', {
                method: 'POST',
                headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
                body: formDataValues
            });

            if (response.status === 401) {
                throw new Error('Unauthorized: Incorrect username or password.');
            }

            if (!response.ok) throw new Error('Login failed. Please try again.');

            const data = await response.json();
            console.log("Logged in: " + JSON.stringify(data))
            if (data["access_token"]) {
                localStorage.setItem('authToken', data["access_token"]); // Store the token in localStorage
                await fetchUserWhoami(auth);
                if (from) {
                    navigate(from)
                } else {
                    navigate('/')
                }
            }
        } catch (error) {
            alert(error)
            console.error(error)
        } finally {
            setLoading(false);
        }
    };

    const handleGoogleSuccess = async (idToken: string) => {
        setLoading(true);
        setError(null);
        try {
            // 1. Send the Google ID token to your backend
            const data = await googleLogin(idToken);

            // 2. Store your app's session token/JWT received from backend
            localStorage.setItem('authToken', data.token); // Example: store in localStorage

            // You might also use an AuthContext for global state management

            // 3. Navigate to the dashboard or home page
            await fetchUserWhoami(auth);
            if (from) {
                navigate(from)
            } else {
                navigate('/')
            }
        } catch (err: unknown) {
            setError('Login failed. Please try again: ' + JSON.stringify(err));
            console.error('Frontend login error:', err);
        } finally {
            setLoading(false);
        }
    };

    const handleGoogleFailure = (error: unknown) => {
        console.error('Google Sign-In Error:', error);
        setError('Google login failed. Please try again: ' + JSON.stringify(error));
        setLoading(false);
    };


    const handleRegistration = async () => {
        // Validate password
        if (!registrationData.password || (registrationData.password as string).length < 6) {
            toaster.push(
                <Message type="error">Password must be at least 6 characters long</Message>,
                { duration: 3000 }
            );
            return;
        }

        // Validate password confirmation
        if (registrationData.password !== registrationData.confirmPassword) {
            toaster.push(
                <Message type="error">Passwords do not match</Message>,
                { duration: 3000 }
            );
            return;
        }

        // Validate required fields
        if (!registrationData.email || !registrationData.firstName || !registrationData.lastName) {
            toaster.push(
                <Message type="error">Please fill in all required fields</Message>,
                { duration: 3000 }
            );
            return;
        }

        setRegistrationLoading(true);
        try {
            const response = await fetch('/api/v1/auth/register', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({
                    email: registrationData.email,
                    password: registrationData.password,
                    firstName: registrationData.firstName,
                    lastName: registrationData.lastName
                })
            });

            if (!response.ok) {
                const errorData = await response.json();
                throw new Error(errorData.message || 'Registration failed. Please try again.');
            }

            const data = await response.json();

            // Show success message
            toaster.push(
                <Message type="success">Registration successful! You can now log in. <br />{JSON.stringify(data)}</Message>,
                { duration: 3000 }
            );

            // Close drawer and reset form
            setDrawerOpen(false);
            setRegistrationData({
                'email': '',
                'password': '',
                'confirmPassword': '',
                'firstName': '',
                'lastName': ''
            });

        } catch (error) {
            toaster.push(
                <Message type="error">{error instanceof Error ? error.message : 'Registration failed'}</Message>,
                { duration: 3000 }
            );
            console.error(error);
        } finally {
            setRegistrationLoading(false);
        }
    };

    const handleLogout = () => {
        auth.setUserInfo(null);
        logout();
    }

    const getPasswordValidationMessage = () => {
        const password = registrationData.password as string;
        if (!password) return null;

        if (password.length < 6) {
            return <Message type="warning" style={{ marginTop: 5 }}>Password must be at least 6 characters</Message>;
        }
        return <Message type="success" style={{ marginTop: 5 }}>Password meets requirements</Message>;
    };

    const getConfirmPasswordValidationMessage = () => {
        const password = registrationData.password as string;
        const confirmPassword = registrationData.confirmPassword as string;

        if (!confirmPassword) return null;

        if (password !== confirmPassword) {
            return <Message type="error" style={{ marginTop: 5 }}>Passwords do not match</Message>;
        }
        return <Message type="success" style={{ marginTop: 5 }}>Passwords match</Message>;
    };

    return (
        <>
            <Stack alignItems="center" justifyContent="center" style={{ height: '100%' }}>
                <Panel header="Sign in" bordered style={{ width: 400 }}>
                    {error && <Panel header={"Error logging in"}><pre><code>{error}</code></pre></Panel>}
                    {auth.getCurrentUserId() != null && <h3>You are already logged in. <Button appearance='link' onClick={handleLogout}>Logout</Button></h3>}
                    <Form fluid formValue={formData} onChange={setFormData} disabled={auth.getCurrentUserId() != null}>
                        <Form.Group>
                            <Form.ControlLabel>Email address</Form.ControlLabel>
                            <Form.Control name="name" />
                        </Form.Group>
                        <Form.Group>
                            <Form.ControlLabel>Password</Form.ControlLabel>
                            <Form.Control name="password" type="password" />
                        </Form.Group>



                        <VStack spacing={10}>
                            <Button appearance="primary" block onClick={handleLogin} disabled={loading || auth.getCurrentUserId() != null}>
                                Sign in
                            </Button>
                            <a href="#" onClick={() => setForgot(!forgot)}>Forgot password?</a>
                            {forgot && <p>Bad for you, we do not provide any password recovery. You are <b>completely fucked</b>. Better luck next time!</p>}

                            <Divider>Verified login</Divider>
                            <Stack style={{width: '100%'}} justifyContent='center'><GoogleLoginButton onSuccess={handleGoogleSuccess} onFailure={handleGoogleFailure} /></Stack>
                        </VStack>
                    </Form>

                    <Divider>OR</Divider>

                    <IconButton
                        icon={<PlusIcon />}
                        block
                        onClick={() => setDrawerOpen(true)}
                        disabled={loading || auth.getCurrentUserId() != null}
                    >
                        Complete quick registration
                    </IconButton>
                </Panel>
            </Stack>

            {/* Registration Drawer */}
            <Drawer
                open={drawerOpen}
                onClose={() => setDrawerOpen(false)}
                placement="top"
                size="lg"
            >
                <Drawer.Header>
                    <Drawer.Title>Quick Registration</Drawer.Title>
                </Drawer.Header>
                <Drawer.Body>
                    <Form
                        fluid
                        formValue={registrationData}
                        onChange={setRegistrationData}
                        style={{ maxWidth: 600, margin: '0 auto' }}
                    >
                        <Stack spacing={20} direction="column">
                            <Stack spacing={20} wrap>
                                <Form.Group style={{ flex: 1, minWidth: 250 }}>
                                    <Form.ControlLabel>First Name *</Form.ControlLabel>
                                    <Form.Control name="firstName" />
                                </Form.Group>
                                <Form.Group style={{ flex: 1, minWidth: 250 }}>
                                    <Form.ControlLabel>Last Name *</Form.ControlLabel>
                                    <Form.Control name="lastName" />
                                </Form.Group>
                            </Stack>

                            <Form.Group>
                                <Form.ControlLabel>Email Address *</Form.ControlLabel>
                                <Form.Control name="email" type="email" />
                            </Form.Group>

                            <Form.Group>
                                <Form.ControlLabel>Password *</Form.ControlLabel>
                                <Form.Control name="password" type="password" />
                                {getPasswordValidationMessage()}
                            </Form.Group>

                            <Form.Group>
                                <Form.ControlLabel>Confirm Password *</Form.ControlLabel>
                                <Form.Control name="confirmPassword" type="password" />
                                {getConfirmPasswordValidationMessage()}
                            </Form.Group>
                        </Stack>
                        <ButtonGroup style={{ display: "flex", margin: "6em auto", width: "100%", justifyContent: "center" }}>
                            <Button
                                onClick={handleRegistration}
                                appearance="primary"
                                loading={registrationLoading}
                                disabled={registrationLoading}
                            >
                                Register
                            </Button>
                            <Button
                                onClick={() => setDrawerOpen(false)}
                                appearance="subtle"
                                disabled={registrationLoading}
                            >
                                Cancel
                            </Button>
                        </ButtonGroup>
                    </Form>

                </Drawer.Body>
            </Drawer>
        </>
    );
};

export default Login;