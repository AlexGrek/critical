import { useState, useEffect, useMemo, useCallback } from "react";
import {
  Button,
  Input,
  Modal,
  MorphModal,
  LogoCritical,
  LogoCriticalAnimated,
  ThemeCombobox,
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
  H1,
  H2,
  H3,
  H4,
  H5,
  H6,
  Paragraph,
  CodeBlock,
  InlineCode,
  ScrollableLogWindow,
  YamlEditor,
} from "~/components";

export function meta() {
  return [
    { title: "{!} UI Gallery - Critical" },
    { name: "description", content: "Component showcase for Critical UI" },
  ];
}

export default function UiGallery() {
  const [inputValue, setInputValue] = useState("");
  const [yamlResource, setYamlResource] = useState<Record<string, unknown>>({
    id: "p_example",
    name: "example-project",
    description: "A sample project for the UI gallery",
    labels: { env: "production", team: "platform" },
    annotations: { "last-deployed": "2026-03-07" },
    state: { created_at: "2026-01-01T00:00:00Z", updated_at: "2026-03-07T12:00:00Z" },
    hash_code: "abc123",
  });

  const editableYamlValue = useMemo(
    () => yamlResource,
    [yamlResource]
  );

  const readOnlyYamlValue = useMemo<Record<string, unknown>>(
    () => ({
      id: "g_platform_admins",
      name: "platform-admins",
      description: "Platform admin group — managed by infrastructure team",
      labels: { tier: "admin", managed: "true" },
      state: { created_at: "2025-12-01T00:00:00Z", updated_at: "2026-02-14T09:30:00Z" },
      hash_code: "deadbeef",
    }),
    []
  );

  const handleYamlSave = useCallback(async (parsed: Record<string, unknown>) => {
    // Simulate an API call
    await new Promise((resolve) => setTimeout(resolve, 600));
    setYamlResource((prev) => ({ ...prev, ...parsed }));
  }, []);
  const [logs, setLogs] = useState<string[]>([
    "[INFO] Application started",
    "[INFO] Connecting to database...",
    "[SUCCESS] Database connection established",
    "[INFO] Loading configuration...",
    "[SUCCESS] Configuration loaded successfully",
  ]);

  // Simulate live log updates
  useEffect(() => {
    const interval = setInterval(() => {
      const newLogs = [
        "[INFO] Processing request...",
        "[DEBUG] Cache hit for key: user_123",
        "[INFO] API call completed in 142ms",
        "[WARNING] Rate limit at 80%",
        "[ERROR] Connection timeout",
        "[SUCCESS] Transaction committed",
        "[INFO] Memory usage: 45%",
        "[DEBUG] Query executed successfully",
      ];
      const randomLog = newLogs[Math.floor(Math.random() * newLogs.length)];
      const timestamp = new Date().toLocaleTimeString();
      setLogs((prev) => [...prev, `[${timestamp}] ${randomLog}`]);
    }, 3000);

    return () => clearInterval(interval);
  }, []);

  return (
    <div className="min-h-screen bg-gradient-subtle dark:bg-gray-950 p-8">
      <div className="max-w-6xl mx-auto space-y-12">
        {/* Header */}
        <header className="text-center space-y-4">
          <div className="flex justify-center gap-4">
            <LogoCritical size="lg" />
            <LogoCriticalAnimated size="lg" />
          </div>
          <h1 className="text-4xl font-bold text-gray-900 dark:text-gray-50">
            {"{!} "}UI Component Gallery
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            A showcase of all available UI components
          </p>

          {/* Theme Switcher */}
          <div className="flex justify-center pt-4">
            <ThemeCombobox />
          </div>
        </header>

        {/* Theme Switcher Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Theme Switcher
          </h2>
          <Card>
            <CardHeader>
              <CardTitle>Color Theme Selector</CardTitle>
              <CardDescription>
                Choose between Light, Dark, and Grayscale (low contrast, eye-saving) modes.
                Your preference is saved to localStorage and persists across sessions.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ThemeCombobox />
            </CardContent>
          </Card>
        </section>

        {/* Logos Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Logos
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <Card className="p-6 space-y-3">
              <CardTitle>Static Logo (Small)</CardTitle>
              <LogoCritical size="sm" />
            </Card>
            <Card className="p-6 space-y-3">
              <CardTitle>Static Logo (Medium)</CardTitle>
              <LogoCritical size="md" />
            </Card>
            <Card className="p-6 space-y-3">
              <CardTitle>Static Logo (Large)</CardTitle>
              <LogoCritical size="lg" />
            </Card>
            <Card className="p-6 space-y-3">
              <CardTitle>Animated Logo (Hover)</CardTitle>
              <LogoCriticalAnimated size="md" />
            </Card>
          </div>
        </section>

        {/* Buttons Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Buttons
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* Primary */}
            <Card className="p-6 space-y-3">
              <CardTitle>Primary</CardTitle>
              <div className="space-y-2">
                <Button variant="primary" size="sm">
                  Small Button
                </Button>
                <Button variant="primary" size="default">
                  Default Button
                </Button>
                <Button variant="primary" size="lg">
                  Large Button
                </Button>
              </div>
            </Card>

            {/* Secondary */}
            <Card className="p-6 space-y-3">
              <CardTitle>Secondary</CardTitle>
              <div className="space-y-2">
                <Button variant="secondary" size="sm">
                  Small Button
                </Button>
                <Button variant="secondary" size="default">
                  Default Button
                </Button>
                <Button variant="secondary" size="lg">
                  Large Button
                </Button>
              </div>
            </Card>

            {/* Destructive */}
            <Card className="p-6 space-y-3">
              <CardTitle>Destructive</CardTitle>
              <div className="space-y-2">
                <Button variant="destructive" size="sm">
                  Delete
                </Button>
                <Button variant="destructive" size="default">
                  Remove Item
                </Button>
                <Button variant="destructive" size="lg">
                  Confirm Deletion
                </Button>
              </div>
            </Card>

            {/* Outline */}
            <Card className="p-6 space-y-3">
              <CardTitle>Outline</CardTitle>
              <div className="space-y-2">
                <Button variant="outline" size="sm">
                  Outline Small
                </Button>
                <Button variant="outline" size="default">
                  Outline Default
                </Button>
                <Button variant="outline" size="lg">
                  Outline Large
                </Button>
              </div>
            </Card>

            {/* Ghost */}
            <Card className="p-6 space-y-3">
              <CardTitle>Ghost</CardTitle>
              <div className="space-y-2">
                <Button variant="ghost" size="sm">
                  Ghost Small
                </Button>
                <Button variant="ghost" size="default">
                  Ghost Default
                </Button>
                <Button variant="ghost" size="lg">
                  Ghost Large
                </Button>
              </div>
            </Card>

            {/* Link */}
            <Card className="p-6 space-y-3">
              <CardTitle>Link</CardTitle>
              <div className="space-y-2">
                <Button variant="link" size="sm">
                  Link Small
                </Button>
                <Button variant="link" size="default">
                  Link Default
                </Button>
                <Button variant="link" size="lg">
                  Link Large
                </Button>
              </div>
            </Card>
          </div>
        </section>

        {/* Disabled States Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Disabled States
          </h2>
          <Paragraph variant="muted" className="text-sm">
            All interactive elements support disabled states with consistent styling:
            opacity reduction, cursor change, and pointer events disabled.
          </Paragraph>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* Disabled Primary Buttons */}
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Primary</CardTitle>
              <div className="space-y-2">
                <Button variant="primary" size="sm" disabled>
                  Small Disabled
                </Button>
                <Button variant="primary" size="default" disabled>
                  Default Disabled
                </Button>
                <Button variant="primary" size="lg" disabled>
                  Large Disabled
                </Button>
              </div>
            </Card>

            {/* Disabled Secondary Buttons */}
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Secondary</CardTitle>
              <div className="space-y-2">
                <Button variant="secondary" size="sm" disabled>
                  Small Disabled
                </Button>
                <Button variant="secondary" size="default" disabled>
                  Default Disabled
                </Button>
                <Button variant="secondary" size="lg" disabled>
                  Large Disabled
                </Button>
              </div>
            </Card>

            {/* Disabled Destructive Buttons */}
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Destructive</CardTitle>
              <div className="space-y-2">
                <Button variant="destructive" size="sm" disabled>
                  Delete Disabled
                </Button>
                <Button variant="destructive" size="default" disabled>
                  Remove Disabled
                </Button>
                <Button variant="destructive" size="lg" disabled>
                  Confirm Disabled
                </Button>
              </div>
            </Card>

            {/* Disabled Outline Buttons */}
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Outline</CardTitle>
              <div className="space-y-2">
                <Button variant="outline" size="sm" disabled>
                  Outline Disabled
                </Button>
                <Button variant="outline" size="default" disabled>
                  Outline Disabled
                </Button>
                <Button variant="outline" size="lg" disabled>
                  Outline Disabled
                </Button>
              </div>
            </Card>

            {/* Disabled Ghost Buttons */}
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Ghost</CardTitle>
              <div className="space-y-2">
                <Button variant="ghost" size="sm" disabled>
                  Ghost Disabled
                </Button>
                <Button variant="ghost" size="default" disabled>
                  Ghost Disabled
                </Button>
                <Button variant="ghost" size="lg" disabled>
                  Ghost Disabled
                </Button>
              </div>
            </Card>

            {/* Disabled Link Buttons */}
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Link</CardTitle>
              <div className="space-y-2">
                <Button variant="link" size="sm" disabled>
                  Link Disabled
                </Button>
                <Button variant="link" size="default" disabled>
                  Link Disabled
                </Button>
                <Button variant="link" size="lg" disabled>
                  Link Disabled
                </Button>
              </div>
            </Card>
          </div>

          {/* Disabled Inputs & Modals */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Inputs</CardTitle>
              <div className="space-y-3">
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Disabled Text Input
                  </label>
                  <Input
                    disabled
                    placeholder="This input is disabled"
                    defaultValue="Read-only content"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Disabled Monospace Input
                  </label>
                  <Input
                    disabled
                    monospace
                    placeholder="Disabled monospace input"
                    defaultValue="const locked = true;"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    Disabled Copyable Input
                  </label>
                  <Input
                    disabled
                    copyable
                    placeholder="Disabled copyable"
                    defaultValue="Copy button is greyed out"
                  />
                </div>
              </div>
            </Card>

            {/* Disabled Modal Buttons */}
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Modal Actions</CardTitle>
              <Paragraph variant="muted" size="sm">
                Buttons within modals can also be disabled to prevent actions when
                conditions aren't met (e.g., form validation).
              </Paragraph>
              <Modal.Root>
                <Modal.Trigger asChild>
                  <Button>Open Modal with Disabled Action</Button>
                </Modal.Trigger>
                <Modal.Content>
                  <Modal.Header>
                    <Modal.Title>Disabled Action Example</Modal.Title>
                    <Modal.Close asChild>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
                      >
                        ✕
                      </Button>
                    </Modal.Close>
                  </Modal.Header>
                  <div className="flex-1 overflow-y-auto p-4">
                    <Modal.Description className="mb-3">
                      The confirm button below is disabled until you complete the required
                      action.
                    </Modal.Description>
                    <p className="text-sm text-gray-600 dark:text-gray-400 mb-3">
                      This might represent a form with missing required fields or other
                      validation errors.
                    </p>
                    <Input disabled placeholder="Example disabled field" />
                  </div>
                  <Modal.Footer>
                    <Modal.Close asChild>
                      <Button variant="outline" size="sm">Cancel</Button>
                    </Modal.Close>
                    <Button size="sm" disabled>Confirm (Disabled)</Button>
                  </Modal.Footer>
                </Modal.Content>
              </Modal.Root>
            </Card>
          </div>
        </section>

        {/* Typography Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Typography
          </h2>
          <Card className="p-6 space-y-6">
            <div className="space-y-4">
              <CardTitle>Headers (H1-H6)</CardTitle>
              <div className="space-y-4 divide-y divide-gray-200 dark:divide-gray-800">
                <div className="pt-4 first:pt-0">
                  <H1>Heading 1 - The Quick Brown Fox</H1>
                </div>
                <div className="pt-4">
                  <H2>Heading 2 - The Quick Brown Fox</H2>
                </div>
                <div className="pt-4">
                  <H3>Heading 3 - The Quick Brown Fox</H3>
                </div>
                <div className="pt-4">
                  <H4>Heading 4 - The Quick Brown Fox</H4>
                </div>
                <div className="pt-4">
                  <H5>Heading 5 - The Quick Brown Fox</H5>
                </div>
                <div className="pt-4">
                  <H6>Heading 6 - The Quick Brown Fox</H6>
                </div>
              </div>
            </div>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card className="p-6 space-y-3">
              <CardTitle>Paragraph Sizes</CardTitle>
              <div className="space-y-3">
                <Paragraph size="xs">
                  Extra small paragraph text for fine print and captions.
                </Paragraph>
                <Paragraph size="sm">
                  Small paragraph text for secondary content.
                </Paragraph>
                <Paragraph size="base">
                  Base paragraph text for body content (default).
                </Paragraph>
                <Paragraph size="lg">
                  Large paragraph text for emphasized content.
                </Paragraph>
                <Paragraph size="xl">
                  Extra large paragraph text for prominent content.
                </Paragraph>
              </div>
            </Card>

            <Card className="p-6 space-y-3">
              <CardTitle>Paragraph Variants</CardTitle>
              <div className="space-y-3">
                <Paragraph variant="default">Default text color</Paragraph>
                <Paragraph variant="muted">Muted text color</Paragraph>
                <Paragraph variant="subtle">Subtle text color</Paragraph>
                <Paragraph variant="primary">Primary color text</Paragraph>
                <Paragraph variant="success">Success message</Paragraph>
                <Paragraph variant="warning">Warning message</Paragraph>
                <Paragraph variant="danger">Error message</Paragraph>
              </div>
            </Card>
          </div>
        </section>

        {/* Code Blocks Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Code Display
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card className="p-6 space-y-3">
              <CardTitle>Inline Code</CardTitle>
              <Paragraph>
                Use <InlineCode>npm install</InlineCode> to install dependencies,
                then run <InlineCode>npm run dev</InlineCode> to start the
                development server.
              </Paragraph>
            </Card>

            <Card className="p-6 space-y-3">
              <CardTitle>Code Block</CardTitle>
              <CodeBlock language="typescript">{`function greet(name: string) {
  return \`Hello, \${name}!\`;
}

const message = greet("World");
console.log(message);`}</CodeBlock>
            </Card>
          </div>

          <Card className="p-6 space-y-3">
            <CardTitle>Multi-language Code Examples</CardTitle>
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Paragraph variant="muted" size="sm">
                  Rust
                </Paragraph>
                <CodeBlock language="rust">{`fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x * 2;
}`}</CodeBlock>
              </div>
              <div className="space-y-2">
                <Paragraph variant="muted" size="sm">
                  JavaScript
                </Paragraph>
                <CodeBlock language="javascript">{`const fetchData = async () => {
  const response = await fetch('/api/data');
  return response.json();
};`}</CodeBlock>
              </div>
            </div>
          </Card>
        </section>

        {/* ScrollableLogWindow Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Log Window
          </h2>
          <Card className="p-6 space-y-3">
            <CardTitle>Scrollable Log Window</CardTitle>
            <CardDescription>
              Terminal-style log display with auto-scroll. New logs appear every 3
              seconds. Scroll up to pause auto-scroll, scroll to bottom to resume.
            </CardDescription>
            <div className="relative">
              <ScrollableLogWindow
                title="Application Logs"
                logs={logs}
                maxHeight="300px"
              />
            </div>
          </Card>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card className="p-6 space-y-3">
              <CardTitle>Compact Log Window</CardTitle>
              <ScrollableLogWindow
                logs={[
                  "Starting build process...",
                  "Compiling src/main.rs",
                  "Compiling src/lib.rs",
                  "Finished in 2.34s",
                  "Build successful",
                ]}
                maxHeight="150px"
              />
            </Card>

            <Card className="p-6 space-y-3">
              <CardTitle>Custom Content</CardTitle>
              <ScrollableLogWindow maxHeight="150px">
                <div className="space-y-2">
                  <div className="text-green-400">✓ All tests passed</div>
                  <div className="text-yellow-400">⚠ 2 warnings</div>
                  <div className="text-blue-400">ℹ Build time: 1.2s</div>
                  <div className="text-white">Total: 42 assertions</div>
                </div>
              </ScrollableLogWindow>
            </Card>
          </div>
        </section>

        {/* Inputs Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Inputs
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card className="p-6 space-y-3">
              <CardTitle>Default Input</CardTitle>
              <Input
                placeholder="Enter some text..."
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
              />
            </Card>
            <Card className="p-6 space-y-3">
              <CardTitle>Monospace Input</CardTitle>
              <Input
                monospace
                placeholder="Monospace font..."
                defaultValue="const x = 42;"
              />
            </Card>
            <Card className="p-6 space-y-3">
              <CardTitle>Disabled Input</CardTitle>
              <Input placeholder="Disabled..." disabled value="Can't edit me" />
            </Card>
            <Card className="p-6 space-y-3">
              <CardTitle>Password Input</CardTitle>
              <Input type="password" placeholder="Enter password..." />
            </Card>
            <Card className="p-6 space-y-3">
              <CardTitle>Copyable Input</CardTitle>
              <Input
                copyable
                placeholder="Text with copy button..."
                defaultValue="Click the copy button!"
              />
            </Card>
            <Card className="p-6 space-y-3">
              <CardTitle>Copyable Monospace</CardTitle>
              <Input
                copyable
                monospace
                placeholder="Code snippet..."
                defaultValue="npm install @critical/ui"
              />
            </Card>
          </div>
        </section>

        {/* Modals Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Modals
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Standard Modal */}
            <Card className="p-6 space-y-3">
              <CardTitle>Standard Modal</CardTitle>
              <Modal.Root>
                <Modal.Trigger asChild>
                  <Button>Open Modal</Button>
                </Modal.Trigger>
                <Modal.Content>
                  <Modal.Header>
                    <Modal.Title>Example Modal</Modal.Title>
                    <Modal.Close asChild>
                      <Button
                        variant="ghost"
                        size="sm"
                        className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
                      >
                        ✕
                      </Button>
                    </Modal.Close>
                  </Modal.Header>
                  <div className="flex-1 overflow-y-auto p-4">
                    <Modal.Description className="mb-3">
                      This is a standard modal dialog built with Radix UI.
                    </Modal.Description>
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                      This modal includes a header, body content, and a footer
                      with action buttons.
                    </p>
                  </div>
                  <Modal.Footer>
                    <Modal.Close asChild>
                      <Button variant="outline" size="sm">Cancel</Button>
                    </Modal.Close>
                    <Button size="sm">Confirm</Button>
                  </Modal.Footer>
                </Modal.Content>
              </Modal.Root>
            </Card>

            {/* Morph Modal */}
            <Card className="p-6 space-y-3">
              <CardTitle>Morph Modal</CardTitle>
              <MorphModal
                trigger={<Button variant="secondary">Open Morph Modal</Button>}
                modalWidth={500}
                modalHeight={400}
              >
                {(close) => (
                  <div className="flex flex-col h-full">
                    <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
                      <h3 className="text-xs font-mono uppercase tracking-wider text-gray-900 dark:text-gray-100">
                        Morphing Modal
                      </h3>
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={close}
                        className="text-gray-500 hover:text-gray-900 dark:text-gray-400 dark:hover:text-gray-100"
                      >
                        ✕
                      </Button>
                    </div>
                    <div className="flex-1 overflow-y-auto p-4 space-y-4">
                      <p className="text-sm text-gray-600 dark:text-gray-400">
                        This modal morphs from the trigger button with a smooth
                        animation using Framer Motion.
                      </p>
                      <div className="space-y-2">
                        <Input placeholder="Try typing something..." />
                        <Button size="sm" className="w-full">Submit</Button>
                      </div>
                    </div>
                  </div>
                )}
              </MorphModal>
            </Card>
          </div>
        </section>

        {/* YAML Editor Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            YAML Editor
          </h2>
          <Paragraph variant="muted" className="text-sm">
            A code editor for resource documents. Includes a copy button (semi-transparent,
            top-right). Read-only mode shows a lock bar instead of greying out the content.
          </Paragraph>
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
            <Card className="p-6 space-y-3">
              <CardTitle>Editable YAML Editor</CardTitle>
              <CardDescription>
                Edit the resource document. Click Save to apply changes. The copy button
                is always available in the top-right corner.
              </CardDescription>
              <div className="h-72 flex flex-col">
                <YamlEditor
                  value={editableYamlValue}
                  onSave={handleYamlSave}
                  readOnlyFields={["state", "hash_code"]}
                  data-testid="gallery-yaml-editor-editable"
                />
              </div>
            </Card>

            <Card className="p-6 space-y-3">
              <CardTitle>Read-only YAML Editor</CardTitle>
              <CardDescription>
                Locked editors show a thin top bar with a lock icon — no greying out.
                Copy still works normally.
              </CardDescription>
              <div className="h-72 flex flex-col">
                <YamlEditor
                  value={readOnlyYamlValue}
                  disabled
                  data-testid="gallery-yaml-editor-readonly"
                />
              </div>
            </Card>
          </div>
        </section>

        {/* Footer */}
        <footer className="text-center text-sm text-gray-500 dark:text-gray-400 pt-8 border-t border-gray-200 dark:border-gray-800">
          <p>
            Built with React Router 7, Vite 7, TailwindCSS 4, Radix UI, and
            Framer Motion
          </p>
        </footer>
      </div>
    </div>
  );
}
