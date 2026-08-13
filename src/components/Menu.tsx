import { useEffect } from "react";

export type CloseEventHandler = () => void;

export function Menu({
  onClose,
  open,
  ...props
}: React.ComponentPropsWithoutRef<"div"> & {
  onClose?: CloseEventHandler;
  open: boolean;
}) {
  useEffect(() => {
    if (!open) {
      return;
    }

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        onClose?.();
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose, open]);

  if (!open) {
    return null;
  }

  return (
    <div className="z-10">
      <div
        className="fixed inset-0"
        onClick={() => {
          onClose?.();
        }}
      />
      <div className="z-10">
        <div {...props} />
      </div>
    </div>
  );
}
