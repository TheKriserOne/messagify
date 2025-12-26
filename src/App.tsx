import { Suspense, lazy } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

// Lazy load components
const Login = lazy(() => import("./pages/./Login"));
const DiscordLayout = lazy(() => import("./layout/Discord/DiscordLayout"));

// Loading fallback component
function LoadingFallback() {
    return (
        <div style={{ padding: "20px", textAlign: "center" }}>
            <p>Loading...</p>
        </div>
    );
}

function App() {
    return (
        <BrowserRouter>
            <Suspense fallback={<LoadingFallback />}>
                <Routes>
                    <Route path="/" element={<Navigate to="/login" replace />} />
                    <Route path="/login" element={<Login />} />
                    <Route path="/user" element={<DiscordLayout/>} />
                    {/* <Route path="/user/:user_id" element={<Home />} /> */}
                </Routes>
            </Suspense>
        </BrowserRouter>
    );
}

export default App;
