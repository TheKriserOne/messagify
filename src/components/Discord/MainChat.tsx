const Chat = () => {
  return (
    <div className="flex-1 flex flex-col">
      {/* Channel Header */}
      <div className="h-12 border-b border-gray-900 px-4 flex items-center shadow-sm">
        <span className="text-gray-400 mr-2">#</span>
        <h3 className="font-semibold text-white"></h3>
      </div>

      {/* Messages */}
      <div className="flex-1 overflow-y-auto px-4 py-4 space-y-4">
        {[1, 2, 3, 4, 5].map((i) => (
          <div
            key={i}
            className="flex space-x-4 group hover:bg-gray-800/50 rounded px-2 py-1 -mx-2"
          >
            <div className="w-10 h-10 bg-indigo-500 rounded-full flex items-center justify-center shrink-0">
              <span className="text-white text-sm font-bold">U{i}</span>
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex items-baseline space-x-2">
                <span className="font-semibold text-white">User {i}</span>
                <span className="text-xs text-gray-400">
                  Today at {9 + i}:{30 + i * 2} AM
                </span>
              </div>
              <div className="text-gray-300 mt-1">
                This is a sample message in the channel. Message content goes
                here.
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
export default Chat;
