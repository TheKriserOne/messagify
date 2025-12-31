import { useEffect, useState } from "react";
import type { Guild } from "../../types/discord";
import { invoke } from "@tauri-apps/api/core";
import { useLocation, useNavigate, useParams } from "react-router-dom";

const Guild = () => {
  const [guilds, setGuilds] = useState<Array<Guild>>([]);
  const [selectedId, setSelectedId] = useState<string>("home");
  const navigate = useNavigate();
  useEffect(() => {
    invoke<string>("fetch_user_guilds")
      .then((json) => {
        const parsed = JSON.parse(json) as Guild[];
        setGuilds(parsed ?? []);
      })
      .catch((e) => {
        console.error("Failed to fetch guilds:", e);
        setGuilds([]);
      });
      console.log(navigate);
  }, []);

  const renderGuildButton = (
    guild: Guild
  ) => {
    const isSelected = guild.id === selectedId;
    const iconUrl = guild.icon
      ? `https://cdn.discordapp.com/icons/${guild.id}/${guild.icon}.png?size=64`
      : null;

    return (
      <div key={guild.id} className="relative w-full flex justify-center">
        <div
          className={[
            "absolute left-0 top-1/2 -translate-y-1/2 rounded-r transition-all",
            isSelected ? "h-10 w-1 bg-white" : "h-2 w-1 bg-white/0",
          ].join(" ")}
        />

        <button
          type="button"
          title={guild.name}
          aria-label={guild.name}
          aria-pressed={isSelected}
          onClick={() => {setSelectedId(guild.id) 

            navigate(`/discord/guild/${guild.id}`)
          }}
          className={[
            "w-12 h-12 rounded-2xl flex items-center justify-center cursor-pointer transition-all outline-none overflow-hidden",
            "hover:rounded-xl",
            isSelected
              ? "bg-indigo-500 ring-2 ring-indigo-300/60"
              : "bg-gray-700 hover:bg-indigo-500",
          ].join(" ")}
        >
          {iconUrl ? (
            <img
              src={iconUrl}
              alt=""
              className="h-full w-full object-cover"
              draggable={false}
            />
          ) : (
            <span className="text-white font-bold">
              {guild.name?.slice(0, 1)?.toUpperCase() ?? "?"}
            </span>
          )}
        </button>
      </div>
    );
  };

  return (
    <div className="w-[72px] bg-gray-900 flex flex-col items-center py-3 space-y-2">
      {/* Home Button */}
      <div className="relative w-full flex justify-center" onClick={() => navigate("/discord/user")}>
        <div
          className={[
            "absolute left-0 top-1/2 -translate-y-1/2 rounded-r transition-all",
            selectedId === "home" ? "h-10 w-1 bg-white" : "h-2 w-1 bg-white/0",
          ].join(" ")}
        />

        <button
          type="button"
          title="Home"
          aria-label="Home"
          aria-pressed={selectedId === "home"}
          onClick={() => setSelectedId("home")}
          className={[
            "w-12 h-12 rounded-2xl flex items-center justify-center cursor-pointer transition-all outline-none overflow-hidden",
            "hover:rounded-xl",
            selectedId === "home"
              ? "bg-indigo-500 ring-2 ring-indigo-300/60"
              : "bg-gray-700 hover:bg-indigo-500",
          ].join(" ")}
        >
          <svg
            viewBox="0 0 24 24"
            fill="currentColor"
            aria-hidden="true"
            className="h-5 w-5 text-white"
          >
            <path d="M12 3.2 3 10.6v9.2c0 .7.5 1.2 1.2 1.2H9v-6.2c0-.7.5-1.2 1.2-1.2h3.6c.7 0 1.2.5 1.2 1.2V21h4.8c.7 0 1.2-.5 1.2-1.2v-9.2L12 3.2z" />
          </svg>
        </button>
      </div>

      {/* Guild Buttons */}
      {guilds.map(renderGuildButton)}

      <div className="pt-1">
        <button
          type="button"
          aria-label="Add Server"
          className="w-12 h-12 bg-gray-700 rounded-2xl flex items-center justify-center cursor-pointer hover:rounded-xl transition-all hover:bg-indigo-500"
        >
          <span className="text-white font-bold">+</span>
        </button>
      </div>
    </div>
  );
};

export default Guild;
