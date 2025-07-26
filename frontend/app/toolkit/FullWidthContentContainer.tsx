import React from 'react';
import { cn } from '~/lib/utils'; // Utility function for class merging

// Base container props interface
interface BaseContainerProps {
  children: React.ReactNode;
  className?: string;
  padding?: 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  paddingX?: 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  paddingY?: 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  paddingTop?: 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  paddingBottom?: 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  paddingLeft?: 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  paddingRight?: 'none' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';
  textAlign?: 'left' | 'center' | 'right' | 'justify';
}

// Polymorphic component props
type PolymorphicComponentProps<T extends React.ElementType> = {
  as?: T;
} & BaseContainerProps &
  Omit<React.ComponentPropsWithoutRef<T>, keyof BaseContainerProps>;

type PolymorphicRef<T extends React.ElementType> = React.ComponentPropsWithRef<T>['ref'];

// Padding class mappings
const paddingClasses = {
  none: '',
  sm: 'p-4',
  md: 'p-6',
  lg: 'p-8',
  xl: 'p-12',
  '2xl': 'p-16',
};

const paddingXClasses = {
  none: '',
  sm: 'px-4',
  md: 'px-6',
  lg: 'px-8',
  xl: 'px-12',
  '2xl': 'px-16',
};

const paddingYClasses = {
  none: '',
  sm: 'py-4',
  md: 'py-6',
  lg: 'py-8',
  xl: 'py-12',
  '2xl': 'py-16',
};

const paddingTopClasses = {
  none: '',
  sm: 'pt-4',
  md: 'pt-6',
  lg: 'pt-8',
  xl: 'pt-12',
  '2xl': 'pt-16',
};

const paddingBottomClasses = {
  none: '',
  sm: 'pb-4',
  md: 'pb-6',
  lg: 'pb-8',
  xl: 'pb-12',
  '2xl': 'pb-16',
};

const paddingLeftClasses = {
  none: '',
  sm: 'pl-4',
  md: 'pl-6',
  lg: 'pl-8',
  xl: 'pl-12',
  '2xl': 'pl-16',
};

const paddingRightClasses = {
  none: '',
  sm: 'pr-4',
  md: 'pr-6',
  lg: 'pr-8',
  xl: 'pr-12',
  '2xl': 'pr-16',
};

const textAlignClasses = {
  left: 'text-left',
  center: 'text-center',
  right: 'text-right',
  justify: 'text-justify',
};

// Helper function to get padding classes
const getPaddingClasses = (props: BaseContainerProps) => {
  const classes: string[] = [];
  
  // Apply general padding first
  if (props.padding) {
    classes.push(paddingClasses[props.padding]);
  }
  
  // Apply specific padding overrides
  if (props.paddingX) {
    classes.push(paddingXClasses[props.paddingX]);
  }
  if (props.paddingY) {
    classes.push(paddingYClasses[props.paddingY]);
  }
  if (props.paddingTop) {
    classes.push(paddingTopClasses[props.paddingTop]);
  }
  if (props.paddingBottom) {
    classes.push(paddingBottomClasses[props.paddingBottom]);
  }
  if (props.paddingLeft) {
    classes.push(paddingLeftClasses[props.paddingLeft]);
  }
  if (props.paddingRight) {
    classes.push(paddingRightClasses[props.paddingRight]);
  }
  
  return classes.join(' ');
};

/**
 * FullWidthTransparentContentContainer
 * 
 * A full-width container with transparent background, no shadows, borders, or border radius.
 * Commonly used padding options with default left text alignment.
 */
export const FullWidthTransparentContentContainer = <T extends React.ElementType = 'div'>(
  {
    children,
    className,
    padding = 'md', // Default medium padding
    paddingX,
    paddingY,
    paddingTop,
    paddingBottom,
    paddingLeft,
    paddingRight,
    textAlign = 'left', // Default left alignment
    as,
    ...props
  }: PolymorphicComponentProps<T> & { ref?: PolymorphicRef<T> }
) => {
  const Component = as || 'div';
  
  const paddingClasses = getPaddingClasses({
      padding,
      paddingX,
      paddingY,
      paddingTop,
      paddingBottom,
      paddingLeft,
      paddingRight,
      children: undefined
  });

  const textAlignClass = textAlignClasses[textAlign];

  return (
    <Component
      className={cn(
        'w-full bg-transparent',
        paddingClasses,
        textAlignClass,
        className
      )}
      {...props}
    >
      {children}
    </Component>
  );
};

/**
 * FullWidthContentContainer
 * 
 * A full-width container with gray-900 background for dark theme.
 * Same padding and text alignment options as the transparent variant.
 */
export const FullWidthContentContainer = <T extends React.ElementType = 'div'>(
  {
    children,
    className,
    padding = 'md', // Default medium padding
    paddingX,
    paddingY,
    paddingTop,
    paddingBottom,
    paddingLeft,
    paddingRight,
    textAlign = 'left', // Default left alignment
    as,
    ...props
  }: PolymorphicComponentProps<T> & { ref?: PolymorphicRef<T> }
) => {
  const Component = as || 'div';
  
  const paddingClasses = getPaddingClasses({
      padding,
      paddingX,
      paddingY,
      paddingTop,
      paddingBottom,
      paddingLeft,
      paddingRight,
      children: undefined
  });

  const textAlignClass = textAlignClasses[textAlign];

  return (
    <Component
      className={cn(
        'w-full bg-gray-900',
        paddingClasses,
        textAlignClass,
        className
      )}
      {...props}
    >
      {children}
    </Component>
  );
};
