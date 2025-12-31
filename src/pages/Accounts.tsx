import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";

export default function Accounts() {
  const navigate = useNavigate();

  const [token, setToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const t = await invoke<string | null>("get_token");
        if (!cancelled) setToken(t);
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, []);

  const hasToken = Boolean(token && token.length > 0);
  const maskedToken = token ? `${token.slice(0, 6)}…${token.slice(-4)}` : "";
  const avatarText = "D";

  return (
    <main className="min-h-screen w-screen bg-slate-950 text-slate-100">
      <div className="mx-auto max-w-5xl px-6 py-10">
        {/* Top bar */}
        <div className="flex items-start justify-between gap-4">
          <div>
            <h1 className="text-3xl font-bold tracking-tight text-slate-100">
              Choose an account
            </h1>
            <p className="mt-2 text-sm text-slate-400">
              Pick the profile you want to use to open your Discord-style
              workspace.
            </p>
          </div>

          <div className="flex items-center gap-2">
            <Link
              to="/login"
              className="rounded-md bg-slate-800 px-3 py-2 text-sm font-semibold text-slate-100 hover:bg-slate-700"
            >
              {hasToken ? "Replace token" : "Add token"}
            </Link>
            <button
              type="button"
              onClick={() => navigate("/discord/user")}
              disabled={!hasToken}
              className="rounded-md bg-indigo-600 px-3 py-2 text-sm font-semibold text-white hover:bg-indigo-500 disabled:cursor-not-allowed disabled:opacity-60"
            >
              Continue
            </button>
          </div>
        </div>

        {/* Content */}
        <div className="mt-8 grid grid-cols-1 gap-6 md:grid-cols-[380px_1fr]">
          {/* List */}
          <section className="rounded-xl border border-slate-800 bg-slate-900/40 p-3">
            <div className="px-2 pb-2 text-xs font-semibold uppercase tracking-wider text-slate-400">
              Account
            </div>

            <div className="space-y-2">
              <div
                className={[
                  "w-full rounded-lg border px-3 py-3 text-left",
                  hasToken
                    ? "border-indigo-500/60 bg-indigo-500/10"
                    : "border-slate-800 bg-slate-950/30",
                ].join(" ")}
              >
                <div className="flex items-center gap-3">
                  <div className="relative">
                    <div className="flex h-11 w-11 items-center justify-center rounded-full bg-slate-700 text-sm font-bold text-slate-100">
                      {avatarText}
                    </div>
                    <span
                      className={[
                        "absolute bottom-0 right-0 h-3 w-3 rounded-full ring-2 ring-slate-900",
                        hasToken ? "bg-emerald-500" : "bg-gray-500",
                      ].join(" ")}
                    />
                  </div>

                  <div className="min-w-0 flex-1">
                    <div className="flex items-center justify-between gap-2">
                      <div className="truncate font-semibold text-slate-100">
                        Discord
                      </div>
                      {hasToken ? (
                        <span className="text-xs font-semibold text-indigo-300">
                          Ready
                        </span>
                      ) : null}
                    </div>
                    <div className="mt-0.5 truncate text-xs text-slate-400">
                      {loading
                        ? "Checking saved token…"
                        : hasToken
                        ? `Saved token: ${maskedToken}`
                        : "No token saved yet"}
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div className="mt-3 border-t border-slate-800 pt-3">
              <Link
                to="/login"
                className="block rounded-lg border border-slate-800 bg-slate-950/30 px-3 py-3 text-sm font-semibold text-slate-100 hover:bg-slate-950/60"
              >
                {hasToken ? "Replace token" : "+ Add token"}
              </Link>
            </div>
          </section>

          {/* Preview */}
          <section className="rounded-xl border border-slate-800 bg-slate-900/40 p-6">
            {hasToken ? (
              <div className="flex h-full flex-col">
                <div className="flex items-center gap-4">
                  <div className="relative">
                    <div className="flex h-14 w-14 items-center justify-center rounded-2xl bg-indigo-600/80 text-lg font-black text-white">
                      {avatarText}
                    </div>
                  </div>
                  <div className="min-w-0">
                    <div className="truncate text-lg font-bold text-slate-100">
                      Discord
                    </div>
                    <div className="truncate text-sm text-slate-400">
                      {maskedToken}
                    </div>
                  </div>
                </div>

                <div className="mt-6 rounded-lg border border-slate-800 bg-slate-950/40 p-4">
                  <div className="text-xs font-semibold uppercase tracking-wider text-slate-400">
                    What happens next
                  </div>
                  <ul className="mt-3 space-y-2 text-sm text-slate-300">
                    <li className="flex gap-2">
                      <span className="mt-1 h-1.5 w-1.5 shrink-0 rounded-full bg-slate-500" />
                      You’ll open the Discord-like layout with your saved token.
                    </li>
                    <li className="flex gap-2">
                      <span className="mt-1 h-1.5 w-1.5 shrink-0 rounded-full bg-slate-500" />
                      You can add/remove accounts later (when we wire storage).
                    </li>
                  </ul>
                </div>

                <div className="mt-auto pt-6">
                  <div className="flex flex-wrap gap-2">
                    <button
                      type="button"
                      onClick={() => navigate("/discord/user")}
                      className="rounded-md bg-indigo-600 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-500"
                    >
                      Continue
                    </button>
                    <Link
                      to="/login"
                      className="rounded-md bg-slate-800 px-4 py-2 text-sm font-semibold text-slate-100 hover:bg-slate-700"
                    >
                      Replace token
                    </Link>
                  </div>
                  <p className="mt-3 text-xs text-slate-500">
                    Tip: this app is currently in “single Discord token” mode.
                  </p>
                </div>
              </div>
            ) : (
              <div className="flex h-full items-center justify-center text-sm text-slate-400">
                No token saved. Add one to continue.
              </div>
            )}
          </section>
        </div>
      </div>
    </main>
  );
}
