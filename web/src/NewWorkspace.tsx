import { useEffect, useRef, useState } from "react";
import { Box, Cloud, LoaderCircle, X } from "lucide-react";
import { requestJson } from "./api";

export function NewWorkspace({
  workspaceId,
  onClose,
  onOpen,
}: {
  workspaceId: string | null;
  onClose: () => void;
  onOpen: (id: string) => void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const createdId = useRef(workspaceId);
  const [method, setMethod] = useState<"onshape" | "step">("onshape");
  const [url, setUrl] = useState("");
  const [file, setFile] = useState<File | null>(null);
  const [pending, setPending] = useState("");
  const [error, setError] = useState("");
  useEffect(() => {
    const element = dialog.current;
    element?.showModal();
    return () => element?.close();
  }, []);
  async function submit() {
    setError("");
    if (method === "step" && (!file || !/\.(step|stp)$/i.test(file.name))) {
      setError("Choose a STEP or STP file.");
      return;
    }
    if (method === "step" && file && file.size >= 100 * 1024 * 1024) {
      setError("Choose a STEP file smaller than 100 MB.");
      return;
    }
    setPending("Creating workspace…");
    try {
      const project = await requestJson<{ id: string }>("workspaces", "POST", {
        id: createdId.current,
      });
      createdId.current = project.id;
      await requestJson("workspace");
      if (method === "onshape") {
        setPending("Connecting to Onshape…");
        await requestJson("onshape/import", "POST", { url: url.trim() });
      } else {
        setPending("Importing STEP file…");
        const form = new FormData();
        form.append("file", file!);
        await requestJson("upload", "POST", form);
      }
      onOpen(project.id);
    } catch (e) {
      setError((e as Error).message);
      setPending("");
    }
  }
  return (
    <dialog
      ref={dialog}
      className="modal new-workspace"
      aria-labelledby="new-workspace-title"
      onCancel={(e) => {
        e.preventDefault();
        if (!pending) onClose();
      }}
    >
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void submit();
        }}
      >
        <div className="modal-head">
          <div>
            <span className="eyebrow">WORKSPACES</span>
            <h2 id="new-workspace-title">
              {workspaceId ? "Add parts to workspace" : "New workspace"}
            </h2>
          </div>
          <button
            type="button"
            className="icon-button"
            aria-label="Close new workspace"
            disabled={!!pending}
            onClick={onClose}
          >
            <X size={19} />
          </button>
        </div>
        <div className="new-workspace-methods">
          <button
            type="button"
            aria-pressed={method === "onshape"}
            disabled={!!pending}
            onClick={() => {
              setMethod("onshape");
              setError("");
            }}
          >
            <Cloud size={18} />
            From Onshape
          </button>
          <button
            type="button"
            aria-pressed={method === "step"}
            disabled={!!pending}
            onClick={() => {
              setMethod("step");
              setError("");
            }}
          >
            <Box size={18} />
            STEP file
          </button>
        </div>
        <div className="modal-body">
          <div hidden={method !== "onshape"}>
            <label className="field">
              <span>Onshape model link</span>
              <input
                autoFocus
                type="url"
                placeholder="Paste a Part Studio or assembly link"
                required={method === "onshape"}
                value={url}
                disabled={!!pending}
                onChange={(e) => setUrl(e.target.value)}
              />
            </label>
            <p className="new-workspace-help">
              The model will be downloaded using your connected Onshape account.
            </p>
          </div>
          <div hidden={method !== "step"}>
            <label className="field">
              <span>STEP file</span>
              <input
                type="file"
                accept=".step,.stp"
                required={method === "step"}
                disabled={!!pending}
                onChange={(e) => setFile(e.target.files?.[0] || null)}
              />
            </label>
            <p className="new-workspace-help">
              Choose a .STEP or .STP file, up to 100 MB.
            </p>
          </div>
          {error && (
            <p className="new-workspace-error" role="alert">
              {error}
            </p>
          )}
        </div>
        <div className="modal-footer">
          <button
            type="button"
            className="secondary"
            disabled={!!pending}
            onClick={onClose}
          >
            Cancel
          </button>
          <button
            type="submit"
            className="primary"
            disabled={!!pending || (method === "onshape" ? !url.trim() : !file)}
          >
            {pending && <LoaderCircle size={16} className="spin" />}
            {pending || (workspaceId ? "Import parts" : "Create workspace")}
          </button>
        </div>
      </form>
    </dialog>
  );
}
