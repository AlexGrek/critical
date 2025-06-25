// src/components/GoogleLoginButton.tsx
import React, { useEffect, useRef } from 'react';
import { GOOGLE_CLIENT_ID } from '../../constants';

interface GoogleLoginButtonProps {
  onSuccess: (idToken: string) => void;
  onFailure?: (error: any) => void;
}

// Extend the Window interface to include google.accounts.id
declare global {
  interface Window {
    google: {
      accounts: {
        id: {
          initialize: (config: { client_id: string; callback: (response: { credential?: string }) => void; auto_select?: boolean; ux_mode?: 'popup' | 'redirect' }) => void;
          renderButton: (element: HTMLElement, options: { theme?: string; size?: string; text?: string; shape?: string; width?: string }) => void;
          prompt: () => void;
          revoke: (args: { hint: string }, callback: (response: { error: string }) => void) => void;
        };
      };
    };
  }
}

const GoogleLoginButton: React.FC<GoogleLoginButtonProps> = ({ onSuccess, onFailure }) => {
  const buttonRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (window.google && buttonRef.current) {
      window.google.accounts.id.initialize({
        client_id: GOOGLE_CLIENT_ID,
        callback: (response) => {
          if (response.credential) {
            onSuccess(response.credential);
          } else if (onFailure) {
            onFailure(new Error('No credential received.'));
          }
        },
        // auto_select: false, // Set to true if you want auto-login for returning users
        // ux_mode: 'popup', // 'redirect' or 'popup' (popup is generally better for SPAs)
      });

      window.google.accounts.id.renderButton(
        buttonRef.current,
        {
          theme: 'outline',
          size: 'large',
          text: 'signin_with', // or 'continue_with'
          shape: 'rectangular',
          width: '300px', // Adjust as needed
        }
      );

      // Optional: One-tap prompt
      // window.google.accounts.id.prompt();
    }
  }, [onSuccess, onFailure]);

  return <div ref={buttonRef}></div>;
};

export default GoogleLoginButton;