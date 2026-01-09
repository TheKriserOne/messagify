import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { Channel, ChannelType } from "../../../types/discord";
import { useNavigate, useParams } from "react-router-dom";

const GuildChannelBar = () => {
  function openChannel(chosenchannelId: string) {
    if (!guildId) return;
    if (channelId === chosenchannelId) return;
    navigate(`/discord/guild/${guildId}/${chosenchannelId}`);
  }

  const [channels, setChannels] = useState<Channel[]>([]);
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(
    new Set()
  );
  const { guildId, channelId } = useParams();
  const navigate = useNavigate();

  useEffect(() => {
    if (!guildId) {
      return;
    }
    invoke<string>("fetch_guild_channels", { guildId: guildId })
      .then((json) => {
        const parsed = JSON.parse(json) as Channel[];
        setChannels(parsed ?? []);
        // Expand all categories by default
        const categories = parsed.filter(
          (c) => c.type === ChannelType.GUILD_CATEGORY
        );
        setExpandedCategories(new Set(categories.map((c) => c.id)));
      })
      .catch((e) => {
        console.error("Failed to fetch guild channels:", e);
        setChannels([]);
      });
  }, [guildId]);

  // Separate categories from regular channels in a single pass
  const categories: Channel[] = [];
  const regularChannels: Channel[] = [];
  for (const c of channels) {
    (c.type === ChannelType.GUILD_CATEGORY ? categories : regularChannels).push(
      c
    );
  }
  categories.sort((a, b) => a.position - b.position);

  // Group channels by parent_id
  const channelsByCategory = new Map<string, Channel[]>();
  for (const channel of regularChannels) {
    const parentId = channel.parent_id ?? "__uncategorized__";
    const list = channelsByCategory.get(parentId) ?? [];
    list.push(channel);
    channelsByCategory.set(parentId, list);
  }

  // Sort channels within each category by position
 

  const toggleCategory = (categoryId: string) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(categoryId)) {
        next.delete(categoryId);
      } else {
        next.add(categoryId);
      }
      return next;
    });
  };

  const getChannelIcon = (channel: Channel) => {
    if (channel.type === ChannelType.GUILD_VOICE || channel.type === ChannelType.GUILD_STAGE_VOICE) {
      // Voice channel icon (speaker/sound waves)
      return (
        <svg
          className="w-4 h-4 mr-1.5"
          fill="currentColor"
          viewBox="0 0 20 20"
        >
          <path
            fillRule="evenodd"
            d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.617.793L4.383 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.383l4-4.793a1 1 0 011.617-.793zM14.657 2.929a1 1 0 011.414 0A9.972 9.972 0 0119 10a9.972 9.972 0 01-2.929 7.071 1 1 0 01-1.414-1.414A7.971 7.971 0 0017 10c0-2.21-.894-4.208-2.343-5.657a1 1 0 010-1.414zm-2.829 2.828a1 1 0 011.415 0A5.983 5.983 0 0115 10a5.984 5.984 0 01-1.757 4.243 1 1 0 01-1.415-1.415A3.984 3.984 0 0013 10a3.983 3.983 0 00-1.172-2.828 1 1 0 010-1.415z"
            clipRule="evenodd"
          />
        </svg>
      );
    }
    // Text channel icon (hash)
    return <span className="mr-1.5">#</span>;
  };

  const renderChannel = (channel: Channel) => {
    const isSelected = channelId === channel.id;
    return (
      <div
        key={channel.id}
        className={[
          "px-2 py-1 rounded cursor-pointer flex items-center",
          isSelected
            ? "bg-gray-700 text-white"
            : "hover:bg-gray-700/50 text-gray-300",
        ].join(" ")}
        onClick={() => openChannel(channel.id)}
      >
        {getChannelIcon(channel)}
        <span className="font-medium text-gray-300">{channel.name}</span>
      </div>
    );
  };

  const renderCategory = (category: Channel) => {
    const children = channelsByCategory.get(category.id) ?? [];
    const isExpanded = expandedCategories.has(category.id);

    if (children.length === 0) return null;

    return (
      <div key={category.id} className="mb-1">
        {/* Category Header (clickable to toggle) */}
        <div
          className="px-2 py-1 flex items-center cursor-pointer hover:bg-gray-700/30 rounded select-none"
          onClick={() => toggleCategory(category.id)}
        >
          {/* Chevron icon */}
          <svg
            className={`w-3 h-3 mr-1 text-gray-400 transition-transform ${
              isExpanded ? "rotate-90" : ""
            }`}
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fillRule="evenodd"
              d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z"
              clipRule="evenodd"
            />
          </svg>
          <span className="text-xs font-semibold text-gray-400 uppercase">
            {category.name}
          </span>
        </div>

        {/* Category Children (collapsible) */}
        {isExpanded && (
          <div className="ml-2 mt-1 space-y-1">
            {children.map(renderChannel)}
          </div>
        )}
      </div>
    );
  };

  // Uncategorized channels (sorted by position)
  const uncategorized = (
    channelsByCategory.get("__uncategorized__") ?? []
  ).sort((a, b) => a.position - b.position);

  return (
    <>
      {/* Server Header */}
      <div className="h-12 border-b border-gray-900 px-4 flex items-center shadow-sm">
        <h2 className="font-semibold text-white">Messagify</h2>
      </div>

      {/* Channels List */}
      <div className="flex-1 overflow-y-auto px-2 py-2">
        {/* Uncategorized channels */}
        {uncategorized.length > 0 && (
          <div className="mb-2">
            <div className="space-y-1">{uncategorized.map(renderChannel)}</div>
          </div>
        )}

        {/* Categorized channels */}
        {categories.map(renderCategory)}
      </div>
    </>
  );
};

export default GuildChannelBar;
