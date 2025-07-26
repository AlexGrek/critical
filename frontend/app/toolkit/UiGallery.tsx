import Button from "./Button";
import Input from "./Input";
import LogoCritical, { LogoCriticalAnimated } from "./LogoCritical";
import { Modal } from "./Modal";
import TopPanel from "./TopPanel";
import { CodeBlock, Paragraph } from "./typography";

const UiGallery = () => {
    return (
        <div className="bg-black text-white min-h-screen p-10 font-sans">
            <TopPanel/>
            <h1 className="text-4xl font-bold font-mono mb-8">Component Gallery</h1>

            {/* Logo */}
            <div className="mb-10">
                <h2 className="text-2xl font-mono mb-4 border-b border-gray-700 pb-2">LogoCritical</h2>
                <div className="flex items-center space-x-6">
                    <LogoCritical size="sm" />
                    <LogoCritical size="md" />
                    <LogoCritical size="lg" />
                    <LogoCriticalAnimated/>
                </div>
            </div>

            {/* Buttons */}
            <div className="mb-10">
                <h2 className="text-2xl font-mono mb-4 border-b border-gray-700 pb-2">Buttons</h2>
                <div className="flex flex-wrap gap-4 items-center">
                    <Button appearance="primary">Primary</Button>
                    <Button appearance="red">Red</Button>
                    <Button appearance="subtle">Subtle</Button>
                    <Button appearance="ghost">Ghost</Button>
                    <Button appearance="link">Link</Button>
                    <Button appearance="primary" size="lg">Large Primary</Button>
                    <Button appearance="red" disabled>Disabled</Button>
                </div>
            </div>

            {/* Inputs */}
            <div className="mb-10">
                <h2 className="text-2xl font-mono mb-4 border-b border-gray-700 pb-2">Inputs</h2>
                <div className="max-w-sm space-y-4">
                    <Input placeholder="Standard font input..." />
                    <Input monospace placeholder="Monospace font input..." />
                    <Input placeholder="Disabled input..." disabled />
                </div>
            </div>

            {/* CodeBlock */}
            <div className="mb-10">
                <h2 className="text-2xl font-mono mb-4 border-b border-gray-700 pb-2">CodeBlock</h2>
                <CodeBlock>{`$ crit issues create --title "New bug" \\
    --description "Something is broken."`}</CodeBlock>
            </div>

            {/* Modal */}
            <div className="mb-10">
                <h2 className="text-2xl font-mono mb-4 border-b border-gray-700 pb-2">Modal</h2>
                <Modal.Root>
                    <Modal.Trigger asChild>
                        <Button>Open Deletion Modal</Button>
                    </Modal.Trigger>
                    <Modal.Content>
                        <Modal.Header>
                            Delete Repository
                        </Modal.Header>
                        <Paragraph className="text-sm text-gray-400">
                            Are you sure you want to delete the 'WebApp' repository? This action cannot be undone.
                        </Paragraph>
                        <Modal.Footer>
                            <Modal.Root>
                                <Modal.Trigger asChild>
                                    <Button appearance="subtle">Cancel</Button>
                                </Modal.Trigger>
                            </Modal.Root>
                            <Button appearance="red">Delete Repository</Button>
                        </Modal.Footer>
                    </Modal.Content>
                </Modal.Root>
            </div>

        </div>
    );
}

export default UiGallery;