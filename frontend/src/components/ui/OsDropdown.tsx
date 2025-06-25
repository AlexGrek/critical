// OsDropdown.tsx
import React, { useRef, useEffect } from 'react';
import { Check } from 'lucide-react'; // Assuming lucide-react is used for icons
import './OsDropdown.css';

interface DropdownItem {
  id: string;
  label: string;
  description?: string;
  onClick: () => void;
  isSelected?: boolean;
}

interface OsDropdownProps {
  items: DropdownItem[];
  isOpen: boolean;
  onClose: () => void;
  triggerRef: React.RefObject<HTMLElement>; // Reference to the button that opens the dropdown
}

const OsDropdown: React.FC<OsDropdownProps> = ({ items, isOpen, onClose, triggerRef }) => {
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        dropdownRef.current &&
        !dropdownRef.current.contains(event.target as Node) &&
        triggerRef.current &&
        !triggerRef.current.contains(event.target as Node)
      ) {
        onClose();
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    } else {
      document.removeEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen, onClose, triggerRef]);

  if (!isOpen) return null;

  return (
    <div
      ref={dropdownRef}
      // Combine Tailwind classes with a custom class for backdrop-filter and animation
      className="absolute right-0 top-full mt-1 w-48 bg-gray-800/95 border border-gray-600/30 rounded-lg shadow-2xl z-50 overflow-hidden os-dropdown-custom-styles"
    >
      {items.map((item) => (
        <button
          key={item.id}
          onClick={() => {
            item.onClick();
            onClose(); // Close dropdown after item click
          }}
          className="w-full px-3 py-2 text-left hover:bg-gray-700/60 transition-colors duration-150 flex items-center justify-between"
        >
          <div>
            <div className="text-sm font-medium text-gray-100">{item.label}</div>
            {item.description && (
              <div className="text-xs text-gray-400">{item.description}</div>
            )}
          </div>
          {item.isSelected && (
            <Check className="w-4 h-4 text-blue-400" />
          )}
        </button>
      ))}
    </div>
  );
};

export default OsDropdown;