const Channel = () => {
  const channels = ["general", "random", "dev", "announcements"];

  return (
    <div className="w-60 bg-gray-800 flex flex-col">
      {/* Server Header */}
      <div className="h-12 border-b border-gray-900 px-4 flex items-center shadow-sm">
        <h2 className="font-semibold text-white">Messagify</h2>
      </div>

      {/* Channels List */}
      <div className="flex-1 overflow-y-auto px-2 py-2">
        <div className="mb-2">
          <div className="px-2 py-1 text-xs font-semibold text-gray-400 uppercase">
            Text Channels
          </div>
          <div className="space-y-1 mt-1">
            {channels.map((channel) => (
              <div
                key={channel}
                className={`px-2 py-1 rounded cursor-pointer flex items-center `}
              >
                <span className="mr-1.5">#</span>
                <span className="font-medium">{channel}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
};

export default Channel;
