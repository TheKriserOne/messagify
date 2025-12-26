import {useRef} from "react";
import {invoke} from "@tauri-apps/api/core";

function Login() {
    const nameInputRef = useRef<HTMLInputElement>(null);
    async function submit(e: React.FormEvent<HTMLFormElement>) {
        e.preventDefault();
        const token = nameInputRef.current?.value || "";
        const response = await invoke<string>("set_token", {token});
        if (response) {
            console.error(response);
        }
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
                        submit(e);
                    }}
                >
                    <input
                        ref={nameInputRef}
                        id="greet-input"
                        placeholder="Enter a token"
                    />
                    <button type="submit">Login</button>
                </form>
            </div>
        </main>
    );
}

export default Login;
