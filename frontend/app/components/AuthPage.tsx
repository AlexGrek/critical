import { useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { Tabs, TabsList, TabsTrigger } from "@radix-ui/react-tabs";
import { useFloating, shift, offset, autoUpdate } from "@floating-ui/react";
import { useNavigate } from "react-router";

const BackgroundBlur = () => (
  <div className="absolute inset-0 z-[-1] overflow-hidden">
    {Array.from({ length: 10 }).map((_, i) => (
      <div
        key={i}
        className="absolute rounded-full opacity-40 blur-3xl mix-blend-screen"
        style={{
          backgroundColor: `hsl(${300 + i * 6}, 70%, 30%)`,
          width: `${200 + Math.random() * 200}px`,
          height: `${200 + Math.random() * 200}px`,
          top: `${Math.random() * 100}%`,
          left: `${Math.random() * 100}%`,
        }}
      />
    ))}
    <div className="absolute inset-0 bg-black/60 backdrop-blur-md" />
  </div>
);

const Logo = () => (
  <div className="text-6xl font-mono text-center text-white mb-10 select-none">
    {'{'}<span className="text-red-600">!</span>{'}'}
  </div>
);

const Input = ({ placeholder, type = "text", name }: { placeholder: string; type?: string; name?: string }) => (
  <input
    name={name}
    type={type}
    placeholder={placeholder}
    className="w-full bg-white/10 text-white text-sm p-3 mb-4 border-none focus:outline-none focus:ring-2 focus:ring-red-600 placeholder-white/60"
  />
);

const Button = ({ children }: { children: React.ReactNode }) => (
  <motion.button
    whileHover={{ scale: 1.02 }}
    whileTap={{ scale: 0.98 }}
    className="w-full bg-red-600 text-black text-sm p-3 hover:bg-black hover:text-red-600 transition-colors"
  >
    {children}
  </motion.button>
);

const authFetch = async (url: string, data: any) => {
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error(await response.text());
  return response.json();
};

export default function AuthPage() {
  const [tab, setTab] = useState("login");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const navigate = useNavigate(); // or useNavigate()

  const handleLogin = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    setLoading(true);
    const form = e.currentTarget;
    const data = {
      uid: form.username.value,
      password: form.password.value,
    };
    try {
      const res = await authFetch("/api/v1/login", data);
      localStorage.setItem("token", res.token);
      navigate("/dashboard"); // or navigate('/dashboard')
    } catch (err: any) {
      setError(err.message || "Login failed");
    } finally {
      setLoading(false);
    }
  };

  const handleRegister = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    setError(null);
    setLoading(true);
    const form = e.currentTarget;
    const password = form.password.value;
    const repeat = form.repeat_password.value;
    if (password !== repeat) {
      setError("Passwords do not match");
      setLoading(false);
      return;
    }
    const data = {
      uid: form.username.value,
      email: form.email.value,
      password,
      invite_id: form.invite_id.value,
      invite_key: form.invite_key.value,
    };
    try {
      const res = await authFetch("/api/v1/register", data);
      localStorage.setItem("token", res.token);
      navigate("/dashboard");
    } catch (err: any) {
      setError(err.message || "Registration failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="w-screen min-h-screen flex items-center justify-center bg-black relative overflow-hidden text-white">
      <BackgroundBlur />
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.4, ease: "easeOut" }}
        className="w-full max-w-sm p-8 bg-gray-900/80 backdrop-blur-md"
      >
        <Logo />
        <Tabs value={tab} onValueChange={setTab}>
          <TabsList className="flex mb-6 text-sm">
            <TabsTrigger
              value="login"
              className={`flex-1 text-center py-2 px-4 text-white ${
                tab === "login" ? "bg-red-600 text-black" : "bg-transparent"
              }`}
            >
              Login
            </TabsTrigger>
            <TabsTrigger
              value="register"
              className={`flex-1 text-center py-2 px-4 text-white ${
                tab === "register" ? "bg-red-600 text-black" : "bg-transparent"
              }`}
            >
              Register
            </TabsTrigger>
          </TabsList>

          <motion.div layout className="relative">
            <AnimatePresence mode="wait" initial={false}>
              {tab === "login" && (
                <motion.div
                  key="login"
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -20 }}
                  transition={{ duration: 0.4 }}
                >
                  <form onSubmit={handleLogin}>
                    <Input placeholder="Username" name="username" />
                    <Input placeholder="Password" type="password" name="password" />
                    <div className="text-right mb-4">
                      <a
                        href="#"
                        className="text-sm text-red-600 hover:underline"
                        onClick={(e) => {
                          e.preventDefault();
                          window.open("/recover", "_blank");
                        }}
                      >
                        Recover password
                      </a>
                    </div>
                    {error && <div className="text-sm text-red-400 mb-4">{error}</div>}
                    <Button>{loading ? "Signing In..." : "Sign In"}</Button>
                  </form>
                </motion.div>
              )}

              {tab === "register" && (
                <motion.div
                  key="register"
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -20 }}
                  transition={{ duration: 0.4 }}
                >
                  <form onSubmit={handleRegister}>
                    <Input placeholder="Username" name="username" />
                    <Input placeholder="Email" type="email" name="email" />
                    <Input placeholder="Password" type="password" name="password" />
                    <Input placeholder="Repeat Password" type="password" name="repeat_password" />
                    <Input placeholder="Invite ID" name="invite_id" />
                    <Input placeholder="Invite Key" name="invite_key" />
                    <label className="text-xs mb-4 inline-flex items-start gap-2">
                      <input type="checkbox" className="mt-1" required />
                      <span>
                        I agree to the{' '}
                        <a
                          href="/tos"
                          target="_blank"
                          className="text-red-600 underline"
                        >
                          Terms of Service
                        </a>
                      </span>
                    </label>
                    {error && <div className="text-sm text-red-400 mb-4">{error}</div>}
                    <Button>{loading ? "Registering..." : "Sign Up"}</Button>
                  </form>
                </motion.div>
              )}
            </AnimatePresence>
          </motion.div>
        </Tabs>
      </motion.div>
    </div>
  );
}
