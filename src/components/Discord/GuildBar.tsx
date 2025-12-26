const Guild = () => {
  return (
    <div className="w-16 bg-gray-900 flex flex-col items-center py-3 space-y-2">
      <div className="w-12 h-12 bg-indigo-500 rounded-2xl flex items-center justify-center cursor-pointer hover:rounded-xl transition-all">
        <span className="text-white font-bold">M</span>
      </div>
      <div className="w-12 h-12 bg-gray-700 rounded-2xl flex items-center justify-center cursor-pointer hover:rounded-xl transition-all hover:bg-indigo-500">
        <span className="text-white font-bold">D</span>
      </div>
      <div className="w-12 h-12 bg-gray-700 rounded-2xl flex items-center justify-center cursor-pointer hover:rounded-xl transition-all hover:bg-indigo-500">
        <span className="text-white font-bold">+</span>
      </div>
    </div>
  );
};

export default Guild;
