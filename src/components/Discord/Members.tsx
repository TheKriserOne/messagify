const MembersBar = () => {
  return (
    <div className="w-60 bg-gray-800 border-l border-gray-900 flex flex-col">
      <div className="h-12 border-b border-gray-900 px-4 flex items-center">
        <h3 className="text-xs font-semibold text-gray-400 uppercase">
          Members — 5
        </h3>
      </div>
      <div className="flex-1 overflow-y-auto px-2 py-2 space-y-1">
        {["Online", "Idle", "Do Not Disturb", "Offline"].map((status, idx) => (
          <div key={status} className="mb-2">
            <div className="px-2 py-1 text-xs font-semibold text-gray-400 uppercase">
              {status} — {idx + 1}
            </div>
            {[1, 2].map((i) => (
              <div
                key={i}
                className="px-2 py-1 rounded cursor-pointer flex items-center space-x-2 hover:bg-gray-700 group"
              >
                <div className="relative">
                  <div className="w-8 h-8 bg-indigo-500 rounded-full flex items-center justify-center">
                    <span className="text-white text-xs font-bold">U</span>
                  </div>
                  <div
                    className={`absolute -bottom-0.5 -right-0.5 w-3 h-3 rounded-full border-2 border-gray-800 ${
                      status === "Online"
                        ? "bg-green-500"
                        : status === "Idle"
                        ? "bg-yellow-500"
                        : status === "Do Not Disturb"
                        ? "bg-red-500"
                        : "bg-gray-500"
                    }`}
                  />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="text-sm font-medium text-gray-300 truncate">
                    User {idx * 2 + i}
                  </div>
                </div>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
};
export default MembersBar;