import {Suspense, lazy} from "react";
import {BrowserRouter, Routes, Route} from "react-router-dom";
import "./App.css";

// Lazy load components
const Home = lazy(() => import("./pages/./Login"));
const About = lazy(() => import("./pages/About"));
const Settings = lazy(() => import("./pages/Settings"));

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
                    <Route path="/" element={<Home />} />
                    <Route path="/about" element={<About />} />
                    <Route path="/settings" element={<Settings />} />
                </Routes>
            </Suspense>
        </BrowserRouter>
    );
}

export default App;
