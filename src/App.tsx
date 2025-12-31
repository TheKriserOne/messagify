import { Suspense, lazy } from "react";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";

// Lazy load components
const Login = lazy(() => import("./pages/Login"));
const DiscordLayout = lazy(() => import("./layout/Discord/DiscordLayout"));
const ProtectedRoutes = lazy(() => import("./contexts/ProtectedRoutes"));
const Accounts = lazy(() => import("./pages/Accounts"));
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
          <Route path="/" element={<Accounts />} />
          <Route path="/login" element={<Login />} />
          <Route element={<ProtectedRoutes />}>
            <Route path="/discord/user/:userId?" element={<DiscordLayout />} />
            <Route path="/discord/guild/:guildId?/:channelId?" element={<DiscordLayout />} />
          </Route>
        </Routes>
      </Suspense>
    </BrowserRouter>
  );
}
export default App;
