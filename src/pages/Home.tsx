import { useState, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

function Home() {
  const [greetMsg, setGreetMsg] = useState("");
  const nameInputRef = useRef<HTMLInputElement>(null);

  async function greet() {
    const name = nameInputRef.current?.value || "";
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="h-screen container">
        <div className="flex flex-col items-center justify-center">
            <div className="">
                <h1 className="font-bold text-5xl py-10">LOGIN</h1>
            </div>

            <form
                className="row"
                onSubmit={(e) => {
                    e.preventDefault();
                    greet();
                }}
            >
                <input
                    ref={nameInputRef}
                    id="greet-input"
                    placeholder="Enter a token"
                />
                <button type="submit">Login</button>
            </form>
            <p>{greetMsg}</p>
        </div>
    </main>
  );
}

export default Home;
