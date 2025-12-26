import {useState, useRef} from "react";
import {invoke} from "@tauri-apps/api/core";

function Login() {
    const [greetMsg, setGreetMsg] = useState("");
    const nameInputRef = useRef<HTMLInputElement>(null);

    async function greet() {
        const name = nameInputRef.current?.value || "";
        setGreetMsg(await invoke("greet", {name}));
    }

    return (
        <main className="bg-slate-900 h-screen w-screen flex justify-center items-center max-w-screen-xl">
            <div className="">
                <h1 className="font-bold text-5xl pb-5 text-indigo-500">Connect Discord</h1>
                <form
                    className="bg-gray-700 rounded-3xl p-2 flex justify-center items-center space-x-4 outline-gray-500 outline-1"
                    onSubmit={(e) => {
                        e.preventDefault();
                        greet();
                    }}
                >
                    <input
                        ref={nameInputRef}
                        className="text-gray-200 px-5 p-2 focus:outline-none rounded-xl"
                        id="greet-input"
                        placeholder="Enter a token"
                    />
                    <button type="submit"
                            className="font-bold px-8 py-4 bg-indigo-500 rounded-2xl
                            text-gray-200"> Confirm
                    </button>
                </form>
                <h3 className="absolute text-green-500 py-4">{greetMsg}</h3>
            </div>
        </main>
    );
}

export default Login;
