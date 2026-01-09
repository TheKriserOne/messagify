import { useState, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { DiscordMessage } from "../../types/discord";
import { useMessageStore } from "../../stores/messageStore";

type MainChatProps = {
  channelId?: string;
  channelTitle?: string;
  messages?: DiscordMessage[];
  loading?: boolean;
  error?: string | null;
};

const Chat = ({
  channelId,
  channelTitle,
  messages,
  loading,
  error,
}: MainChatProps) => {
  const list = messages ?? [];
  const [messageInput, setMessageInput] = useState("");
  const [sending, setSending] = useState(false);
  const addMessage = useMessageStore((state) => state.addMessage);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  // Auto-resize textarea
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.style.height = "auto";
      inputRef.current.style.height = `${inputRef.current.scrollHeight}px`;
    }
  }, [messageInput]);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!channelId || !messageInput.trim() || sending) return;

    const content = messageInput.trim();
    setMessageInput("");
    setSending(true);

    try {
      const json = await invoke<string>("send_message", {
        channelId: channelId,
        content: content,
      });
      const sentMessage = JSON.parse(json) as DiscordMessage;
      // Add the sent message to the store immediately
      addMessage(channelId, sentMessage);
    } catch (e) {
      console.error("Failed to send message:", e);
      // Restore the message input on error
      setMessageInput(content);
    } finally {
      setSending(false);
      // Reset textarea height
      if (inputRef.current) {
        inputRef.current.style.height = "auto";
      }
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      const form = e.currentTarget.form;
      if (form) {
        form.requestSubmit();
      }
    }
  };

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

      {/* Message Input */}
      {channelId && (
        <div className="border-t border-gray-900 px-4 py-3">
          <form onSubmit={handleSubmit} className="flex items-end gap-2">
            <div className="flex-1 relative">
              <textarea
                ref={inputRef}
                value={messageInput}
                onChange={(e) => setMessageInput(e.target.value)}
                onKeyDown={handleKeyDown}
                placeholder={`Message #${channelTitle ?? "channel"}`}
                disabled={sending}
                rows={1}
                className="w-full bg-gray-700 text-white placeholder-gray-400 rounded-lg px-4 py-2 pr-12 resize-none focus:outline-none focus:ring-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed max-h-48 overflow-y-auto"
              />
            </div>
            <button
              type="submit"
              disabled={!messageInput.trim() || sending}
              className="px-4 py-2 bg-indigo-500 text-white rounded-lg hover:bg-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors font-medium"
            >
              {sending ? "Sending..." : "Send"}
            </button>
          </form>
        </div>
      )}
    </div>
  );
};
export default Chat;
