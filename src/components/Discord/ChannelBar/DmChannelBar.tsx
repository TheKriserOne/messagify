import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { DmChannel } from "../../../types/discord";
import { useNavigate, useParams } from "react-router-dom";

const DmChannelBar = () => {
  const [channels, setChannels] = useState<DmChannel[]>([]);
  const { userChannelId } = useParams();
  const navigate = useNavigate();
  useEffect(() => {
    invoke<string>("fetch_user_channels")
      .then((json) => {
        const parsed = JSON.parse(json) as DmChannel[];
        setChannels(parsed ?? []);
      })
      .catch((e) => {
        console.error("Failed to fetch user channels:", e);
        setChannels([]);
      });
  }, []);

  return (
    <>
      {/* DM Header */}
      <div className="h-12 border-b border-gray-900 px-4 flex items-center shadow-sm">
        <h2 className="font-semibold text-white">Direct Messages</h2>
      </div>

      {/* DM Channels List */}
      <div className="flex-1 overflow-y-auto px-2 py-2">
        <div className="mb-2">
          <div className="px-2 py-1 text-xs font-semibold text-gray-400 uppercase">
            Direct Messages
          </div>
          <div className="space-y-1 mt-1">
            {channels.length === 0 ? (
              <div className="px-2 py-1 rounded flex items-center text-gray-300">
                <span className="text-sm">No DMs yet</span>
              </div>
            ) : (
              channels.map((channel) => {
                const nameFromRecipients =
                  channel.recipients
                    ?.map((r) => r.global_name ?? r.username)
                    .filter(Boolean)
                    .join(", ") ?? "";

                const label =
                  (channel.name ?? "").trim() ||
                  nameFromRecipients.trim() ||
                  "Unknown DM";

                const isSelected = userChannelId === channel.id;

                return (
                  <div
                    key={channel.id}
                    className={[
                      "px-2 py-1 rounded cursor-pointer flex items-center",
                      isSelected
                        ? "bg-gray-700 text-white"
                        : "hover:bg-gray-700/50 text-gray-300",
                    ].join(" ")}
                    onClick={() => {
                      navigate(`/discord/user/${channel.id}`);
                    }}
                  >
                    <span className="mr-2 text-gray-400">@</span>
                    <span className="text-sm font-medium">{label}</span>
                  </div>
                );
              })
            )}
          </div>
        </div>
      </div>
    </>
  );
};

export default DmChannelBar;
