import { useEffect } from "react";

export type CloseEventHandler = () => void;

export function Modal({
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

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
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
    <div className="fixed inset-0 z-10 flex items-center justify-center">
      <div
        className="animate-fade-in absolute inset-0 bg-black/70"
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
