import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Channel } from "../../../types/discord";
import { useParams } from "react-router-dom";

const GuildChannelBar = () => {
  // guildId will be used when implementing guild channel fetching
  const [channels, setChannels] = useState<Channel[]>([]);
  const params = useParams();
  useEffect(() => {
    if (!params.guildId) {
      return;
    }
    invoke<string>("fetch_guild_channels", { guildId: params.guildId })
      .then((json) => {
        const parsed = JSON.parse(json) as Channel[];
        setChannels(parsed ?? []);
      })
      .catch((e) => {
        console.error("Failed to fetch guild channels:", e);
        setChannels([]);
      });
  }, [params.guildId]);
  return (
    <>
      {/* Server Header */}
      <div className="h-12 border-b border-gray-900 px-4 flex items-center shadow-sm">
        <h2 className="font-semibold text-white">Messagify</h2>
      </div>

      {/* Channels List */}
      <div className="flex-1 overflow-y-auto px-2 py-2">
        <div className="mb-2">
          <div className="px-2 py-1 text-xs font-semibold text-gray-400 uppercase">
            Text Channels
          </div>
          <div className="space-y-1 mt-1">
            {channels.map((channel) => (
              <div
                key={channel.id}
                className={`px-2 py-1 rounded cursor-pointer flex items-center hover:bg-gray-700/50`}
              >
                <span className="mr-1.5">#</span>
                <span className="font-medium text-gray-300">
                  {channel.name}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </>
  );
};

export default GuildChannelBar;
