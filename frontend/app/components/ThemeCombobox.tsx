import { Fragment } from "react";
import {
  Listbox,
  ListboxButton,
  ListboxOption,
  ListboxOptions,
  Transition,
} from "@headlessui/react";
import { Check, ChevronDown, Moon, Sun, Eye, Heart, Flame, Droplets, Monitor } from "lucide-react";
import { useTheme, type Theme } from "~/contexts/ThemeContext";

type ThemeOption = {
  value: Theme;
  label: string;
  description: string;
  icon: typeof Sun | typeof Moon | typeof Eye | typeof Heart | typeof Flame | typeof Droplets | typeof Monitor;
};

const themes: ThemeOption[] = [
  {
    value: "light",
    label: "Light",
    description: "Bright and clear interface",
    icon: Sun,
  },
  {
    value: "dark",
    label: "Dark",
    description: "Easy on the eyes in low light",
    icon: Moon,
  },
  {
    value: "barbie",
    label: "Barbie",
    description: "Pink-focused light theme",
    icon: Heart,
  },
  {
    value: "fusion",
    label: "Fusion light",
    description: "Light blue accent, slightly blue backgrounds, medium roundness",
    icon: Droplets,
  },
  {
    value: "orange",
    label: "Orange",
    description: "Orange-tinted dark theme",
    icon: Flame,
  },
  {
    value: "grayscale",
    label: "Grayscale",
    description: "Neutral gray, no color tint",
    icon: Eye,
  },
  {
    value: "nostalgic95",
    label: "Nostalgic 95",
    description: "Windows 95/98 silver chrome, navy blue, zero roundness",
    icon: Monitor,
  },
];

export function ThemeCombobox() {
  const { theme, setTheme } = useTheme();
  const selectedTheme = themes.find((t) => t.value === theme) || themes[0];

  return (
    <Listbox
      value={selectedTheme}
      onChange={(value) => {
        if (value) setTheme(value.value);
      }}
    >
      <div className="relative w-full max-w-xs">
        <ListboxButton className="relative w-full cursor-pointer rounded-(--radius-component) border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-900 py-2.5 pl-10 pr-10 text-left text-sm text-gray-900 dark:text-gray-50 focus:border-primary-500 focus:outline-none focus:ring-2 focus:ring-primary-500/20 transition-colors">
          <span className="absolute inset-y-0 left-0 flex items-center pl-3">
            <selectedTheme.icon
              className="h-5 w-5 text-gray-500 dark:text-gray-400"
              aria-hidden="true"
            />
          </span>
          <span className="block truncate">{selectedTheme.label}</span>
          <span className="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3">
            <ChevronDown
              className="h-5 w-5 text-gray-400"
              aria-hidden="true"
            />
          </span>
        </ListboxButton>
        <Transition
          as={Fragment}
          leave="transition ease-in duration-100"
          leaveFrom="opacity-100"
          leaveTo="opacity-0"
        >
          <ListboxOptions className="absolute z-50 mt-2 w-full min-w-[320px] overflow-hidden rounded-(--radius-component) bg-white dark:bg-gray-900 shadow-lg ring-1 ring-black/5 dark:ring-white/10 focus:outline-none">
            {themes.map((themeOption) => (
              <ListboxOption
                key={themeOption.value}
                value={themeOption}
                className={({ focus }) =>
                  `relative cursor-pointer select-none py-3 pl-10 pr-10 transition-colors ${
                    focus
                      ? "bg-primary-50 dark:bg-primary-900/20 text-primary-900 dark:text-primary-100"
                      : "text-gray-900 dark:text-gray-100"
                  }`
                }
              >
                {({ selected, focus }) => (
                  <>
                    <div className="flex items-center gap-3">
                      <themeOption.icon
                        className={`h-5 w-5 ${
                          focus
                            ? "text-primary-600 dark:text-primary-400"
                            : "text-gray-400 dark:text-gray-500"
                        }`}
                        aria-hidden="true"
                      />
                      <div className="flex flex-col">
                        <span
                          className={`block truncate ${
                            selected ? "font-semibold" : "font-normal"
                          }`}
                        >
                          {themeOption.label}
                        </span>
                        <span className="text-xs text-gray-500 dark:text-gray-400">
                          {themeOption.description}
                        </span>
                      </div>
                    </div>
                    {selected && (
                      <span className="absolute inset-y-0 right-0 flex items-center pr-3">
                        <Check
                          className="h-5 w-5 text-primary-600 dark:text-primary-400"
                          aria-hidden="true"
                        />
                      </span>
                    )}
                  </>
                )}
              </ListboxOption>
            ))}
          </ListboxOptions>
        </Transition>
      </div>
    </Listbox>
  );
}
