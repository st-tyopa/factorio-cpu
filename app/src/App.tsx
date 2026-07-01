import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import CodeMirror from "@uiw/react-codemirror";
import { json as jsonLang } from "@codemirror/lang-json";
import { oneDark } from "@codemirror/theme-one-dark";
import "./App.css";

// ── Types ─────────────────────────────────────────────────────────────────────

interface DocEntry {
  id: string;
  title: string;
  content: string;
  path: string;
}

type Page = "blueprint" | string; // string = doc id

// ── Blueprint page ────────────────────────────────────────────────────────────

function BlueprintPage() {
  const [blueprint, setBlueprint] = useState("");
  const [json, setJson] = useState("");
  const [status, setStatus] = useState<{ text: string; ok: boolean } | null>(null);

  async function decodeBlueprint() {
    try {
      const result = await invoke<string>("decode_blueprint", { bp: blueprint.trim() });
      setJson(result);
      setStatus({ text: "Декодировано успешно", ok: true });
    } catch (e) {
      setStatus({ text: `Ошибка: ${e}`, ok: false });
    }
  }

  async function encodeBlueprint() {
    try {
      const result = await invoke<string>("encode_blueprint", { json });
      setBlueprint(result);
      setStatus({ text: "Закодировано успешно", ok: true });
    } catch (e) {
      setStatus({ text: `Ошибка: ${e}`, ok: false });
    }
  }

  return (
    <div className="bp-layout">
      <div className="panels">
        <div className="panel">
          <div className="panel-header">
            <span className="panel-title">Blueprint</span>
            <button className="btn" onClick={decodeBlueprint}>Decode → JSON</button>
          </div>
          <textarea
            className="editor"
            value={blueprint}
            onChange={(e) => setBlueprint(e.target.value)}
            placeholder="Вставьте blueprint string..."
            spellCheck={false}
          />
        </div>
        <div className="panel">
          <div className="panel-header">
            <span className="panel-title">JSON</span>
            <button className="btn" onClick={encodeBlueprint}>Encode → Blueprint</button>
          </div>
          <div className="editor-cm">
            <CodeMirror
              value={json}
              onChange={setJson}
              extensions={[jsonLang()]}
              theme={oneDark}
              height="100%"
              style={{ height: "100%" }}
              basicSetup={{ lineNumbers: true, foldGutter: true }}
            />
          </div>
        </div>
      </div>
      {status && (
        <div className={`status ${status.ok ? "status-ok" : "status-err"}`}>
          {status.text}
        </div>
      )}
    </div>
  );
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function resolvePath(basePath: string, href: string): string {
  const baseDir = basePath.includes("/")
    ? basePath.slice(0, basePath.lastIndexOf("/") + 1)
    : "";
  const parts = (baseDir + href).split("/");
  const out: string[] = [];
  for (const p of parts) {
    if (p === "..") out.pop();
    else if (p !== ".") out.push(p);
  }
  return out.join("/");
}

// ── Doc page ──────────────────────────────────────────────────────────────────

function DocPage({
  doc,
  pathToId,
  onNavigate,
}: {
  doc: DocEntry;
  pathToId: Record<string, string>;
  onNavigate: (id: string) => void;
}) {
  const components = {
    a({ href, children }: { href?: string; children?: React.ReactNode }) {
      if (!href) return <span>{children}</span>;

      if (!href.startsWith("http") && !href.startsWith("#")) {
        const resolved = resolvePath(doc.path, href.split("#")[0]);
        const targetId = pathToId[resolved];
        if (targetId) {
          return (
            <a href="#" onClick={(e) => { e.preventDefault(); onNavigate(targetId); }}>
              {children}
            </a>
          );
        }
      }

      return <a href={href} target="_blank" rel="noopener noreferrer">{children}</a>;
    },
  };

  return (
    <div className="doc-page">
      <Markdown remarkPlugins={[remarkGfm]} components={components}>
        {doc.content}
      </Markdown>
    </div>
  );
}

// ── App ───────────────────────────────────────────────────────────────────────

export default function App() {
  const [page, setPage] = useState<Page>("blueprint");
  const [docs, setDocs] = useState<DocEntry[]>([]);

  useEffect(() => {
    invoke<DocEntry[]>("get_docs").then(setDocs).catch(console.error);
  }, []);

  const pathToId = useMemo(() => {
    const map: Record<string, string> = {};
    docs.forEach((d) => { map[d.path] = d.id; });
    return map;
  }, [docs]);

  const currentDoc = docs.find((d) => d.id === page);

  return (
    <div className="app">
      <nav className="navbar">
        <NavItem id="blueprint" label="Blueprint Tool" current={page} onClick={setPage} />
        <div className="nav-divider" />
        {docs.map((doc) => (
          <NavItem key={doc.id} id={doc.id} label={doc.title} current={page} onClick={setPage} />
        ))}
      </nav>

      <main className="main">
        {page === "blueprint" && <BlueprintPage />}
        {currentDoc && (
          <DocPage doc={currentDoc} pathToId={pathToId} onNavigate={setPage} />
        )}
      </main>
    </div>
  );
}

function NavItem({
  id, label, current, onClick,
}: {
  id: string; label: string; current: string; onClick: (id: string) => void;
}) {
  return (
    <button
      className={`nav-item ${current === id ? "nav-item--active" : ""}`}
      onClick={() => onClick(id)}
    >
      {label}
    </button>
  );
}
