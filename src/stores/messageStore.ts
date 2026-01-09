import { create } from "zustand";
import type { DiscordMessage } from "../types/discord";

type channel_id = string;
interface MessageState {

  // Map of channelId -> messages array
  messages: Map<channel_id, DiscordMessage[]>;

  // Set all messages for a channel (used for initial fetch)
  setMessages: (channelId: string, messages: DiscordMessage[]) => void;

  // Add a new message to a channel
  addMessage: (channelId: string, message: DiscordMessage) => void;

  // Update an existing message
  updateMessage: (
    channelId: string,
    messageId: string,
    message: Partial<DiscordMessage>
  ) => void;

  // Delete a message
  deleteMessage: (channelId: string, messageId: string) => void;

  // Clear messages for a channel
  clearChannel: (channelId: string) => void;

  // Clear all messages
  clearAll: () => void;
}

export const useMessageStore = create<MessageState>((set, get) => ({
  messages: new Map(),

  setMessages: (channelId, messages) =>
    set((state) => {
      const newMessages = new Map(state.messages);
      const existingMessages = newMessages.get(channelId) ?? [];
      
      // Create a Map for O(1) lookup of existing messages by ID
      const existingIds = new Set(existingMessages.map(m => m.id));
      
      // Filter out messages that already exist, then merge
      const newUniqueMessages = messages.filter(m => !existingIds.has(m.id));
      const merged = [...existingMessages, ...newUniqueMessages];
      
      // Sort by timestamp to maintain chronological order
      newMessages.set(channelId, merged);
      return { messages: newMessages };
    }),

  addMessage: (channelId, message) =>
    set((state) => {
      const newMessages = new Map(state.messages);
      // Get existing messages or create new array if channel doesn't exist
      const channelMessages = newMessages.get(channelId);
      if (channelMessages) {
        // Channel exists - insert new message if not duplicate
        if (!channelMessages.some((m) => m.id === message.id)) {
          newMessages.set(channelId, [message, ...channelMessages]);
        }
      } else {
        // Channel doesn't exist - create new entry with the message
        newMessages.set(channelId, [message]);
      }
      console.log(message);
      return { messages: newMessages };
    }),

  updateMessage: (channelId, messageId, updatedFields) =>
    set((state) => {
      const newMessages = new Map(state.messages);
      const channelMessages = newMessages.get(channelId);

      if (channelMessages) {
        const updatedMessages = channelMessages.map((m) =>
          m.id === messageId ? { ...m, ...updatedFields } : m
        );
        newMessages.set(channelId, updatedMessages);
      }

      return { messages: newMessages };
    }),

  deleteMessage: (channelId, messageId) =>
    set((state) => {
      const newMessages = new Map(state.messages);
      const channelMessages = newMessages.get(channelId);

      if (channelMessages) {
        newMessages.set(
          channelId,
          channelMessages.filter((m) => m.id !== messageId)
        );
      }

      return { messages: newMessages };
    }),
  clearChannel: (channelId) =>
    set((state) => {
      const newMessages = new Map(state.messages);
      newMessages.delete(channelId);
      return { messages: newMessages };
    }),

  clearAll: () => set({ messages: new Map() }),
}));
