import { useState } from "react";
import {
  Button,
  Input,
  Modal,
  MorphModal,
  LogoCritical,
  LogoCriticalAnimated,
} from "~/components";

export function meta() {
  return [
    { title: "UI Gallery - Critical" },
    { name: "description", content: "Component showcase for Critical UI" },
  ];
}

export default function UiGallery() {
  const [inputValue, setInputValue] = useState("");

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
            UI Component Gallery
          </h1>
          <p className="text-gray-600 dark:text-gray-400">
            A showcase of all available UI components
          </p>
        </header>

        {/* Logos Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Logos
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Static Logo (Small)
              </h3>
              <LogoCritical size="sm" />
            </div>
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Static Logo (Medium)
              </h3>
              <LogoCritical size="md" />
            </div>
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Static Logo (Large)
              </h3>
              <LogoCritical size="lg" />
            </div>
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Animated Logo (Hover)
              </h3>
              <LogoCriticalAnimated size="md" />
            </div>
          </div>
        </section>

        {/* Buttons Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Buttons
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {/* Primary */}
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Primary
              </h3>
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
            </div>

            {/* Secondary */}
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Secondary
              </h3>
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
            </div>

            {/* Destructive */}
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Destructive
              </h3>
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
            </div>

            {/* Outline */}
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Outline
              </h3>
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
            </div>

            {/* Ghost */}
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Ghost
              </h3>
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
            </div>

            {/* Link */}
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Link
              </h3>
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
            </div>
          </div>
        </section>

        {/* Inputs Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Inputs
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Default Input
              </h3>
              <Input
                placeholder="Enter some text..."
                value={inputValue}
                onChange={(e) => setInputValue(e.target.value)}
              />
            </div>
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Monospace Input
              </h3>
              <Input
                monospace
                placeholder="Monospace font..."
                defaultValue="const x = 42;"
              />
            </div>
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Disabled Input
              </h3>
              <Input placeholder="Disabled..." disabled value="Can't edit me" />
            </div>
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Password Input
              </h3>
              <Input type="password" placeholder="Enter password..." />
            </div>
          </div>
        </section>

        {/* Modals Section */}
        <section className="space-y-4">
          <h2 className="text-2xl font-semibold text-gray-900 dark:text-gray-50">
            Modals
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Standard Modal */}
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Standard Modal
              </h3>
              <Modal.Root>
                <Modal.Trigger asChild>
                  <Button>Open Modal</Button>
                </Modal.Trigger>
                <Modal.Content>
                  <Modal.Header>
                    <Modal.Title>Example Modal</Modal.Title>
                    <Modal.Description>
                      This is a standard modal dialog built with Radix UI.
                    </Modal.Description>
                  </Modal.Header>
                  <div className="py-4">
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                      This modal includes a header, body content, and a footer
                      with action buttons.
                    </p>
                  </div>
                  <Modal.Footer>
                    <Modal.Close asChild>
                      <Button variant="outline">Cancel</Button>
                    </Modal.Close>
                    <Button>Confirm</Button>
                  </Modal.Footer>
                </Modal.Content>
              </Modal.Root>
            </div>

            {/* Morph Modal */}
            <div className="rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 p-6 space-y-3">
              <h3 className="font-medium text-gray-900 dark:text-gray-50">
                Morph Modal
              </h3>
              <MorphModal
                trigger={<Button variant="secondary">Open Morph Modal</Button>}
                modalWidth={500}
                modalHeight={400}
              >
                <div className="space-y-4">
                  <h3 className="text-xl font-bold text-gray-900 dark:text-gray-50">
                    Morphing Modal
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400">
                    This modal morphs from the trigger button with a smooth
                    animation using Framer Motion.
                  </p>
                  <div className="space-y-2">
                    <Input placeholder="Try typing something..." />
                    <Button className="w-full">Submit</Button>
                  </div>
                </div>
              </MorphModal>
            </div>
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
