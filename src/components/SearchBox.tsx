import { SearchIcon, XIcon } from "lucide-react";
import { useRef } from "react";

export type ChangeEventHandler = (value: string) => void;

export function SearchBox({
  onChange,
  value,
  ...props
}: Omit<React.ComponentPropsWithoutRef<"div">, "onChange"> & {
  onChange?: ChangeEventHandler;
  value?: string;
}) {
  const inputRef = useRef<HTMLInputElement>(null);

  return (
    <div
      onClick={() => {
        inputRef.current?.focus();
      }}
    >
      <div {...props}>
        <SearchIcon size={16} />
        <input
          ref={inputRef}
          type="search"
          placeholder="Search"
          value={value}
          onChange={(event) => {
            onChange?.(event.target.value);
          }}
        />
        <button
          className="btn btn-ghost p-1"
          style={{ visibility: value ? "visible" : "hidden" }}
          onClick={() => {
            onChange?.("");
          }}
        >
          <XIcon size={16} />
        </button>
      </div>
    </div>
  );
}
