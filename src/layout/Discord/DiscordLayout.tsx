import Guild from "../../components/Discord/GuildBar";
import ChannelBarController from "../../components/Discord/ChannelBar/ChannelBarController";
import Chat from "../../components/Discord/MainChat";
import MembersBar from "../../components/Discord/Members";

function DiscordLayout() {
  return (
    <div className="flex h-screen w-screen bg-gray-800 text-gray-100">
      {/* Guild Bar */}
      <Guild />

      {/* Channels Sidebar */}
      <ChannelBarController />

      {/* Main Chat Area */}
      <Chat />

      {/* Members Sidebar */}
      <MembersBar />
    </div>
  );
}

export default DiscordLayout;
