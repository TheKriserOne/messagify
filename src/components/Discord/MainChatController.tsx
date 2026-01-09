import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import type { DiscordMessage } from "../../types/discord";
import Chat from "./MainChat";
import { useMessageStore } from "../../stores/messageStore";

const EMPTY_ARRAY: DiscordMessage[] = [];
export default function ChatController() {
  const { channelId, userChannelId } = useParams();
  const targetId = channelId ?? userChannelId as string;
  const setMessages = useMessageStore((state) => state.setMessages);
  const messages = useMessageStore(
    (state) => state.messages.get(targetId) ?? EMPTY_ARRAY
  );
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    if (!targetId || messages.length >= 50)
      return void (setError(null), setLoading(false));
    let cancelled = false;
    setLoading(true);
    setError(null);

    invoke<string>("fetch_channel_messages", { channelId: targetId, limit: 50, before: messages[0]?.id ?? undefined })
      .then((json) => {
        if (!cancelled) {
          setMessages(targetId, JSON.parse(json) as DiscordMessage[]);
        }
      })
      .catch((e) => {
        if (!cancelled) {
          setError(String(e));
        }
      })
      .finally(() => !cancelled && setLoading(false));
    setLoading(false);
    return () => {
      cancelled = true;
    };
  }, [targetId]);
  return (
    <Chat
      channelId={targetId}
      channelTitle={targetId ?? "Select a channel"}
      messages={messages}
      loading={loading}
      error={error}
    />
  );
}
