import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useMessageStore } from "../stores/messageStore";
import type { DiscordMessage, DiscordUserLite } from "../types/discord";

// Gateway event types matching Rust GatewayEvent enum
interface GatewayEventPayload {
  type:
    | "MessageCreate"
    | "MessageUpdate"
    | "MessageDelete"
    | "Ready"
    | "GatewayError"
    | "Connected"
    | "Disconnected";
  data?: unknown;
}

interface MessageUpdateData {
  id: string;
  channel_id: string;
  content?: string;
  edited_timestamp?: string | null;
}

interface MessageDeleteData {
  id: string;
  channel_id: string;
}

/**
 * Hook that listens to Discord Gateway events from Tauri and updates the Zustand store.
 * Should be called once at the app level (e.g., in DiscordLayout).
 */
export function useDiscordEvents() {
  const addMessage = useMessageStore((state) => state.addMessage);
  const updateMessage = useMessageStore((state) => state.updateMessage);
  const deleteMessage = useMessageStore((state) => state.deleteMessage);
  
  // Prevent duplicate listeners in StrictMode
  const listenerRef = useRef<UnlistenFn | null>(null);
  const isListeningRef = useRef(false);

  useEffect(() => {
    if (isListeningRef.current) return;
    isListeningRef.current = true;

    let unlisten: UnlistenFn | null = null;

    const setupListener = async () => {
      unlisten = await listen<GatewayEventPayload>(
        "discord-gateway",
        (event) => {
          const payload = event.payload;
          switch (payload.type) {
            case "MessageCreate": {
              console.log(payload.data);
              const data = payload.data as DiscordMessage;
              const message: DiscordMessage = {
                id: data.id,
                channel_id: data.channel_id,
                content: data.content,
                timestamp: data.timestamp,
                edited_timestamp: data.edited_timestamp,
                author: {
                  id: data.author.id,
                  username: data.author.username,
                  global_name: data.author.global_name,
                  avatar: data.author.avatar,
                },
              };
              console.log(
                "[Gateway] MESSAGE_CREATE:",
                data.channel_id,
                message
              );
              addMessage(data.channel_id, message);
              break;
            }

            case "MessageUpdate": {
              const data = payload.data as MessageUpdateData;
              console.log(
                "[Gateway] MESSAGE_UPDATE:",
                data.channel_id,
                data.id
              );
              updateMessage(data.channel_id, data.id, {
                content: data.content,
                edited_timestamp: data.edited_timestamp,
              });
              break;
            }

            case "MessageDelete": {
              const data = payload.data as MessageDeleteData;
              console.log(
                "[Gateway] MESSAGE_DELETE:",
                data.channel_id,
                data.id
              );
              deleteMessage(data.channel_id, data.id);
              break;
            }

            case "Ready":
              console.log("[Gateway] READY");
              break;

            case "Connected":
              console.log("[Gateway] Connected to Discord Gateway");
              break;

            case "Disconnected":
              console.log("[Gateway] Disconnected from Discord Gateway");
              break;

            case "GatewayError":
              console.error("[Gateway] Error:", payload.data);
              break;

            default:
              console.log("[Gateway] Unknown event:", payload);
          }
        }
      );

      listenerRef.current = unlisten;
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
      isListeningRef.current = false;
    };
  }, [addMessage, updateMessage, deleteMessage]);
}

/**
 * Component that initializes Discord event listeners.
 * Place this in your layout to start listening to Gateway events.
 */
export function DiscordEventListener() {
  useDiscordEvents();
  return null;
}
