/**
 * Nyx messaging client — Matrix/Continuwuity via Heimdall.
 * /api/nyx/messaging/*  →  Continuwuity (Matrix CS API)
 */
import { api, type ApiResponse } from "../client";

export interface Room {
  room_id: string;
  name?: string;
  avatar_url?: string;
  last_message?: Message;
  unread_count: number;
  members: RoomMember[];
  created_at: string;
}

export interface RoomMember {
  user_id: string;
  alias: string;
  display_name: string;
  avatar_url?: string;
}

export interface Message {
  event_id: string;
  room_id: string;
  sender_id: string;
  sender_alias: string;
  body: string;
  type: "m.text" | "m.image" | "m.video" | "m.file";
  media_url?: string;
  timestamp: number;
  is_read?: boolean;
}

export interface SendMessageRequest {
  body: string;
  type?: "m.text" | "m.image";
  media_url?: string;
}

export const messagingApi = {
  getRooms(): Promise<ApiResponse<Room[]>> {
    return api.get<ApiResponse<Room[]>>("/api/nyx/messaging/rooms");
  },

  getRoom(roomId: string): Promise<Room> {
    return api.get<Room>(`/api/nyx/messaging/rooms/${roomId}`);
  },

  createDM(targetAlias: string): Promise<Room> {
    return api.post<Room>("/api/nyx/messaging/rooms/dm", {
      target_alias: targetAlias,
    });
  },

  getMessages(roomId: string, from?: string): Promise<ApiResponse<Message[]>> {
    const q = from ? `?from=${from}` : "";
    return api.get<ApiResponse<Message[]>>(
      `/api/nyx/messaging/rooms/${roomId}/messages${q}`,
    );
  },

  sendMessage(roomId: string, req: SendMessageRequest): Promise<Message> {
    return api.post<Message>(
      `/api/nyx/messaging/rooms/${roomId}/messages`,
      req,
    );
  },

  markRead(roomId: string, eventId: string): Promise<void> {
    return api.post<void>(`/api/nyx/messaging/rooms/${roomId}/read`, {
      event_id: eventId,
    });
  },
};
