import type { DiscordMessage } from "../../types/discord";

type MainChatProps = {
  channelTitle?: string;
  messages?: DiscordMessage[];
  loading?: boolean;
  error?: string | null;
};

const Chat = ({ channelTitle, messages, loading, error }: MainChatProps) => {
  const list = messages ?? [];
  console.log("list " + list);
  return (
    <div className="flex-1 flex flex-col">
      {/* Channel Header */}
      <div className="h-12 border-b border-gray-900 px-4 flex items-center shadow-sm">
        <span className="text-gray-400 mr-2">#</span>
        <h3 className="font-semibold text-white">
          {channelTitle ?? "Select a channel"}
        </h3>
      </div>

      {/* Messages — flex-col-reverse renders newest (index 0) at bottom
      Idk how well that would work in the future when I have to append messages but I guess we will find out */}
      <div className="flex-1 overflow-y-auto px-4 py-4 flex flex-col-reverse gap-4">
        {loading ? (
          <div className="text-sm text-gray-400">Loading messages…</div>
        ) : error ? (
          <div className="text-sm text-red-300">Failed to load: {error}</div>
        ) : list.length === 0 ? (
          <div className="text-sm text-gray-400">No messages.</div>
        ) : (
          list.map((m) => {
            const initials =
              (m.author.global_name ?? m.author.username ?? "?")
                .slice(0, 2)
                .toUpperCase() || "?";
            return (
              <div
                key={m.id}
                className="flex space-x-4 group hover:bg-gray-800/50 rounded px-2 py-1 -mx-2"
              >
                <div className="w-10 h-10 bg-indigo-500 rounded-full flex items-center justify-center shrink-0">
                  <span className="text-white text-sm font-bold">
                    {initials}
                  </span>
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-baseline space-x-2">
                    <span className="font-semibold text-white">
                      {m.author.global_name ?? m.author.username}
                    </span>
                    <span className="text-xs text-gray-400">
                      {new Date(m.timestamp).toLocaleString()}
                    </span>
                  </div>
                  <div className="text-gray-300 mt-1 whitespace-pre-wrap wrap-break-word">
                    {m.content || (
                      <span className="text-gray-500">(no content)</span>
                    )}
                  </div>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
};
export default Chat;
