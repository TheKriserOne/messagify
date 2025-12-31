export interface Guild {
  id: string;
  name: string;
  icon: string | null;
  // icon_hash: string | null;
  // splash: string | null;
  // discovery_splash: string | null;
  // owner: boolean | null;
  // owner_id: string;  // Snowflake
  // permissions: string | null;
  // region: string | null;  // Deprecated
  // afk_channel_id: string | null;  // Snowflake
  // afk_timeout: number;
  // widget_enabled: boolean | null;
  // widget_channel_id: string | null;  // Snowflake
  // verification_level: number;
  // default_message_notifications: number;
  // explicit_content_filter: number;
  // roles: Role[];
  // emojis: Emoji[];
  // features: string[];
  // mfa_level: number;
  // application_id: string | null;  // Snowflake
  // system_channel_id: string | null;  // Snowflake
  // system_channel_flags: number;
  // rules_channel_id: string | null;  // Snowflake
  // max_presences: number | null;
  // max_members: number | null;
  // vanity_url_code: string | null;
  // description: string | null;
  // banner: string | null;
  // premium_tier: number;
  // premium_subscription_count: number | null;
  // preferred_locale: string;
  // public_updates_channel_id: string | null;  // Snowflake
  // max_video_channel_users: number | null;
  // max_stage_video_channel_users: number | null;
  // approximate_member_count: number | null;
  // approximate_presence_count: number | null;
  // welcome_screen: WelcomeScreen | null;
  // nsfw_level: number;
  // stickers: Sticker[] | null;
  // premium_progress_bar_enabled: boolean;
  // safety_alerts_channel_id: string | null;  // Snowflake
  // incidents_data: IncidentsData | null;
}
export interface Channel {
  id: string;
  name: string;
  type: number;
  position: number;
  // permission_overwrites: PermissionOverwrite[];
  parent_id: string | null;
  rate_limit_per_user: number | null;
  topic: string | null;
  nsfw: boolean;
  last_message_id: string | null;
}

export interface DiscordUserLite {
  id: string;
  username: string;
  global_name?: string | null;
  avatar?: string | null;
}

// Discord "DM channel" objects returned from GET /users/@me/channels.
// We keep this minimal on purpose and parse in TypeScript.
export interface DmChannel {
  id: string;
  type: number; // 1 = DM, 3 = group DM
  name?: string | null;
  last_message_id?: string | null;
  recipients?: DiscordUserLite[];
}
