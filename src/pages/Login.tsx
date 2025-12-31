import { useRef } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";

function Login() {
  const nameInputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  async function submit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault();
    const token = nameInputRef.current?.value || "";
    if (!token) {
      return;
    }
    try {
      const success = await invoke<boolean>("set_token", {
        token: token,
      });
      if (success) {
        navigate("/discord/user");
      }
    } catch (e) {
      console.error(e);
      return;
    }
  }

  return (
    <main className="bg-slate-900 h-screen w-screen flex justify-center items-center">
      <div>
        <h1 className="font-bold text-5xl text-center pb-5 text-indigo-500">
          Connect Discord
        </h1>
        <form
          className="bg-gray-700 rounded-3xl p-2 flex justify-center items-center space-x-4 outline-gray-500 outline-1"
          onSubmit={(e) => {
            submit(e);
          }}
        >
          {/*TODO: figure out why the background color changes when saved info pops up */}
          <input
            ref={nameInputRef}
            className="text-gray-200 px-5 p-2 focus:outline-none rounded-xl bg-transparent"
            id="greet-input"
            placeholder="Enter a token"
          />
          <button
            type="submit"
            className="font-bold px-8 py-4 bg-indigo-500 rounded-2xl
                            text-gray-200"
          >
            Confirm
          </button>
        </form>
      </div>
    </main>
  );
}

export default Login;
