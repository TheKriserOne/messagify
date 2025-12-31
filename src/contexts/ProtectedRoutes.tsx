import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Outlet, useLocation, useNavigate } from "react-router-dom";

export default function ProtectedRoutes() {
  const navigate = useNavigate();
  const location = useLocation();
  useEffect(() => {
    (async () => {
      const token = await invoke<string>("get_token").catch((err) => {
        navigate("/");
      });
    })();
  }, [location]);
  return <Outlet />;
}
