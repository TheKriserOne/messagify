import { useEffect } from "react";
import { useLocation, useNavigate, useParams } from "react-router-dom";
import DmChannelBar from "./DmChannelBar";
import GuildChannelBar from "./GuildChannelBar";

const ChannelBarController = () => {
  const location = useLocation();
  const { guildId } = useParams();
  const navigate = useNavigate();

  const isGuildPath = location.pathname.includes("/guild/");
  // Redirect to /discord/user if guild path but no guildId
  useEffect(() => {

    if (isGuildPath && !guildId) {
      navigate("/discord/user", { replace: true });
    }
  }, [isGuildPath, guildId]);

  return (
    <div className="w-60 bg-gray-800 flex flex-col">
        {isGuildPath && guildId ? <GuildChannelBar/> : <DmChannelBar/>}
    </div>
  );
};

export default ChannelBarController;
