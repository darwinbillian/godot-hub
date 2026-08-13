import clsx from "clsx";
import { SearchIcon, XIcon } from "lucide-react";
import { useRef } from "react";

export type ChangeEventHandler = (value: string) => void;

export function Search({
  onChange,
  onClick,
  value,
  ...props
}: Omit<React.ComponentPropsWithoutRef<"div">, "onChange"> & {
  onChange?: ChangeEventHandler;
  value?: string;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <div
      onClick={(e) => {
        onClick?.(e);
        if (!e.defaultPrevented) {
          inputRef.current?.focus();
        }
      }}
      {...props}
    >
      <SearchIcon size={16} />
      <input
        ref={inputRef}
        type="search"
        placeholder="Search"
        value={value}
        onChange={(e) => {
          onChange?.(e.target.value);
        }}
      />
      <button
        className={clsx("btn btn-ghost p-1", !value && "invisible")}
        onClick={() => {
          onChange?.("");
        }}
      >
        <XIcon size={16} />
      </button>
    </div>
  );
}
