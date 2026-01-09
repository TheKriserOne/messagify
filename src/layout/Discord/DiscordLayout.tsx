import Guild from "../../components/Discord/GuildBar";
import MembersBar from "../../components/Discord/Members";
import { DiscordEventListener } from "../../hooks/useDiscordEvents";
import { Outlet } from "react-router-dom";

function DiscordLayout() {
  return (
    <div className="flex h-screen w-screen bg-gray-800 text-gray-100">
      <DiscordEventListener />

      {/* Guild Bar */}
      <Guild />

      <Outlet />

      {/* Members Sidebar */}
      <MembersBar />
    </div>
  );
}

export default DiscordLayout;
