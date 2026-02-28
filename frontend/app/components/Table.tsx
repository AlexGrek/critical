import React from "react";
import { cn } from "~/lib/utils";

// ---------------------------------------------------------------------------
// Table.Root — outer scroll wrapper + <table>
// ---------------------------------------------------------------------------

const TableRoot = React.forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement>
>(({ className, children, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("w-full overflow-x-auto", className)}
    {...props}
  >
    <table className="w-full border-collapse text-sm">{children}</table>
  </div>
));
TableRoot.displayName = "Table.Root";

// ---------------------------------------------------------------------------
// Table.Header — <thead>
// ---------------------------------------------------------------------------

const TableHeader = React.forwardRef<
  HTMLTableSectionElement,
  React.HTMLAttributes<HTMLTableSectionElement>
>(({ className, ...props }, ref) => (
  <thead
    ref={ref}
    className={cn(
      "bg-gray-50 dark:bg-gray-900/60",
      "border-b border-gray-200 dark:border-gray-800",
      className
    )}
    {...props}
  />
));
TableHeader.displayName = "Table.Header";

// ---------------------------------------------------------------------------
// Table.Body — <tbody>
// ---------------------------------------------------------------------------

const TableBody = React.forwardRef<
  HTMLTableSectionElement,
  React.HTMLAttributes<HTMLTableSectionElement>
>(({ className, ...props }, ref) => (
  <tbody
    ref={ref}
    className={cn(
      "divide-y divide-gray-100 dark:divide-gray-800/60",
      className
    )}
    {...props}
  />
));
TableBody.displayName = "Table.Body";

// ---------------------------------------------------------------------------
// Table.Row — <tr>
// ---------------------------------------------------------------------------

interface TableRowProps extends React.HTMLAttributes<HTMLTableRowElement> {
  selected?: boolean;
}

const TableRow = React.forwardRef<HTMLTableRowElement, TableRowProps>(
  ({ className, selected, ...props }, ref) => (
    <tr
      ref={ref}
      data-selected={selected || undefined}
      className={cn(
        "transition-colors",
        "hover:bg-gray-50/80 dark:hover:bg-gray-800/40",
        selected &&
          "bg-primary-50/40 dark:bg-primary-950/30 border-l-2 border-l-primary-500",
        className
      )}
      {...props}
    />
  )
);
TableRow.displayName = "Table.Row";

// ---------------------------------------------------------------------------
// Table.Head — <th>
// ---------------------------------------------------------------------------

const TableHead = React.forwardRef<
  HTMLTableCellElement,
  React.ThHTMLAttributes<HTMLTableCellElement>
>(({ className, ...props }, ref) => (
  <th
    ref={ref}
    className={cn(
      "px-4 py-3 text-left text-xs font-semibold uppercase tracking-wider",
      "text-gray-500 dark:text-gray-400 whitespace-nowrap",
      className
    )}
    {...props}
  />
));
TableHead.displayName = "Table.Head";

// ---------------------------------------------------------------------------
// Table.Cell — <td>
// ---------------------------------------------------------------------------

const TableCell = React.forwardRef<
  HTMLTableCellElement,
  React.TdHTMLAttributes<HTMLTableCellElement>
>(({ className, ...props }, ref) => (
  <td
    ref={ref}
    className={cn("px-4 py-3 text-gray-900 dark:text-gray-100", className)}
    {...props}
  />
));
TableCell.displayName = "Table.Cell";

// ---------------------------------------------------------------------------
// Table.Empty — full-width empty-state row (renders its own <tbody>)
// ---------------------------------------------------------------------------

const TableEmpty = ({
  colSpan,
  children,
  className,
}: {
  colSpan: number;
  children: React.ReactNode;
  className?: string;
}) => (
  <tbody>
    <tr>
      <td
        colSpan={colSpan}
        className={cn(
          "py-14 text-center text-sm text-gray-400 dark:text-gray-500",
          className
        )}
      >
        {children}
      </td>
    </tr>
  </tbody>
);
TableEmpty.displayName = "Table.Empty";

// ---------------------------------------------------------------------------
// Namespace export
// ---------------------------------------------------------------------------

export const Table = {
  Root: TableRoot,
  Header: TableHeader,
  Body: TableBody,
  Row: TableRow,
  Head: TableHead,
  Cell: TableCell,
  Empty: TableEmpty,
};
